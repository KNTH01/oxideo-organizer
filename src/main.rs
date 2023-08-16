use anyhow::Result;
use chrono::NaiveDateTime;
use clap::Parser;
use indicatif::ProgressBar;
use std::{fs, path::PathBuf, time::Duration};
use tracing::{debug, error, info, instrument, warn, Instrument, Level};

#[derive(Parser)]
#[command(name = "Oxideo Organizer")]
#[command(author = "KNTH")]
#[command(version = "0.1.0")]
#[command(about = "Automagically sort photos for you!", long_about = None)]
pub struct Cli {
    input: String,
    output: String,
}

pub struct Counter {
    all: u32,
    media: u32,
    processed: u32,
}

#[instrument]
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        // .with_max_level(Level::DEBUG)
        .init();

    let cli = Cli::parse();

    let mut non_media_paths: Vec<String> = vec![];

    let mut counter = Counter {
        all: 0,
        media: 0,
        processed: 0,
    };

    let bar = ProgressBar::new(1000);
    parse_input_dir(
        &cli.input,
        &cli.output,
        &mut counter,
        &mut non_media_paths,
        &bar,
    )?;

    info!("Successfully parsed the input directory. There are {} files, in which {} are detected media. {} got processed.", 
        counter.all, counter.media, counter.processed
    );

    if counter.all > counter.media {
        for path in non_media_paths {
            warn!("Non media files: {}", path);
        }
    }

    Ok(())
}

fn parse_input_dir(
    input: &str,
    output: &str,
    counter: &mut Counter,
    non_media_paths: &mut Vec<String>,
    bar: &ProgressBar,
) -> Result<()> {
    let ext_list = [
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico", "heic", "webp", "svg", "raw", "mp4",
        "mov", "avi", "3gp", "mkv", "flv", "wmv", "mpeg", "webm",
    ];

    match fs::read_dir(input) {
        Ok(paths) => {
            for path in paths {
                match path {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        let path_display = path.display();
                        let ext = path.extension().and_then(std::ffi::OsStr::to_str);
                        let is_media = ext
                            .map(|e| ext_list.contains(&e.to_lowercase().as_str()))
                            .unwrap_or(false);

                        if is_media {
                            debug!("Media file: {}", path_display);
                            counter.media += 1;

                            match read_metadata(path.to_str().unwrap()) {
                                Some(datetime) => {
                                    copy_datetime_media(&path, output, &datetime)?;
                                    counter.processed += 1;
                                    bar.inc(1);
                                }
                                None => {
                                    warn!("Cannot get media DateTimeOriginal");
                                    copy_untouched_media(&path, output)?;
                                    bar.inc(1);
                                }
                            }
                        } else if path.is_dir() {
                            parse_input_dir(
                                path.to_str().unwrap(),
                                output,
                                counter,
                                non_media_paths,
                                bar,
                            )?;
                        } else if path.is_file() {
                            non_media_paths.push(path_display.to_string());
                            copy_untouched_media(&path, output)?;
                            bar.inc(1);
                        }
                    }
                    Err(e) => tracing::error!("Error reading directory: {}", e),
                }
                counter.all += 1;
            }
        }
        Err(e) => {
            error!("Error listing directory: {}", e);
        }
    }

    Ok(())
}

fn copy_datetime_media(path: &PathBuf, output_dir: &str, datetime: &NaiveDateTime) -> Result<()> {
    let format_year = datetime.format("%Y").to_string();
    let format_month = datetime.format("%m").to_string();
    let format_date = datetime.format("%Y-%m-%d %T").to_string();
    debug!("creation time: {format_date}");

    let mut output_dir = std::path::PathBuf::from(output_dir);
    output_dir.push("");
    let output_dir = output_dir.to_string_lossy().into_owned();
    let target_path = format!("{output_dir}/{format_year}/{format_year}-{format_month}");
    fs::create_dir_all(&target_path)?;

    let dest_media_file_name = path.file_name().unwrap();
    let mut dest_media_path = PathBuf::from(target_path);
    dest_media_path.push(dest_media_file_name);

    fs::copy(path, &dest_media_path)?;
    Ok(())
}

fn copy_untouched_media(path: &PathBuf, dest: &str) -> Result<()> {
    let mut dest_media_path = PathBuf::new();

    dest_media_path.push(dest);
    dest_media_path.push("untouched");
    dest_media_path.push(path);

    let mut dest_dir = dest_media_path.clone();
    dest_dir.pop();

    fs::create_dir_all(dest_dir)?;
    fs::copy(path, &dest_media_path)?;

    Ok(())
}

fn read_metadata(path: &str) -> Option<NaiveDateTime> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("Cannot open file: {e}");
            return None;
        }
    };

    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => match exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
            Some(res) => {
                let datetime = res.display_value().to_string();
                if let Ok(datetime) = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
                {
                    debug!(
                        "EXIF metadata DateTimeOriginal found: {}",
                        datetime.to_string()
                    );
                    return Some(datetime);
                }
            }
            None => {
                warn!("Cannot found EXIF metadata field DateTimeOriginal");
                return None;
            }
        },
        Err(e) => error!("Cannot read EXIF metadata: {e}"),
    };

    None
}
