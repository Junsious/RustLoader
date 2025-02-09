use async_process::Command;
use inquire::{Select, Text, Confirm};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command as SyncCommand;
use regex::Regex;
use reqwest;

// URLs
const YT_DLP_FILENAME: &str = "yt-dlp.exe";
const YT_DLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
const FFMPEG_DOWNLOAD_URL: &str = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎥 Welcome to the YouTube Video Downloader \n");

    // Check yt-dlp
    if check_yt_dlp().is_err() {
        println!("⚠ yt-dlp not found. Installing...");
        install_yt_dlp()?;
    }

    // Check FFmpeg
    if check_ffmpeg().is_err() {
        println!("⚠ FFmpeg not found. Installing...");
        install_ffmpeg()?;
    }

    loop {
        // Get the video URL
        let url = loop {
            let input = Text::new("🔗 Enter video URL:")
                .prompt()?
                .trim()
                .to_string();

            if let Some(valid_url) = clean_youtube_url(&input) {
                break valid_url;
            } else {
                println!("❌ Error: Invalid URL. Please try again!");
            }
        };

        // Path to save video
        let save_path = loop {
            let path = Text::new("📁 Enter the folder to save the video:")
                .prompt()?
                .trim()
                .to_string();

            if Path::new(&path).is_dir() {
                break path;
            } else {
                println!("❌ Error: Folder does not exist. Please try again.");
            }
        };

        // Choose quality and format
        let qualities = vec!["1080p", "2K", "4K", "<=720p", "<=480p"];
        let quality = Select::new("🎚 Choose quality:", qualities).prompt()?;

        let formats = vec!["mp4", "webm"];
        let video_format = Select::new("🎞 Choose format:", formats).prompt()?;

        // Define the format
        let format = match (quality, video_format.as_ref()) {
            ("1080p", "mp4") => "bv*[height=1080]+ba/b", // Video with default audio
            ("2K", "mp4") => "bv*[height=1440]+ba/b",    // Video with default audio
            ("4K", "mp4") => "bv*[height=2160]+ba/b",    // Video with default audio
            ("<=720p", "mp4") => "bv*[height<=720]+ba/b", // Video with default audio
            ("<=480p", "mp4") => "bv*[height<=480]+ba/b", // Video with default audio
            ("1080p", "webm") => "bv*[height=1080]+ba",   // WebM quality
            ("2K", "webm") => "bv*[height=1440]+ba",      // WebM quality
            ("4K", "webm") => "bv*[height=2160]+ba",      // WebM quality
            ("<=720p", "webm") => "bv*[height<=720]+ba",   // WebM quality
            ("<=480p", "webm") => "bv*[height<=480]+ba",   // WebM quality
            _ => "bestvideo+bestaudio",                    // Default if no match
        };

        // Ensure MP4 uses non-Opus audio (usually AAC)
        let format = if video_format == "mp4" {
            format.replace("ba", "bestaudio[ext=m4a]") // Enforce m4a (AAC)
        } else {
            format.to_string() // WebM formats, no change needed
        };

        // Download video
        println!("⏳ Downloading video...");
        let output_status = Command::new("yt-dlp")
            .arg("-f").arg(format)
            .arg("--merge-output-format").arg(video_format)
            .arg("-o").arg(format!("{}/%(title)s.{}", save_path, video_format))
            .arg(&url)
            .status()
            .await?;

        if output_status.success() {
            println!("✅ Video downloaded successfully!");
        } else {
            println!("❌ Error while downloading.");
        }

        // Repeat?
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

// Check yt-dlp
fn check_yt_dlp() -> Result<(), ()> {
    if SyncCommand::new("yt-dlp").arg("--version").output().is_ok() {
        return Ok(());
    }
    Err(())
}

// Install yt-dlp
fn install_yt_dlp() -> Result<(), Box<dyn std::error::Error>> {
    let target_path = env::var("APPDATA").unwrap_or("C:\\yt-dlp".to_string());
    let exe_path = Path::new(&target_path).join(YT_DLP_FILENAME);

    if !exe_path.exists() {
        println!("🌐 Downloading yt-dlp...");
        let response = reqwest::blocking::get(YT_DLP_DOWNLOAD_URL)?;
        let mut file = fs::File::create(&exe_path)?;
        io::copy(&mut response.bytes()?.as_ref(), &mut file)?;
    }
    env::set_var("PATH", format!("{};{}", exe_path.parent().unwrap().display(), env::var("PATH").unwrap()));
    println!("✅ yt-dlp installed!");
    Ok(())
}

// Check FFmpeg
fn check_ffmpeg() -> Result<(), ()> {
    if SyncCommand::new("ffmpeg").arg("-version").output().is_ok() {
        return Ok(());
    }
    Err(())
}

// Install FFmpeg
fn install_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    let target_path = env::var("APPDATA").unwrap_or("C:\\ffmpeg".to_string());
    let ffmpeg_folder = Path::new(&target_path).join("ffmpeg");
    let ffmpeg_exe = ffmpeg_folder.join("bin").join("ffmpeg.exe");

    if !ffmpeg_exe.exists() {
        println!("🌐 Downloading FFmpeg...");
        let response = reqwest::blocking::get(FFMPEG_DOWNLOAD_URL)?;
        let archive_path = Path::new(&target_path).join("ffmpeg.zip");
        let mut file = fs::File::create(&archive_path)?;
        io::copy(&mut response.bytes()?.as_ref(), &mut file)?;

        println!("📦 Extracting FFmpeg...");
        let zip_file = fs::File::open(&archive_path)?;
        zip_extract::extract(zip_file, &ffmpeg_folder, true)?;
        fs::remove_file(&archive_path)?;
    }
    env::set_var("PATH", format!("{};{}", ffmpeg_folder.join("bin").display(), env::var("PATH").unwrap()));
    println!("✅ FFmpeg installed!");
    Ok(())
}

// Clean YouTube URL
fn clean_youtube_url(url: &str) -> Option<String> {
    let re = Regex::new(r"^(https?://)?(www\.)?(youtube\.com/watch\?v=[\w-]+|youtu\.be/[\w-]+)").unwrap();
    re.find(url).map(|m| m.as_str().to_string())
}
