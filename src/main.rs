use async_process::Command;
use inquire::{Select, Text};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as SyncCommand;

// Path to yt-dlp.exe in the program folder
const YT_DLP_FILENAME: &str = "yt-dlp.exe";
const YT_DLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if yt-dlp is installed
    if check_yt_dlp().is_err() {
        println!("⚠ yt-dlp not found in the system.");
        install_yt_dlp()?; // Install yt-dlp if it's not found
    }

    // Prompt for the video URL
    let url = Text::new("Enter the video URL:").prompt()?;
    if !url.starts_with("https://www.youtube.com/watch") {
        println!("❌ Error: Invalid YouTube URL.");
        return Ok(());
    }

    // Prompt for the save path
    let save_path = Text::new("Enter the save path:").prompt()?;
    if !fs::metadata(&save_path).is_ok() {
        println!("❌ Error: The specified folder does not exist.");
        return Ok(());
    }

    // Select video quality
    let qualities = vec!["Best quality", "Medium quality", "Low quality"];
    let quality = Select::new("Select video quality:", qualities).prompt()?;

    let format = match quality {
        "Best quality" => "best",
        "Medium quality" => "bv*[height<=720]+ba/b",
        "Low quality" => "bv*[height<=480]+ba/b",
        _ => "best",
    };

    // Download the video
    println!("⏳ Downloading video...");
    let status = Command::new("yt-dlp")
        .arg("-f")
        .arg(format)
        .arg("-o")
        .arg(format!("{}/%(title)s.%(ext)s", save_path))
        .arg(url)
        .status()
        .await?;

    if status.success() {
        println!("✅ Video successfully downloaded!");
    } else {
        println!("❌ Error while downloading the video.");
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

// Installs yt-dlp if it's not present
fn install_yt_dlp() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_dir()?.join(YT_DLP_FILENAME);
    let appdata_path = env::var("APPDATA").unwrap_or_else(|_| "C:\\yt-dlp".to_string());
    let target_path = Path::new(&appdata_path).join("yt-dlp.exe");

    if !target_path.exists() {
        if exe_path.exists() {
            // Copy yt-dlp.exe from the current folder
            println!("📂 Found yt-dlp.exe, copying to {}", target_path.display());
            fs::copy(&exe_path, &target_path)?;
        } else {
            // Download yt-dlp.exe
            println!("🌐 Downloading yt-dlp.exe...");
            let response = reqwest::blocking::get(YT_DLP_DOWNLOAD_URL)?;
            let mut file = fs::File::create(&target_path)?;
            io::copy(&mut response.bytes()?.as_ref(), &mut file)?;
        }
    }

    // Temporarily add the path to PATH
    env::set_var("PATH", format!("{};{}", target_path.parent().unwrap().display(), env::var("PATH").unwrap()));

    println!("✅ yt-dlp successfully installed!");
    Ok(())
}
