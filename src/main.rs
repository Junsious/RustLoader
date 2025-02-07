use async_process::Command;
use inquire::{Select, Text};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as SyncCommand;

// Путь к yt-dlp.exe в папке с программой
const YT_DLP_FILENAME: &str = "yt-dlp.exe";
const YT_DLP_DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Проверяем, установлен ли yt-dlp
    if check_yt_dlp().is_err() {
        println!("⚠ yt-dlp не найден в системе.");
        install_yt_dlp()?;
    }

    // Запрашиваем ссылку на видео
    let url = Text::new("Введите ссылку на видео:").prompt()?;
    if !url.starts_with("https://www.youtube.com/watch") {
        println!("❌ Ошибка: Некорректная ссылка на YouTube.");
        return Ok(());
    }

    // Запрашиваем путь для сохранения
    let save_path = Text::new("Введите путь для сохранения файла:").prompt()?;
    if !fs::metadata(&save_path).is_ok() {
        println!("❌ Ошибка: Указанная папка не существует.");
        return Ok(());
    }

    // Выбор качества видео
    let qualities = vec!["Лучшее качество", "Среднее качество", "Низкое качество"];
    let quality = Select::new("Выберите качество видео:", qualities).prompt()?;

    let format = match quality {
        "Лучшее качество" => "best",
        "Среднее качество" => "bv*[height<=720]+ba/b",
        "Низкое качество" => "bv*[height<=480]+ba/b",
        _ => "best",
    };

    // Загружаем видео
    println!("⏳ Загружаем видео...");
    let status = Command::new("yt-dlp")
        .arg("-f")
        .arg(format)
        .arg("-o")
        .arg(format!("{}/%(title)s.%(ext)s", save_path))
        .arg(url)
        .status()
        .await?;

    if status.success() {
        println!("✅ Видео успешно загружено!");
    } else {
        println!("❌ Ошибка при загрузке видео.");
    }

    Ok(())
}

// Проверяет, установлен ли yt-dlp в системе
fn check_yt_dlp() -> Result<(), ()> {
    if SyncCommand::new("yt-dlp").arg("--version").output().is_ok() {
        return Ok(());
    }
    Err(())
}

// Устанавливает yt-dlp, если его нет
fn install_yt_dlp() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = env::current_dir()?.join(YT_DLP_FILENAME);
    let appdata_path = env::var("APPDATA").unwrap_or_else(|_| "C:\\yt-dlp".to_string());
    let target_path = Path::new(&appdata_path).join("yt-dlp.exe");

    if !target_path.exists() {
        if exe_path.exists() {
            // Копируем yt-dlp.exe из текущей папки
            println!("📂 Найден yt-dlp.exe, копируем в {}", target_path.display());
            fs::copy(&exe_path, &target_path)?;
        } else {
            // Скачиваем yt-dlp.exe
            println!("🌐 Скачиваем yt-dlp.exe...");
            let response = reqwest::blocking::get(YT_DLP_DOWNLOAD_URL)?;
            let mut file = fs::File::create(&target_path)?;
            io::copy(&mut response.bytes()?.as_ref(), &mut file)?;
        }
    }

    // Добавляем путь в PATH (временно)
    env::set_var("PATH", format!("{};{}", target_path.parent().unwrap().display(), env::var("PATH").unwrap()));

    println!("✅ yt-dlp успешно установлен!");
    Ok(())
}
