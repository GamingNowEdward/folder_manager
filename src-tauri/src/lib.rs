mod application;
mod commands;
mod domain;
mod error;
mod infrastructure;

use application::config_service::ConfigService;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let service = ConfigService::initialize(app.handle())?;
            app.manage(service);

            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(hwnd) = window.hwnd() {
                    infrastructure::windows::acrylic::enable(hwnd.0 as isize);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::load_config,
            commands::config::save_config,
            commands::system::open_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
