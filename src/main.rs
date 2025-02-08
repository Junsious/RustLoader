use async_process::Command;
use inquire::{Select, Text, Confirm};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as SyncCommand;
use regex::Regex;

// Path to yt-dlp.exe
const YT_DLP_FILENAME: &str = "yt-dlp.exe";
const YT_DLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

// Main async function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("\n Welcome to the YouTube Video Downloader \n");

        // Check if yt-dlp is installed
        if check_yt_dlp().is_err() {
            println!("⚠ yt-dlp is not found. Installing...");
            install_yt_dlp()?;
        }

        // Prompt for the YouTube URL with auto-cleaning
        let url = loop {
            let input = Text::new("🔗 Enter the YouTube video URL:")
                .prompt()?
                .trim()
                .to_string();

            if let Some(valid_url) = clean_youtube_url(&input) {
                break valid_url;
            } else {
                println!("❌ Error: Invalid URL. Please enter a valid YouTube link!");
            }
        };

        // Prompt for the save path
        let save_path = loop {
            let path = Text::new("📁 Enter the directory to save the video:")
                .prompt()?
                .trim()
                .to_string();

            if Path::new(&path).is_dir() {
                break path;
            } else {
                println!("❌ Error: The specified folder does not exist. Please try again.");
            }
        };

        // Video quality selection
        let qualities = vec![" High", " Medium", " Low"];
        let quality = Select::new("🎚 Select video quality:", qualities)
            .prompt()?;

        let format = match quality {
            " High" => "best",
            " Medium" => "bv*[height<=720]+ba/b",
            " Low" => "bv*[height<=480]+ba/b",
            _ => "best",
        };

        // Start the download process
        println!("⏳ Downloading video...");
        let status = Command::new("yt-dlp")
            .arg("-f")
            .arg(format)
            .arg("-o")
            .arg(format!("{}/%(title)s.%(ext)s", save_path))
            .arg(&url)
            .status()
            .await?;

        if status.success() {
            println!("✅ Video downloaded successfully!");
        } else {
            println!("❌ Error occurred during download.");
        }

        // Ask the user if they want to continue
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

// Checks if yt-dlp is installed
fn check_yt_dlp() -> Result<(), ()> {
    if SyncCommand::new("yt-dlp").arg("--version").output().is_ok() {
        return Ok(());
    }
    Err(())
}

// Installs yt-dlp if not found
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
    println!("✅ yt-dlp successfully installed!");
    Ok(())
}

// Cleans and validates YouTube URLs
fn clean_youtube_url(url: &str) -> Option<String> {
    let re = Regex::new(r"^(https?:\/\/)?(www\.)?(youtube\.com\/watch\?v=[\w-]+|youtu\.be\/[\w-]+)").unwrap();
    re.find(url).map(|m| m.as_str().to_string())
}
