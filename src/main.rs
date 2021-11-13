use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug)]
struct CustomError(String);

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug, StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: PathBuf,
    /// Dry run
    #[structopt(short, long)]
    dryrun: bool,
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    println!("Debug: params = {:?}", args);

    let content = grant::get_content(&args.path)?;
    println!("file content: {}", content);

    Ok(())
}
