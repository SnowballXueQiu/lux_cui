use std::process::{Command, exit};
use std::string::String;
use std::fmt;
use inquire::{Confirm, Select, Text};
use tokio::sync::mpsc::{self, Sender, Receiver};
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
        .expect("URL or BV Code is required");

    let output = Command::new("lux")
        .arg("-j")
        .arg(url.clone())
        .output()
        .expect("Failed to execute command");

    let info: Vec<UrlInfo> = match serde_json::from_slice(&output.stdout) {
        Ok(info) => info,
        Err(_) => {
            println!("Error getting info");
            println!("{}", String::from_utf8_lossy(&output.stdout));
            exit(1);
        }
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
        videos.push((selected, av.title.clone()));
    }

    let thread_count = Text::new("Thread count:")
        .with_default("4")
        .prompt()
        .unwrap();

    let confirmation = Confirm::new(format!("Download {} videos with a total size of {} MB? (y or n)", videos.len(), total_size / 800 / 1000).as_str())
        .prompt()
        .unwrap();

    if !confirmation {
        println!("Download cancelled");
        exit(0);
    }

    let (log_tx, log_rx) = mpsc::channel::<String>(100);

    let download_handle = tokio::spawn(download_videos(videos, url.clone(), thread_count, log_tx.clone()));
    let log_handle = tokio::spawn(log_thread(log_rx));

    if let Err(e) = download_handle.await {
        eprintln!("Error in download handle: {:?}", e);
    }

    if let Err(e) = log_handle.await {
        eprintln!("Error in log handle: {:?}", e);
    }
}

async fn download_videos(videos: Vec<(StreamInfo, String)>, url: String, thread_count: String, log_tx: Sender<String>) {
    for (index, (video, title)) in videos.iter().enumerate() {
        let message = format!("Downloading {} of {}: \"{}\"", index + 1, videos.len(), title);
        if let Err(e) = log_tx.send(message).await {
            eprintln!("Error sending log message: {:?}", e);
            return;
        }

        let output = Command::new("lux")
            .arg("-f")
            .arg(video.id.clone())
            .arg("-n")
            .arg(thread_count.clone())
            .arg("-items")
            .arg((index + 1).to_string())
            .arg("-p")
            .arg(url.clone())
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error_message = format!(
                        "Error during download of \"{}\": {}",
                        title,
                        String::from_utf8_lossy(&output.stderr)
                    );
                    if let Err(e) = log_tx.send(error_message).await {
                        eprintln!("Error sending log message: {:?}", e);
                    }
                } else {
                    let success_message = format!("Download completed for video {}: \"{}\"", index + 1, title);
                    if let Err(e) = log_tx.send(success_message).await {
                        eprintln!("Error sending log message: {:?}", e);
                    }
                }
            }
            Err(e) => {
                let error_message = format!("Failed to execute command for \"{}\": {:?}", title, e);
                if let Err(e) = log_tx.send(error_message).await {
                    eprintln!("Error sending log message: {:?}", e);
                }
            }
        }
    }

    // Send a message indicating that all downloads are complete
    if let Err(e) = log_tx.send("ALL_DOWNLOADS_COMPLETE".to_string()).await {
        eprintln!("Error sending completion message: {:?}", e);
    }
}

async fn log_thread(mut log_rx: Receiver<String>) {
    while let Some(message) = log_rx.recv().await {
        if message == "ALL_DOWNLOADS_COMPLETE" {
            println!(
                "\
                ----------------------------------------\n\
                All downloads completed\n\
                ----------------------------------------\n\
                Exiting...\
                "
            );
            break;
        }
        println!("{}", message);
    }
}
