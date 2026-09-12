use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::domain::config::{Workspace, CONFIG_VERSION};
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::config_file::{parse_config, ConfigFile};

const CONFIG_FILE_NAME: &str = "config.json";

pub fn resolve_config_path(app: &AppHandle) -> AppResult<PathBuf> {
    if cfg!(not(debug_assertions)) {
        let exe_path =
            std::env::current_exe().map_err(|error| AppError::ExePath(error.to_string()))?;
        let dir = exe_path.parent().ok_or(AppError::ExeDir)?;
        Ok(dir.join(CONFIG_FILE_NAME))
    } else {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| AppError::AppDataDir(error.to_string()))?;
        fs::create_dir_all(&dir).map_err(|error| AppError::CreateConfigDir(error.to_string()))?;
        Ok(dir.join(CONFIG_FILE_NAME))
    }
}

pub fn read_workspace(path: &Path) -> AppResult<Option<Workspace>> {
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(path).map_err(|error| AppError::ReadConfig(error.to_string()))?;
    let value: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(_) => {
            quarantine(path)?;
            return Ok(None);
        }
    };
    match parse_config(value) {
        Ok(workspace) => Ok(Some(workspace)),
        Err(_) => {
            quarantine(path)?;
            Ok(None)
        }
    }
}

pub fn write_workspace(path: &Path, workspace: &Workspace) -> AppResult<()> {
    backup_legacy_file(path)?;
    let file = ConfigFile::from_workspace(workspace);
    let json = serde_json::to_string_pretty(&file)
        .map_err(|error| AppError::Serialize(error.to_string()))?;
    write_atomically(path, &json)
}

fn backup_legacy_file(path: &Path) -> AppResult<()> {
    if !path.exists() {
        return Ok(());
    }
    let content =
        fs::read_to_string(path).map_err(|error| AppError::ReadConfig(error.to_string()))?;
    let is_legacy = serde_json::from_str::<Value>(&content)
        .ok()
        .and_then(|value| value.get("version").and_then(Value::as_u64))
        .is_none_or(|version| version < CONFIG_VERSION as u64);
    if !is_legacy {
        return Ok(());
    }
    let backup = backup_path(path);
    fs::copy(path, &backup).map_err(|error| AppError::BackupConfig(error.to_string()))?;
    Ok(())
}

fn write_atomically(path: &Path, contents: &str) -> AppResult<()> {
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, contents).map_err(|error| AppError::WriteConfig(error.to_string()))?;
    fs::rename(&temp, path).map_err(|error| {
        let _ = fs::remove_file(&temp);
        AppError::WriteConfig(error.to_string())
    })
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(CONFIG_FILE_NAME);
    path.with_file_name(format!("{file_name}.bak"))
}

fn quarantine(path: &Path) -> AppResult<()> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(CONFIG_FILE_NAME);
    let target = path.with_file_name(format!("{file_name}.corrupt-{stamp}.bak"));
    fs::rename(path, target).map_err(|error| AppError::BackupConfig(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::folder::Folder;
    use crate::domain::project::Project;

    fn sample_workspace() -> Workspace {
        let mut project = Project::new("工作");
        project.folders.push(Folder::new("A", "C:\\A"));
        Workspace {
            current_project_id: project.id.clone(),
            projects: vec![project],
        }
    }

    #[test]
    fn missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        assert!(read_workspace(&path).unwrap().is_none());
    }

    #[test]
    fn roundtrips_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let workspace = sample_workspace();

        write_workspace(&path, &workspace).unwrap();
        let loaded = read_workspace(&path).unwrap().unwrap();

        assert_eq!(loaded, workspace);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn backs_up_legacy_config_before_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let legacy =
            r#"{ "current_project": "工作", "projects": [{ "name": "工作", "folders": [] }] }"#;
        fs::write(&path, legacy).unwrap();

        write_workspace(&path, &sample_workspace()).unwrap();

        let backup = dir.path().join("config.json.bak");
        assert!(backup.exists());
        assert_eq!(fs::read_to_string(backup).unwrap(), legacy);
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("\"version\": 2"));
    }

    #[test]
    fn does_not_back_up_current_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        write_workspace(&path, &sample_workspace()).unwrap();
        write_workspace(&path, &sample_workspace()).unwrap();

        assert!(!dir.path().join("config.json.bak").exists());
    }

    #[test]
    fn quarantines_corrupt_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, "not json at all").unwrap();

        let loaded = read_workspace(&path).unwrap();

        assert!(loaded.is_none());
        assert!(!path.exists());
        let quarantined: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains(".corrupt-"))
            .collect();
        assert_eq!(quarantined.len(), 1);
    }

    #[test]
    fn quarantines_unsupported_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{ "version": 99, "projects": [] }"#).unwrap();

        assert!(read_workspace(&path).unwrap().is_none());
        assert!(!path.exists());
    }
}
