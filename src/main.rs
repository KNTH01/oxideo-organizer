use std::{fs, io::Error};

use clap::Parser;
use tracing::{error, info, instrument, warn};

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
    tracing_subscriber::fmt::init();

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
                            info!("Media file: {}", path_display);
                            count_media += 1;
                        } else {
                            non_media_paths.push(path_display.to_string());
                        }
                    }
                    Err(e) => tracing::error!("Error reading directory: {}", e),
                }
                count_all += 1;
            }

            info!(
                "Successfully parse the input. There are {} files, in which {} are detected media",
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
