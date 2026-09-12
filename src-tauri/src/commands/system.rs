use crate::error::AppResult;
use crate::infrastructure::windows::shell;

#[tauri::command]
pub fn open_folder(path: String) -> AppResult<()> {
    shell::open_folder(&path)
}
