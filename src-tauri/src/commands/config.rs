use serde_json::Value;
use std::sync::Arc;
use tauri::State;

use crate::application::config_service::ConfigService;
use crate::error::AppResult;
use crate::infrastructure::persistence::config_file::{parse_config, ConfigFile};

#[tauri::command]
pub fn load_config(service: State<'_, Arc<ConfigService>>) -> AppResult<ConfigFile> {
    service
        .load()
        .map(|workspace| ConfigFile::from_workspace(&workspace))
}

#[tauri::command]
pub fn save_config(service: State<'_, Arc<ConfigService>>, data: Value) -> AppResult<()> {
    let workspace = parse_config(data)?;
    service.save(workspace)
}
