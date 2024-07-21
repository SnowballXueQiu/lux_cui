use std::process::{Command, exit};
use std::string::String;
use std::fmt;

use inquire::{Confirm, Select, Text};

use crate::model::url_info::{StreamInfo, UrlInfo};

mod model;

impl fmt::Display for StreamInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {} | quality: {}", self.id, self.quality)
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let url = Text::new("The video's URL or BV Code:")
        .prompt()
        .expect("URl or BV Code is required");

    let output = Command::new("lux")
        .arg("-j")
        .arg(url.clone())
        .output()
        .expect("Failed to execute command");

    let Ok(info): Result<Vec<UrlInfo>, serde_json::Error> = serde_json::from_slice(&output.stdout) else {
        println!("Error getting info");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        exit(1);
    };

    let mut videos = Vec::new();
    let mut total_size = 0;

    for av in info {
        let mut video_streams: Vec<StreamInfo> = Vec::new();
        for (_, v) in av.streams.as_object().unwrap() {
            let info: StreamInfo = serde_json::from_value(v.clone()).unwrap();
            video_streams.push(info);
        }

        let selected = Select::new(
            format!("Select a stream for: {:?}", av.title).as_str(),
            video_streams.clone(),
        )
            .prompt()
            .unwrap();

        total_size += selected.size;
        videos.push(selected);
    }

    let thread_count = Text::new("Thread count:")
        .with_default("4")
        .prompt()
        .unwrap();

    let confirmation = Confirm::new(format!("Download {} videos with a total size of {} MB?", videos.len(), total_size / 800 / 1000).as_str())
        .prompt()
        .unwrap();

    if !confirmation {
        println!("Download cancelled");
        exit(0);
    }

    for (index, video) in videos.iter().enumerate() {
        println!("Downloading {} of {}", index + 1, videos.len());

        let download = Command::new("lux")
            .arg("-f")
            .arg(video.clone().id)
            .arg("-n")
            .arg(thread_count.clone())
            .arg("-items")
            .arg(index.to_string())
            .arg("-p")
            .arg(url.clone())
            .output()
            .expect("Failed to execute command");
    }
}