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
}
