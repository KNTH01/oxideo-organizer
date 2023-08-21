use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use organizer::Organizer;
use tracing::{instrument, warn, Level};

pub mod counter;
pub mod organizer;

#[derive(Parser)]
#[command(name = "Oxideo Organizer")]
#[command(author = "KNTH")]
#[command(version = "0.1.0")]
#[command(about = "Automagically sort photos for you!", long_about = None)]
pub struct Cli {
    input: PathBuf,
    output: PathBuf,
}

#[instrument]
fn main() -> Result<()> {
    // tracing_subscriber::fmt()
    //     .with_max_level(Level::INFO)
    //     // .with_max_level(Level::DEBUG)
    //     .init();

    let cli = Cli::parse();

    let input = cli.input.to_str().unwrap();
    let output = cli.output.to_str().unwrap();
    
    let o = Organizer::new(input, output);
    o.walk_dir(input)?;

    Ok(())
}
