use std::path::PathBuf;

use clap::Parser;

use crate::audio::wav_decode;

mod audio;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,
}

fn convert(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    println!("Converting: {:?} -> {:?}", input, output);
    let result = wav_decode(&input, &output)?;
    println!("Successfully converted to: {}", result);
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(e) = convert(args.input, args.output) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
