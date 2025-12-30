use chrono::Local;
use color_eyre::Result;
use color_eyre::eyre::{Ok, ensure};
use encoding_rs::SHIFT_JIS;
use mp4::Mp4Reader;
use std::env::args;
use std::fs::File;
use std::io::{BufReader, Write};

const HEADER: &str = &include_str!("header.txt");
const FPS: u32 = 30;

fn format_entry(i: usize, offset: u128, filepath: &str) -> Result<(String, u128)> {
    let f = File::open(filepath)?;
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mp4 = Mp4Reader::read_header(reader, size)?;
    let length = mp4.duration().as_millis() * (FPS as u128) / 1000;

    let group_id = i + 1;
    let video_id = 2 * i + 0;
    let audio_id = 2 * i + 1;
    let start = offset + 1;
    let end = offset + length;

    Ok((
        format!(
            include_str!("entry.txt"),
            arg = filepath,
            group_id = group_id,
            video_id = video_id,
            audio_id = audio_id,
            start = start,
            end = end,
        ),
        length,
    ))
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let now = Local::now();
    let filepath = format!("aviutl-drop-rs-{}.exo", now.format("%Y%m%d-%H%M%S"));

    let mut buffer = String::new();

    buffer.push_str(HEADER);

    let mut offset = 0;

    let filepaths: Vec<_> = args().skip(1).collect();

    ensure!(!filepaths.is_empty());

    for (i, filepath) in filepaths.iter().enumerate() {
        let (string, length) = format_entry(i, offset, &filepath)?;
        buffer.push_str(&string);
        offset += length;
    }

    let (cow, _encoding_used, _had_errors) = SHIFT_JIS.encode(&buffer);

    let mut file = File::create(filepath)?;
    file.write_all(&cow)?;

    Ok(())
}
