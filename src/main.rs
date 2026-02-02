use std::{any, path::PathBuf};

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

fn convert(input: PathBuf, output: PathBuf) {
    println!("input: {:?}, output: {:?}", input, output);
    let result = wav_decode(&input, &output).unwrap();
    print!("{}", result);
}

fn main() {
    let args = Args::parse();
    convert(args.input, args.output)
}
