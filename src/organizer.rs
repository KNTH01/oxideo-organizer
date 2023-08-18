use crate::counter::{Counter, Counters};
use anyhow::Result;
use chrono::NaiveDateTime;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{fs, path::PathBuf};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

struct List {
    non_media_paths: Vec<PathBuf>,
}

pub struct Organizer<'a> {
    pub counter: Counter,
    pub input: &'a str,
    pub output: &'a str,
    list: List,
}

impl<'a> Organizer<'a> {
    pub fn new(input: &'a str, output: &'a str) -> Self {
        Self {
            counter: Counter::default(),
            input,
            output,
            list: List {
                non_media_paths: vec![],
            },
        }
    }

    pub fn walk_dir(&self, input: &str) -> Result<()> {
        let ext_list = [
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico", "heic", "webp", "svg", "raw", "mp4",
            "mov", "avi", "3gp", "mkv", "flv", "wmv", "mpeg", "webm",
        ];

        let v1 = WalkDir::new(input)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let progress = ProgressBar::new(v1.len() as u64);

        progress.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap(),
        );

        v1.par_iter().try_for_each(|entry| -> Result<()> {
            let path = entry.path();
            let path_display = path.display();

            progress.set_message(path_display.to_string());

            if path.is_file() {
                debug!("{}", path.display());

                let path_buf = path.to_path_buf();
                let ext = path.extension().and_then(std::ffi::OsStr::to_str);
                let is_media = ext
                    .map(|e| ext_list.contains(&e.to_lowercase().as_str()))
                    .unwrap_or(false);

                if is_media {
                    debug!("Media file: {}", path_display);

                    self.counter.increment(Counters::Media);

                    match self.read_metadata(path.to_str().unwrap()) {
                        Some(datetime) => {
                            self.copy_datetime_media(&path_buf, &datetime)?;
                            self.counter.increment(Counters::Processed);
                        }
                        None => {
                            warn!("Cannot get media DateTimeOriginal");
                            self.copy_untouched_media(&path_buf)?;
                        }
                    }
                } else {
                    // self.list.non_media_paths.push(path.to_path_buf());
                    self.copy_untouched_media(&path_buf)?;
                }
                self.counter.increment(Counters::All);
            }

            progress.inc(1);

            Ok(())
        })?;

        progress.set_message("Done");
        progress.finish();

        info!("Successfully parsed the input directory. There are {} files, in which {} are detected media. {} got processed.",
        self.counter.get(Counters::All), self.counter.get(Counters::Media), self.counter.get(Counters::Processed)
        );

        if self.counter.get(Counters::All) > self.counter.get(Counters::Media) {
            for path in &self.list.non_media_paths {
                warn!("Non media file: {}", path.as_path().display());
            }
        }

        Ok(())
    }

    fn read_metadata(&self, path: &str) -> Option<NaiveDateTime> {
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
                    if let Ok(datetime) =
                        NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
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

    fn copy_datetime_media(&self, path: &PathBuf, datetime: &NaiveDateTime) -> Result<()> {
        let format_year = datetime.format("%Y").to_string();
        let format_month = datetime.format("%m").to_string();
        let format_date = datetime.format("%Y-%m-%d %T").to_string();
        debug!("creation time: {format_date}");

        let mut output_dir = std::path::PathBuf::from(self.output);
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

    fn copy_untouched_media(&self, path: &PathBuf) -> Result<()> {
        let mut dest_media_path = PathBuf::new();

        dest_media_path.push(self.output);
        dest_media_path.push("untouched");
        dest_media_path.push(path);

        let mut dest_dir = dest_media_path.clone();
        dest_dir.pop();

        fs::create_dir_all(dest_dir)?;
        fs::copy(path, &dest_media_path)?;

        Ok(())
    }
}
