use std::fs;

use clap::Parser;

#[derive(Parser)]
#[command(name = "Oxideo Organizer")]
#[command(author = "KNTH")]
#[command(version = "0.1.0")]
#[command(about = "Automagically sort photos for you!", long_about = None)]
pub struct Cli {
    input: String,
    output: String,
}

fn main() {
    let cli = Cli::parse();

    println!("Hello, world!!!! {}", cli.input);

    match fs::read_dir(cli.input) {
        Ok(paths) => {
            let mut i = 0;
            for path in paths {
                println!("Name: {}", path.unwrap().path().display());
                i += 1;
            }
            println!("\nSuccessfully parse the input. There are {} files", i);
            // Ok(())
        }
        Err(e) => {
            eprintln!("Error listing directory: {}", e);
            // Err(e)
        }
    }
}
