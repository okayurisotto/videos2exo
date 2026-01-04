use std::fs::File;
use std::io::Write;

use chrono::Local;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::Ok;
use encoding_rs::SHIFT_JIS;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(num_args = 2..)]
    files: Vec<String>,

    #[arg(short, long, default_value_t = 30)]
    fps: u32,
}

struct Entry<'a> {
    filepath: &'a String,
    group_id: usize,
    video_id: usize,
    audio_id: usize,
    length: u128,
}

impl Entry<'_> {
    fn to_string_video(&self, offset: u128) -> String {
        format!(
            include_str!("video.txt"),
            filepath = self.filepath,
            group_id = self.group_id,
            object_id = self.video_id,
            start = offset,
            end = offset + self.length,
        )
    }
    fn to_string_audio(&self, offset: u128) -> String {
        format!(
            include_str!("audio.txt"),
            filepath = self.filepath,
            group_id = self.group_id,
            object_id = self.audio_id,
            start = offset,
            end = offset + self.length,
        )
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let now = Local::now();
    let filepath = format!("aviutl-drop-rs-{}.exo", now.format("%Y%m%d-%H%M%S"));

    let mut entries: Vec<Entry> = Vec::new();

    for (i, filepath) in args.files.iter().enumerate() {
        let file = File::open(filepath)?;
        let mp4reader = mp4::read_mp4(file)?;
        let length = mp4reader.duration().as_millis() * 30 / 1000;

        println!("{}: {:?}", filepath, length);

        entries.push(Entry {
            filepath: filepath,
            group_id: i + 1,
            video_id: i,
            audio_id: args.files.len() + i,
            length,
        });
    }

    let mut buffer: String = String::from(format!(
        include_str!("header.txt"),
        length = entries.iter().map(|entry| entry.length).sum::<u128>() + entries.len() as u128
    ));

    let mut offset = 1;
    for entry in &entries {
        buffer.push_str(&entry.to_string_video(offset));
        buffer.push_str(&entry.to_string_audio(offset));
        offset += entry.length + 1;
    }

    let (cow, _encoding_used, _had_errors) = SHIFT_JIS.encode(&buffer);

    let mut file = File::create(filepath)?;
    file.write_all(&cow)?;

    Ok(())
}
