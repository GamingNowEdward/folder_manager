//! 应用级设置的持久化（`settings.json`），与项目数据 `config.json` 分离。
//!
//! 目前只有本地 HTTP API 端口一个字段。只有 Rust 端读写这个文件，
//! 前端永远不直接落盘，因此不会与 `config.json` 的快照保存产生竞态。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::json_store;

const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsFile {
    api_port: u16,
}

/// `settings.json` 与 `config.json` 同目录。
pub fn resolve_settings_path(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(json_store::resolve_config_dir(app)?.join(SETTINGS_FILE_NAME))
}

/// 读取持久化的 API 端口。文件不存在 / 内容损坏 / 端口非法（0）时返回 `None`，
/// 由调用方回落到默认端口。
pub fn read_api_port(path: &Path) -> Option<u16> {
    let content = std::fs::read_to_string(path).ok()?;
    let settings: SettingsFile = serde_json::from_str(&content).ok()?;
    (settings.api_port > 0).then_some(settings.api_port)
}

/// 原子写入 API 端口。
pub fn write_api_port(path: &Path, port: u16) -> AppResult<()> {
    if port == 0 {
        return Err(AppError::InvalidArgument(
            "端口必须是 1..=65535".to_string(),
        ));
    }
    let settings = SettingsFile { api_port: port };
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|error| AppError::Serialize(error.to_string()))?;
    json_store::write_atomically(path, &json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_api_port() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        write_api_port(&path, 23456).unwrap();

        assert_eq!(read_api_port(&path), Some(23456));
        // 写入是原子的，不会留下临时文件
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn missing_file_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        assert_eq!(read_api_port(&path), None);
    }

    #[test]
    fn corrupt_or_illegal_file_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(read_api_port(&path), None);

        std::fs::write(&path, r#"{ "api_port": 0 }"#).unwrap();
        assert_eq!(read_api_port(&path), None);
    }

    #[test]
    fn rejects_zero_port() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        assert!(write_api_port(&path, 0).is_err());
        assert!(!path.exists());
    }
}
