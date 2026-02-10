# nami3

A simplified command-line tool for converting WAV files to MP3 format.

This is a lightweight version of [waveemapi3](https://github.com/wsand02/waveemapi3/) focused on providing basic WAV-to-MP3 conversion functionality through a CLI interface.

## Features

- Converts WAV files to MP3 format
- Supports both mono and stereo audio
- Handles various bit depths (16, 24, and 32-bit)
- Interactive overwrite confirmation (can be disabled)

## Installation

Requires the Rust toolchain to be installed.

```bash
git clone https://github.com/wsand02/nami3
cd nami3
cargo install --path .
```

## Usage

### Basic Conversion

```bash
nami3 -i input.wav -o output.mp3
```

### Force Overwrite (Skip Confirmation)

```bash
nami3 -i input.wav -o output.mp3 --force
```

### Command-line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--input` | `-i` | Input WAV file path |
| `--output` | `-o` | Output MP3 file path |
| `--force` | `-f` | Skip overwrite confirmation |

## Supported Formats

### Input (WAV)
- Sample rates: Any supported by the input file
- Channels: Mono or Stereo
- Bit depths: 16, 24, and 32-bit (both signed integer and floating point)

### Output (MP3)
- Bitrate: 128 kbps VBR (configurable in code)
- Quality: Decent (configurable in code:D)
- ID3 tags: Basic placeholder tags included

## Technical Details

The tool uses:
- `hound` for WAV file reading
- `mp3lame_encoder` for MP3 encoding
- `clap` for command-line argument parsing
- `anyhow` for error handling

## Limitations

- Currently supports only WAV input format
- Output quality settings are hardcoded (128 kbps VBR)
- ID3 tags are placeholder values and not customizable via CLI

## Building from Source

```bash
git clone https://github.com/wsand02/nami3
cd nami3
cargo build --release
```

The binary will be available in `target/release/nami3`.
