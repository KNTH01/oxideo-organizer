use anyhow::Result;
use chrono::NaiveDateTime;
use indicatif::ProgressBar;
use std::{fs, path::PathBuf};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

pub struct Counter {
    pub all: u32,
    pub media: u32,
    pub processed: u32,
}

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
            counter: Counter {
                all: 0,
                media: 0,
                processed: 0,
            },
            input,
            output,
            list: List {
                non_media_paths: vec![],
            },
        }
    }

    pub fn walk_dir(&mut self, input: &str) -> Result<()> {
        let ext_list = [
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "ico", "heic", "webp", "svg", "raw", "mp4",
            "mov", "avi", "3gp", "mkv", "flv", "wmv", "mpeg", "webm",
        ];

        //     let bar = ProgressBar::new(1000);

        for entry in WalkDir::new(input) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        debug!("{}", path.display());

                        let path_buf = path.to_path_buf();
                        let path_display = path.display();
                        let ext = path.extension().and_then(std::ffi::OsStr::to_str);
                        let is_media = ext
                            .map(|e| ext_list.contains(&e.to_lowercase().as_str()))
                            .unwrap_or(false);

                        if is_media {
                            debug!("Media file: {}", path_display);

                            self.counter.media += 1;

                            match self.read_metadata(path.to_str().unwrap()) {
                                Some(datetime) => {
                                    self.copy_datetime_media(&path_buf, &datetime)?;
                                    self.counter.processed += 1;
                                    // bar.inc(1);
                                }
                                None => {
                                    warn!("Cannot get media DateTimeOriginal");
                                    self.copy_untouched_media(&path_buf)?;
                                    // bar.inc(1);
                                }
                            }
                        } else {
                            self.list.non_media_paths.push(path.to_path_buf());
                            self.copy_untouched_media(&path_buf)?;
                            // bar.inc(1);
                        }
                        self.counter.all += 1;
                    }
                }
                Err(e) => tracing::error!("Error reading directory: {}", e),
            }
        }

        info!("Successfully parsed the input directory. There are {} files, in which {} are detected media. {} got processed.",
        self.counter.all, self.counter.media, self.counter.processed
    );

        if self.counter.all > self.counter.media {
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
