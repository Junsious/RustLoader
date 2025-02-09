use async_process::Command;
use inquire::{Select, Text, Confirm};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command as SyncCommand;
use regex::Regex;

// Path to yt-dlp.exe
const YT_DLP_FILENAME: &str = "yt-dlp.exe";
const YT_DLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

// Main asynchronous function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("\n🎥 Welcome to the YouTube Video Downloader \n");

        // Check if yt-dlp is installed
        if check_yt_dlp().is_err() {
            println!("⚠ yt-dlp not found. Installing...");
            install_yt_dlp()?;
        }

        // Enter the URL with validation
        let url = loop {
            let input = Text::new("🔗 Enter the YouTube video URL:")
                .prompt()?
                .trim()
                .to_string();

            if let Some(valid_url) = clean_youtube_url(&input) {
                break valid_url;
            } else {
                println!("❌ Error: Invalid URL. Please try again!");
            }
        };

        // Enter the save path
        let save_path = loop {
            let path = Text::new("📁 Enter the folder to save the video:")
                .prompt()?
                .trim()
                .to_string();

            if Path::new(&path).is_dir() {
                break path;
            } else {
                println!("❌ Error: The specified folder does not exist. Please try again.");
            }
        };

        // Select video quality
        let qualities = vec!["1080p", "2K", "4K", "<=720p", "<=480p"];
        let quality = Select::new("🎚 Choose video quality:", qualities).prompt()?;

        // Select video format
        let formats = vec!["mp4", "webm"];
        let video_format = Select::new("🎞 Choose video format:", formats).prompt()?;

        // Determine the correct format option for yt-dlp
        let format = match quality {
            "1080p" => "bv*[height=1080]+ba/b",
            "2K" => "bv*[height=1440]+ba/b",
            "4K" => "bv*[height=2160]+ba/b",
            "<=720p" => "bv*[height<=720]+ba/b",
            "<=480p" => "bv*[height<=480]+ba/b",
            _ => "bestvideo+bestaudio",
        };

        // Start downloading the video
        println!("⏳ Downloading video...");

        let output_status = Command::new("yt-dlp")
            .arg("-f")
            .arg(format)  // Specify the selected video format
            .arg("--merge-output-format")
            .arg(video_format) // Enforce the chosen output format
            .arg("-o")
            .arg(format!("{}/%(title)s.{}", save_path, video_format)) // Ensure correct file extension
            .arg(&url)
            .status()
            .await?;

        if output_status.success() {
            println!("✅ Video downloaded successfully!");
        } else {
            println!("❌ Error occurred during download.");
        }

        // Ask if the user wants to download another video
        let close = Confirm::new("🔄 Do you want to download another video?")
            .with_default(true)
            .prompt()?;

        if !close {
            println!("👋 Goodbye!");
            break;
        }
    }

    Ok(())
}

// Check if yt-dlp is installed
fn check_yt_dlp() -> Result<(), ()> {
    if SyncCommand::new("yt-dlp").arg("--version").output().is_ok() {
        return Ok(());
    }
    Err(())
}

// Install yt-dlp
fn install_yt_dlp() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_dir()?.join(YT_DLP_FILENAME);
    let appdata_path = env::var("APPDATA").unwrap_or_else(|_| "C:\\yt-dlp".to_string());
    let target_path = Path::new(&appdata_path).join("yt-dlp.exe");

    if !target_path.exists() {
        if exe_path.exists() {
            println!("📂 Found yt-dlp.exe, copying to {}", target_path.display());
            fs::copy(&exe_path, &target_path)?;
        } else {
            println!("🌐 Downloading yt-dlp.exe...");
            let response = reqwest::blocking::get(YT_DLP_DOWNLOAD_URL)?;
            let mut file = fs::File::create(&target_path)?;
            io::copy(&mut response.bytes()?.as_ref(), &mut file)?;
        }
    }

    env::set_var("PATH", format!("{};{}", target_path.parent().unwrap().display(), env::var("PATH").unwrap()));
    println!("✅ yt-dlp installed successfully!");
    Ok(())
}

// Clean and validate YouTube URL
fn clean_youtube_url(url: &str) -> Option<String> {
    let re = Regex::new(r"^(https?:\/\/)?(www\.)?(youtube\.com\/watch\?v=[\w-]+|youtu\.be\/[\w-]+)").unwrap();
    re.find(url).map(|m| m.as_str().to_string())
}
