use std::fs::File;
use std::io::{BufWriter, Write};

use chrono::Local;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::Ok;
use encoding_rs::SHIFT_JIS;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// exoファイルに含めたいmp4ファイル
    #[arg(num_args = 2..)]
    files: Vec<String>,

    /// 編集プロジェクトのフレームレート
    #[arg(short, long, default_value_t = 30)]
    fps: u128,
}

struct Entry<'a> {
    filepath: &'a String,
    group_id: usize,
    video_id: usize,
    audio_id: usize,
    duration: u64,
}

impl Entry<'_> {
    fn to_string_video(&self, offset: u64) -> String {
        format!(
            include_str!("video.txt"),
            filepath = self.filepath,
            group_id = self.group_id,
            object_id = self.video_id,
            start = offset,
            end = offset + self.duration,
        )
    }
    fn to_string_audio(&self, offset: u64) -> String {
        format!(
            include_str!("audio.txt"),
            filepath = self.filepath,
            group_id = self.group_id,
            object_id = self.audio_id,
            start = offset,
            end = offset + self.duration,
        )
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let now = Local::now();
    let exo_path = format!("videos2exo-{}.exo", now.format("%Y%m%d-%H%M%S"));

    let mut entries: Vec<Entry> = Vec::new();

    for (i, video_path) in args.files.iter().enumerate() {
        let video_file = File::open(video_path)?;
        let mp4reader = mp4::read_mp4(video_file)?;
        let duration = (mp4reader.duration().as_millis() * args.fps / 1000) as u64;

        println!("{}: {:?}", video_path, duration);

        entries.push(Entry {
            filepath: video_path,
            group_id: i + 1,
            video_id: i,
            audio_id: args.files.len() + i,
            duration,
        });
    }

    let exo_file = File::create(exo_path)?;
    let mut exo_writer = BufWriter::new(exo_file);

    let header = format!(
        include_str!("header.txt"),
        length = entries.iter().map(|entry| entry.duration).sum::<u64>() + entries.len() as u64
    );
    exo_writer.write_all(header.as_bytes())?;

    let mut offset = 1;
    for entry in &entries {
        let chunk = entry.to_string_video(offset);
        let (encoded, _, _) = &SHIFT_JIS.encode(&chunk);
        exo_writer.write_all(encoded)?;
        offset += entry.duration + 1;
    }

    let mut offset = 1;
    for entry in &entries {
        let chunk = entry.to_string_audio(offset);
        let (encoded, _, _) = &SHIFT_JIS.encode(&chunk);
        exo_writer.write_all(encoded)?;
        offset += entry.duration + 1;
    }

    Ok(())
}
