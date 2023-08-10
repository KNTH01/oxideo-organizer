use std::{fs, io::Error};

use clap::Parser;
use tracing::{error, info, instrument};

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
    match fs::read_dir(cli.input) {
        Ok(dir_entries) => {
            let mut i = 0;
            for dir_entry in dir_entries {
                info!("Name: {}", dir_entry.unwrap().path().display());
                i += 1;
            }
            info!("Successfully parse the input. There are {} files", i);
            Ok(())
        }
        Err(e) => {
            error!("Error listing directory: {}", e);
            Err(e)
        }
    }
}
