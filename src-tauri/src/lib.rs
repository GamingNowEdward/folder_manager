mod application;
mod commands;
mod domain;
mod error;
mod infrastructure;

use application::config_service::{AppEventHandle, ConfigService};
use infrastructure::http::server::{self, ServerHandle};
use infrastructure::persistence::settings_file;
use tauri::{Manager, RunEvent};

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // 唯一的配置权威状态：Tauri command 与 HTTP API 共用同一个 Arc<ConfigService>。
            let service = ConfigService::initialize(
                app.handle(),
                Some(Box::new(AppEventHandle::new(app.handle().clone()))),
            )?
            .shared();
            app.manage(service.clone());

            // 本地 HTTP API：启动失败只记录日志，主 UI 照常可用（health 不可用）。
            eprintln!(
                "[folder-manager] 配置文件: {}",
                service.config_path().display()
            );
            // 端口优先级：环境变量 > settings.json > 默认 17890。
            let settings_path = settings_file::resolve_settings_path(app.handle())?;
            let initial_port = server::env_port()
                .or_else(|| settings_file::read_api_port(&settings_path))
                .unwrap_or(server::DEFAULT_API_PORT);
            match server::start(service, env!("CARGO_PKG_VERSION"), initial_port) {
                Ok(handle) => {
                    app.manage(handle);
                }
                Err(error) => {
                    eprintln!("[folder-manager-api] error 启动失败: {error}");
                }
            }

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
            commands::system::open_folder,
            commands::api_settings::get_api_settings,
            commands::api_settings::set_api_port,
            commands::api_settings::api_documentation
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if matches!(event, RunEvent::Exit) {
            // 退出时优雅关闭 HTTP server，释放端口。
            if let Some(handle) = app_handle.try_state::<ServerHandle>() {
                handle.shutdown();
            }
        }
    });
}
