use chrono::{DateTime, Local};
use clap::Parser;
use std::{
    fs,
    io::{self, Error},
    path::PathBuf,
};
use tracing::{debug, error, info, instrument, warn, Level};

#[derive(Parser)]
#[command(name = "Oxideo Organizer")]
#[command(author = "KNTH")]
#[command(version = "0.1.0")]
#[command(about = "Automagically sort photos for you!", long_about = None)]
pub struct Cli {
    input: String,
    output: String,
}

#[instrument]
fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let cli = Cli::parse();
    let ext_list = [
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico", "heic", "webp", "svg", "raw", "mp4",
        "mov", "avi", "3gp", "mkv", "flv", "wmv", "mpeg", "webm",
    ];

    let mut non_media_paths: Vec<String> = vec![];

    match fs::read_dir(cli.input) {
        Ok(paths) => {
            let mut count_media = 0;
            let mut count_all = 0;
            for path in paths {
                match path {
                    Ok(dir_entry) => {
                        let path = dir_entry.path();
                        let ext = path.extension().and_then(std::ffi::OsStr::to_str);
                        let is_media = ext
                            .map(|e| ext_list.contains(&e.to_lowercase().as_str()))
                            .unwrap_or(false);

                        let path_display = path.display();
                        if is_media {
                            debug!("Media file: {}", path_display);
                            count_media += 1;

                            // TODO: use EXIF metadata instead for photos
                            let metadata = fs::metadata(&path)?;
                            let creation_time = metadata.created()?;
                            let datetime: DateTime<Local> = DateTime::from(creation_time);
                            let format_year = datetime.format("%Y").to_string();
                            let format_month = datetime.format("%m").to_string();
                            let format_date = datetime.format("%Y-%m-%d %T").to_string();
                            debug!("creation time: {format_date}");

                            let target_path =
                                make_dir(cli.output.clone(), format_year, format_month)?;

                            let dest_media_file_name = path.file_name().unwrap();
                            let mut dest_media_path = PathBuf::from(target_path);
                            dest_media_path.push(dest_media_file_name);

                            fs::copy(&path, &dest_media_path)?;
                        } else {
                            non_media_paths.push(path_display.to_string());
                        }
                    }
                    Err(e) => tracing::error!("Error reading directory: {}", e),
                }
                count_all += 1;
            }

            info!(
                "Successfully parsed the input directory. There are {} files, in which {} are detected media",
                count_all, count_media
            );

            if count_all > count_media {
                for path in non_media_paths {
                    warn!("Non media files: {}", path);
                }
            }

            Ok(())
        }
        Err(e) => {
            error!("Error listing directory: {}", e);
            Err(e)
        }
    }
}

fn make_dir(output_dir: String, year: String, month: String) -> io::Result<String> {
    let mut output_dir = std::path::PathBuf::from(output_dir);
    output_dir.push("");
    let output_dir = output_dir.to_string_lossy().into_owned();
    let target_path = format!("{output_dir}/{year}/{year}-{month}");
    fs::create_dir_all(&target_path)?;

    Ok(target_path)
}
