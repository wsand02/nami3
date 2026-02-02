use mp3lame_encoder::{Builder, DualPcm, Encoder, FlushNoGap, Id3Tag, MonoPcm};

use std::fs::File;
use std::io::{BufWriter, Write};

use std::cmp;
use std::path::PathBuf;

const CHUNK_SIZE: usize = 1152; // https://stackoverflow.com/questions/72416908/mp3-exact-frame-size-calculation
const I32_MAXPONE: f32 = 2147483648.0_f32; // 2^31
const I24_MAXPONE: f32 = 8388608.0_f32; // 2^23
const I16_MAXPONE: f32 = 32768.0_f32; // 2^15

const MONO: usize = 1;
const STEREO: usize = 2;

pub fn wav_decode(input_path: &PathBuf, output_path: &PathBuf) -> Result<String, anyhow::Error> {
    let mut reader = hound::WavReader::open(&input_path).unwrap();
    let channels = reader.spec().channels as usize;
    let bit_depth = reader.spec().bits_per_sample;
    if channels != MONO && channels != STEREO {
        return Err(hound::Error::Unsupported.into());
    }
    let sample_rate = reader.spec().sample_rate;

    let decode_result = match bit_depth {
        16 => process_samples(
            reader.samples::<i16>(),
            channels,
            1.0 / I16_MAXPONE,
            sample_rate,
            output_path,
        )?,
        24 => process_samples(
            reader.samples::<i32>(),
            channels,
            1.0 / I24_MAXPONE,
            sample_rate,
            output_path,
        )?,
        32 => match reader.spec().sample_format {
            hound::SampleFormat::Float => process_samples(
                reader.samples::<f32>(),
                channels,
                1.0,
                sample_rate,
                output_path,
            )?,
            hound::SampleFormat::Int => process_samples(
                reader.samples::<i32>(),
                channels,
                1.0 / I32_MAXPONE,
                sample_rate,
                output_path,
            )?,
        },
        _ => return Err(hound::Error::Unsupported.into()),
    };
    Ok(decode_result)
}

fn placeholder_id3_tag() -> Id3Tag<'static> {
    Id3Tag {
        title: b"title",
        artist: b"artist",
        album_art: &[],
        album: b"album",
        year: b"year",
        comment: b"comment",
    }
}

fn process_samples<T>(
    samples: impl Iterator<Item = hound::Result<T>>,
    channels: usize,
    scale: f32,
    sample_rate: u32,
    output: &PathBuf,
) -> Result<String, anyhow::Error>
where
    f64: From<T>,
{
    let mut mp3_encoder = Builder::new().ok_or_else(|| anyhow::anyhow!("Generic error idk"))?;
    mp3_encoder
        .set_num_channels(channels as u8)
        .expect("Failed to set number of channels on MP3 encoder");
    mp3_encoder
        .set_sample_rate(sample_rate)
        .map_err(|e| anyhow::anyhow!(e))?;
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps128)
        .map_err(|e| anyhow::anyhow!(e))?;
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Decent)
        .map_err(|e| anyhow::anyhow!(e))?;

    mp3_encoder
        .set_id3_tag(placeholder_id3_tag())
        .map_err(|_| anyhow::anyhow!("id no"))?;

    let mut mp3_encoder = mp3_encoder.build().map_err(|e| anyhow::anyhow!(e))?;
    let file = File::create(output)?;
    let mut bwriter = BufWriter::new(file);
    let mut left = Vec::with_capacity(CHUNK_SIZE);
    let mut right = Vec::with_capacity(CHUNK_SIZE);
    let is_stereo = channels == 2;

    for (idx, sample) in samples.enumerate() {
        let sample_val = sample?;
        let srb: f64 = sample_val.into();
        let sr: f32 = srb as f32;
        let s = sr * scale;
        if is_stereo {
            if idx % 2 == 0 {
                left.push(s);
            } else {
                right.push(s);
            }
            if left.len() >= CHUNK_SIZE && right.len() >= CHUNK_SIZE {
                encode_dual(
                    &left[..CHUNK_SIZE],
                    &right[..CHUNK_SIZE],
                    &mut bwriter,
                    &mut mp3_encoder,
                )?;
                left.clear();
                right.clear();
            }
        } else {
            left.push(s);
            if left.len() >= CHUNK_SIZE {
                encode_mono(&left, &mut bwriter, &mut mp3_encoder)?;
                left.clear();
            }
        }
    }
    if is_stereo {
        if !left.is_empty() || !right.is_empty() {
            let max_len = std::cmp::max(left.len(), right.len());
            left.resize(max_len, 0.0);
            right.resize(max_len, 0.0);
            encode_dual(&left, &right, &mut bwriter, &mut mp3_encoder)?;
            left.clear();
            right.clear();
        }
    } else if !left.is_empty() {
        encode_mono(&left, &mut bwriter, &mut mp3_encoder)?;
        left.clear();
    }
    let num_frames = cmp::max(left.len(), right.len());
    let mut tail = Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(num_frames));
    let flushed = mp3_encoder
        .flush_to_vec::<FlushNoGap>(&mut tail)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if flushed > 0 {
        bwriter.write_all(&tail)?;
    }

    bwriter.flush()?;
    Ok(output.to_string_lossy().to_string())
}

fn encode_dual(
    left: &[f32],
    right: &[f32],
    bwriter: &mut BufWriter<File>,
    encoder: &mut Encoder,
) -> Result<(), anyhow::Error> {
    let chunk = DualPcm { left, right };
    let num_frames = cmp::max(left.len(), right.len());
    let mut mp3_out_buffer =
        Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(num_frames));
    let encoded = encoder
        .encode_to_vec(chunk, &mut mp3_out_buffer)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if encoded > 0 {
        bwriter.write_all(&mp3_out_buffer)?;
    }
    Ok(())
}

fn encode_mono(
    left: &[f32],
    bwriter: &mut BufWriter<File>,
    encoder: &mut Encoder,
) -> Result<(), anyhow::Error> {
    let chunk = MonoPcm(left);
    let num_frames = left.len();
    let mut mp3_out_buffer =
        Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(num_frames));
    let encoded: usize = encoder
        .encode_to_vec(chunk, &mut mp3_out_buffer)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    if encoded > 0 {
        bwriter.write_all(&mp3_out_buffer)?;
    }
    Ok(())
}
