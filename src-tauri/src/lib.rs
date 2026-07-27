use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[cfg(target_os = "windows")]
mod acrylic {
    use std::ffi::c_void;

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            dw_attribute: u32,
            pv_attribute: *const c_void,
            cb_attribute: u32,
        ) -> i32;
    }

    const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
    const DWMSBT_ACRYLIC: i32 = 3;

    pub fn enable(hwnd: isize) -> bool {
        unsafe {
            let backdrop = DWMSBT_ACRYLIC;
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &backdrop as *const _ as *const c_void,
                std::mem::size_of::<i32>() as u32,
            ) == 0
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppData {
    pub current_project: String,
    pub projects: Vec<Project>,
}

struct AppState {
    data: Mutex<AppData>,
}

fn get_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(not(debug_assertions)) {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("无法获取 exe 路径: {}", e))?;
        let dir = exe_path.parent()
            .ok_or_else(|| "无法获取 exe 目录".to_string())?;
        Ok(dir.join("config.json"))
    } else {
        let mut dir = app.path().app_data_dir()
            .map_err(|e| format!("无法获取数据目录: {}", e))?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;
        dir.push("config.json");
        Ok(dir)
    }
}

#[tauri::command]
fn load_config(state: State<AppState>) -> Result<AppData, String> {
    let data = state.data.lock().map_err(|e| format!("锁获取失败: {}", e))?;
    Ok(data.clone())
}

#[tauri::command]
fn save_config(app: AppHandle, state: State<AppState>, data: AppData) -> Result<(), String> {
    {
        let mut current = state.data.lock().map_err(|e| format!("锁获取失败: {}", e))?;
        *current = data.clone();
    }
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    let path = get_config_path(&app)?;
    fs::write(&path, &json)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    if !std::path::Path::new(&path).exists() {
        return Err(format!("路径不存在: {}", path));
    }
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("打开文件夹失败: {}", e))?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let path = get_config_path(&app.handle())
                .unwrap_or_else(|_| {
                    let mut dir = std::env::temp_dir();
                    dir.push("folder-manager-config.json");
                    dir
                });
            let data = fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<AppData>(&s).ok())
                .unwrap_or(AppData {
                    current_project: String::new(),
                    projects: Vec::new(),
                });
            app.manage(AppState {
                data: Mutex::new(data),
            });

            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    acrylic::enable(hwnd.0 as isize);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_config, save_config, open_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
