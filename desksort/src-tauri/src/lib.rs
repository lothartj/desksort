use anyhow::Context;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    result::Result,
    sync::Mutex,
};
use tauri::State;
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
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
pub struct PathMapping {
    extension: String,
    target_path: String,
}

pub struct AppState {
    db: Mutex<Connection>,
}

fn init_db(conn: &Connection) -> Result<(), Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS path_mappings (
            extension TEXT PRIMARY KEY,
            target_path TEXT NOT NULL
        )",
        [],
    )?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM path_mappings",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        println!("Initializing default paths...");
        let desktop = get_desktop_path()?;
        let sorted_dir = desktop.join("Sorted");

        let default_paths = [
            (".pdf", sorted_dir.join("Documents")),
            (".docx", sorted_dir.join("Documents")),
            (".doc", sorted_dir.join("Documents")),
            (".txt", sorted_dir.join("Documents")),
            (".odt", sorted_dir.join("Documents")),
            (".rtf", sorted_dir.join("Documents")),
            (".xls", sorted_dir.join("Spreadsheets")),
            (".xlsx", sorted_dir.join("Spreadsheets")),
            (".csv", sorted_dir.join("Spreadsheets")),
            (".ods", sorted_dir.join("Spreadsheets")),
            (".pptx", sorted_dir.join("Presentations")),
            (".odp", sorted_dir.join("Presentations")),
            (".key", sorted_dir.join("Presentations")),
            (".jpg", sorted_dir.join("Images")),
            (".jpeg", sorted_dir.join("Images")),
            (".png", sorted_dir.join("Images")),
            (".gif", sorted_dir.join("Images")),
            (".bmp", sorted_dir.join("Images")),
            (".webp", sorted_dir.join("Images")),
            (".tiff", sorted_dir.join("Images")),
            (".mp4", sorted_dir.join("Videos")),
            (".mkv", sorted_dir.join("Videos")),
            (".avi", sorted_dir.join("Videos")),
            (".mov", sorted_dir.join("Videos")),
            (".webm", sorted_dir.join("Videos")),
            (".flv", sorted_dir.join("Videos")),
            (".wmv", sorted_dir.join("Videos")),
            (".mp3", sorted_dir.join("Audio")),
            (".wav", sorted_dir.join("Audio")),
            (".aac", sorted_dir.join("Audio")),
            (".ogg", sorted_dir.join("Audio")),
            (".flac", sorted_dir.join("Audio")),
            (".zip", sorted_dir.join("Archives")),
            (".rar", sorted_dir.join("Archives")),
            (".7z", sorted_dir.join("Archives")),
            (".tar", sorted_dir.join("Archives")),
            (".gz", sorted_dir.join("Archives")),
            (".tar.gz", sorted_dir.join("Archives")),
            (".exe", sorted_dir.join("Executables")),
            (".msi", sorted_dir.join("Executables")),
            (".sh", sorted_dir.join("Executables")),
            (".bat", sorted_dir.join("Executables")),
            (".AppImage", sorted_dir.join("Executables")),
            (".js", sorted_dir.join("Code")),
            (".py", sorted_dir.join("Code")),
            (".rs", sorted_dir.join("Code")),
            (".cpp", sorted_dir.join("Code")),
            (".java", sorted_dir.join("Code")),
            (".html", sorted_dir.join("Code")),
            (".css", sorted_dir.join("Code")),
            (".json", sorted_dir.join("Code")),
            (".ts", sorted_dir.join("Code")),
            ("folder", sorted_dir.join("Folders")),
        ];

        let tx = conn.transaction()?;
        for (ext, path) in default_paths.iter() {
            tx.execute(
                "INSERT OR IGNORE INTO path_mappings (extension, target_path) VALUES (?, ?)",
                params![ext, path.to_str().unwrap()],
            )?;
        }
        tx.commit()?;
        println!("Default paths initialized");
    }

    Ok(())
}

fn get_db_path() -> Result<PathBuf, Error> {
    let config_dir = dirs::config_dir().ok_or(Error::ConfigDirNotFound)?;
    let db_dir = config_dir.join("desksort");
    fs::create_dir_all(&db_dir)?;
    Ok(db_dir.join("settings.db"))
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

pub mod commands {
    use super::*;

    #[tauri::command]
    pub async fn get_path_mapping(extension: String, state: State<'_, AppState>) -> Result<Option<String>, Error> {
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT target_path FROM path_mappings WHERE extension = ?")?;
        let mut rows = stmt.query(params![extension])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    #[tauri::command]
    pub async fn set_path_mapping(extension: String, target_path: String, state: State<'_, AppState>) -> Result<(), Error> {
        println!("Setting path mapping: {} -> {}", extension, target_path);
        let conn = state.db.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO path_mappings (extension, target_path) VALUES (?, ?)",
            params![extension, target_path],
        )?;
        Ok(())
    }

    #[tauri::command]
    pub async fn get_all_mappings(state: State<'_, AppState>) -> Result<Vec<PathMapping>, Error> {
        println!("Getting all mappings...");
        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT extension, target_path FROM path_mappings")?;
        let mappings = stmt.query_map([], |row| {
            Ok(PathMapping {
                extension: row.get(0)?,
                target_path: row.get(1)?,
            })
        })?;

        let mut result = Vec::new();
        for mapping in mappings {
            result.push(mapping?);
        }
        println!("Found {} mappings", result.len());
        Ok(result)
    }

    #[tauri::command]
    pub async fn scan_and_sort(state: State<'_, AppState>) -> Result<SortResult, Error> {
        let desktop_path = get_desktop_path()?;
        let mut result = SortResult {
            moved_files: Vec::new(),
            errors: Vec::new(),
        };

        let conn = state.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT target_path FROM path_mappings WHERE extension = ?")?;

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
                    result.errors.push(format!("Failed to read entry: {}", e));
                    continue;
                }
            };

            let path = entry.path();
            let extension = if path.is_dir() {
                String::from("folder")
            } else {
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| format!(".{}", e.to_lowercase()))
                    .unwrap_or_default()
            };

            let mut rows = stmt.query(params![extension])?;
            if let Some(row) = rows.next()? {
                let target_dir: String = row.get(0)?;
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
                let mut counter = 1;
                let mut final_path = target_path.clone();

                while final_path.exists() {
                    let file_stem = target_path.file_stem().unwrap().to_str().unwrap();
                    let extension = target_path
                        .extension()
                        .map(|ext| format!(".{}", ext.to_str().unwrap()))
                        .unwrap_or_default();
                    final_path = target_dir.join(format!("{}_{}{}", file_stem, counter, extension));
                    counter += 1;
                }

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

        Ok(result)
    }
}

#[derive(Serialize)]
pub struct SortResult {
    moved_files: Vec<String>,
    errors: Vec<String>,
}

pub fn run() {
    let db_path = get_db_path().expect("Failed to get database path");
    let conn = Connection::open(db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(AppState {
            db: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_and_sort,
            commands::get_path_mapping,
            commands::set_path_mapping,
            commands::get_all_mappings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
