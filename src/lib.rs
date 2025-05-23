use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

fn is_movie_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let ext_lower = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("")
        .to_lowercase();
    ext_lower == "mkv" || ext_lower == "mp4"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallpaper {
    pub file: PathBuf,
    #[serde(rename = "preview")]
    pub preview_img: PathBuf,
    pub title: String,
    #[serde(rename = "type")]
    pub wallpaper_type: String,
    pub description: Option<String>,
}

impl Wallpaper {
    pub fn build(dir_path: &Path) -> Option<Wallpaper> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return None;
        }

        let entries = fs::read_dir(dir_path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name == "project.json" {
                let file_content =
                    fs::read_to_string(path.clone()).expect("LogRocket: error reading file");
                let mut wallpaper: Wallpaper = serde_json::from_str(&file_content).unwrap();

                let parent = path.parent()?;
                let new_file_path = PathBuf::new().join(parent);
                wallpaper.file = new_file_path.join(wallpaper.file);
                if is_movie_file(&wallpaper.file) {
                    let new_preview_path = PathBuf::new().join(parent);
                    wallpaper.preview_img = new_preview_path.join(wallpaper.preview_img);

                    return Some(wallpaper);
                } else {
                    println!(
                        "{} is not video, now only support video file wallpaper!",
                        wallpaper.file.display()
                    );
                    break;
                }
            }
        }

        None
    }
}

pub fn load_wallpaper(dir_path: &Path) -> Result<Vec<Wallpaper>> {
    if !dir_path.exists() {
        return Err(anyhow!("This path does not exist"));
    }

    if !dir_path.is_dir() {
        return Err(anyhow!("This path is not a directory"));
    }

    let mut wallpapers: Vec<Wallpaper> = Vec::new();

    let entries = fs::read_dir(dir_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(wallpaper) = Wallpaper::build(&path) {
            wallpapers.push(wallpaper);
        }
    }

    Ok(wallpapers)
}

pub fn play_playlist(playlist_dir: &Path, mutep: bool) {
    let mute_args = if mutep { "no-audio" } else { "" };

    let other_args = format!("-o '{} --loop-playlist shuffle'", mute_args);

    let command = format!("mpvpaper '*' {} '{}'", other_args, playlist_dir.display());
    println!("command: {}", command);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("执行异常，提示");
    let res = String::from_utf8(output.stdout).unwrap();
    println!("run: {}", res);
}

pub fn generate_wallpapers(wallpapers: &Vec<Wallpaper>, output_path: &Path) -> Result<()> {
    // 创建目录保存临时视频链接
    let save_path = output_path;

    if save_path.exists() {
        fs::remove_dir_all(save_path)?;
    }
    fs::create_dir_all(save_path)?;

    for wallpaper in wallpapers {
        let file_name = wallpaper.file.file_name().unwrap().to_str().unwrap();
        let new_file_path = save_path.join(Path::new(file_name));
        std::os::unix::fs::symlink(wallpaper.file.clone(), new_file_path)?;
    }

    Ok(())
}

pub fn play_wallpapers(load_path: &Path, wallpapers: &Vec<Wallpaper>, mutep: bool) -> Result<()> {
    // 关闭其他mpv
    let command = format!("pkill mpvpaper");
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("执行异常，提示");
    let res = String::from_utf8(output.stdout).unwrap();
    println!("kill other mpvpaper: {}", res);

    // 创建目录保存临时视频链接
    if let Some(save_path) = load_path.parent() {
        let save_path = save_path.join(Path::new("wallpaper_temp"));

        if save_path.exists() {
            fs::remove_dir_all(save_path.clone())?;
        }
        fs::create_dir_all(save_path.clone())?;

        for wallpaper in wallpapers {
            let file_name = wallpaper.file.file_name().unwrap().to_str().unwrap();
            let new_file_path = save_path.clone().join(Path::new(file_name));
            std::os::unix::fs::symlink(wallpaper.file.clone(), new_file_path)?;
        }

        play_playlist(&save_path, mutep);
    }

    Ok(())
}
