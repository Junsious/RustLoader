use async_process::Command;
use inquire::{Select, Text, Confirm};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command as SyncCommand;
use regex::Regex;
use reqwest;
use zip_extract;

// URLs
const YT_DLP_FILENAME: &str = "yt-dlp.exe";
const YT_DLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
const FFMPEG_DOWNLOAD_URL: &str = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
const VLC_DOWNLOAD_URL: &str = "https://download.videolan.org/vlc/last/win64/vlc-3.0.18-win64.exe";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎥 Welcome to the YouTube Video Downloader \n");

    // Check and suggest installing VLC
    fn check_vlc() -> Result<(), ()> {
        // Try running VLC through PATH
        if SyncCommand::new("vlc").arg("--version").output().is_ok() {
            return Ok(());
        }

        // Check standard VLC paths (Windows)
        let common_paths = [
            "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
            "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
        ];

        for path in &common_paths {
            if Path::new(path).exists() {
                println!("✅ VLC found at path: {}", path);
                return Ok(());
            }
        }

        Err(())
    }

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

        let format = match quality {
            "1080p" => "bv*[height=1080]+ba/b",
            "2K" => "bv*[height=1440]+ba/b",
            "4K" => "bv*[height=2160]+ba/b",
            "<=720p" => "bv*[height<=720]+ba/b",
            "<=480p" => "bv*[height<=480]+ba/b",
            _ => "bestvideo+bestaudio",
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

// Check VLC
fn check_vlc() -> Result<(), ()> {
    if SyncCommand::new("vlc").arg("--version").output().is_ok() {
        return Ok(());
    }
    Err(())
}

// Install VLC
fn install_vlc() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Downloading VLC...");
    let response = reqwest::blocking::get(VLC_DOWNLOAD_URL)?;
    let installer_path = "vlc_installer.exe";
    let mut file = fs::File::create(installer_path)?;
    io::copy(&mut response.bytes()?.as_ref(), &mut file)?;

    println!("⚙️ Installing VLC...");
    let status = SyncCommand::new(installer_path)
        .arg("/S") // Silent install
        .spawn()?
        .wait()?;

    if !status.success() {
        println!("❌ VLC installation failed!");
        return Err("VLC installation failed".into());
    }

    println!("✅ VLC installed! Adding to PATH...");

    // Add VLC to PATH
    let vlc_path = "C:\\Program Files\\VideoLAN\\VLC";
    let path_var = env::var("PATH").unwrap_or_default();
    if !path_var.contains(vlc_path) {
        env::set_var("PATH", format!("{};{}", vlc_path, path_var));
        println!("✅ VLC added to PATH!");
    }

    fs::remove_file(installer_path)?; // Remove installer
    Ok(())
}

// Clean YouTube URL
fn clean_youtube_url(url: &str) -> Option<String> {
    let re = Regex::new(r"^(https?://)?(www\.)?(youtube\.com/watch\?v=[\w-]+|youtu\.be/[\w-]+)").unwrap();
    re.find(url).map(|m| m.as_str().to_string())
}
