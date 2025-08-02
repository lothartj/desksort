use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    result::Result,
};
use tauri::State;
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Desktop path not found")]
    DesktopNotFound,
    #[error("Config directory not found")]
    ConfigDirNotFound,
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings(HashMap<String, String>);

#[derive(Default)]
pub struct AppState {
    settings: std::sync::Mutex<Settings>,
}

fn get_config_path() -> Result<PathBuf, Error> {
    let config_dir = dirs::config_dir().ok_or(Error::ConfigDirNotFound)?;
    Ok(config_dir.join("desksort").join("settings.json"))
}

fn get_desktop_path() -> Result<PathBuf, Error> {
    dirs::desktop_dir().ok_or(Error::DesktopNotFound)
}

fn ensure_dir_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn get_file_category(path: &Path) -> Option<&'static str> {
    if path.is_dir() {
        return Some("folders");
    }

    let extension = path.extension()?.to_str()?.to_lowercase();
    let ext = format!(".{}", extension);

    let categories = [
        ("documents", &[".pdf", ".docx", ".doc", ".txt", ".odt", ".rtf"][..]),
        ("spreadsheets", &[".xls", ".xlsx", ".csv", ".ods"][..]),
        ("presentations", &[".pptx", ".odp", ".key"][..]),
        (
            "images",
            &[".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".tiff"][..],
        ),
        (
            "videos",
            &[".mp4", ".mkv", ".avi", ".mov", ".webm", ".flv", ".wmv"][..],
        ),
        ("audio", &[".mp3", ".wav", ".aac", ".ogg", ".flac"][..]),
        ("archives", &[".zip", ".rar", ".7z", ".tar", ".gz", ".tar.gz"][..]),
        ("executables", &[".exe", ".msi", ".sh", ".bat", ".appimage"][..]),
        (
            "code",
            &[
                ".js", ".py", ".rs", ".cpp", ".java", ".html", ".css", ".json", ".ts",
            ][..],
        ),
    ];

    for (category, extensions) in categories {
        if extensions.contains(&ext.as_str()) {
            return Some(category);
        }
    }

    None
}

fn generate_unique_path(target_path: &Path) -> PathBuf {
    let file_stem = target_path.file_stem().unwrap().to_str().unwrap();
    let extension = target_path
        .extension()
        .map(|ext| format!(".{}", ext.to_str().unwrap()))
        .unwrap_or_default();
    let parent = target_path.parent().unwrap();

    let mut counter = 1;
    let mut new_path = target_path.to_path_buf();

    while new_path.exists() {
        new_path = parent.join(format!("{}_{}{}", file_stem, counter, extension));
        counter += 1;
    }

    new_path
}

fn save_settings_to_disk(settings: &Settings) -> Result<(), Error> {
    let config_path = get_config_path()?;
    ensure_dir_exists(config_path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(config_path, content)?;
    Ok(())
}

#[derive(Serialize)]
pub struct SortResult {
    moved_files: Vec<String>,
    errors: Vec<String>,
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub async fn load_settings(state: State<'_, AppState>) -> Result<Settings, Error> {
        let config_path = get_config_path()?;
        let mut settings = state.settings.lock().unwrap();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            *settings = serde_json::from_str(&content)?;
        } else {
            let desktop = get_desktop_path()?;
            let sorted_dir = desktop.join("Sorted");

            settings.0.insert(
                "documents".to_string(),
                sorted_dir.join("Documents").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "spreadsheets".to_string(),
                sorted_dir.join("Spreadsheets").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "presentations".to_string(),
                sorted_dir.join("Presentations").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "images".to_string(),
                sorted_dir.join("Images").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "videos".to_string(),
                sorted_dir.join("Videos").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "audio".to_string(),
                sorted_dir.join("Audio").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "archives".to_string(),
                sorted_dir.join("Archives").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "executables".to_string(),
                sorted_dir.join("Executables").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "code".to_string(),
                sorted_dir.join("Code").to_str().unwrap().to_string(),
            );
            settings.0.insert(
                "folders".to_string(),
                sorted_dir.join("Folders").to_str().unwrap().to_string(),
            );
            save_settings_to_disk(&settings)?;
        }

        Ok(settings.clone())
    }

    #[tauri::command]
    pub async fn save_settings(
        settings: Settings,
        state: State<'_, AppState>,
    ) -> Result<(), Error> {
        *state.settings.lock().unwrap() = settings.clone();
        save_settings_to_disk(&settings)
    }

    #[tauri::command]
    pub async fn scan_and_sort(state: State<'_, AppState>) -> Result<SortResult, Error> {
        let settings = state.settings.lock().unwrap();
        let desktop_path = get_desktop_path()?;
        let mut result = SortResult {
            moved_files: Vec::new(),
            errors: Vec::new(),
        };

        for entry in WalkDir::new(&desktop_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| {
                !e.file_name()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            })
        {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to read entry: {}", e));
                    continue;
                }
            };

            let path = entry.path();
            if let Some(category) = get_file_category(path) {
                if let Some(target_dir) = settings.0.get(category) {
                    let target_dir = PathBuf::from(target_dir);
                    ensure_dir_exists(&target_dir)
                        .with_context(|| {
                            format!(
                                "Failed to create target directory: {}",
                                target_dir.display()
                            )
                        })
                        .map_err(|e| {
                            result.errors.push(e.to_string());
                            return;
                        })
                        .ok();

                    let file_name = path.file_name().unwrap();
                    let target_path = target_dir.join(file_name);
                    let final_path = generate_unique_path(&target_path);

                    match fs::rename(path, &final_path) {
                        Ok(_) => result.moved_files.push(format!(
                            "Moved {} to {}",
                            path.display(),
                            final_path.display()
                        )),
                        Err(e) => result.errors.push(format!(
                            "Failed to move {}: {}",
                            path.display(),
                            e
                        )),
                    }
                }
            }
        }

        Ok(result)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::scan_and_sort,
            commands::load_settings,
            commands::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
