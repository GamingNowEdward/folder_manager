//! 设置界面对应的 Tauri command：读写本地 HTTP API 端口、复制接口文档。
//!
//! 端口设置持久化在 `settings.json`，**保存后需重启应用生效**——
//! 这里不会去重启正在运行的 HTTP server，避免运行期切换端口的复杂性。
//! 端口解析优先级：`FOLDER_MANAGER_API_PORT` 环境变量 > settings.json > 默认 17890。

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::AppResult;
use crate::infrastructure::http::server::{self, ServerHandle, DEFAULT_API_PORT};
use crate::infrastructure::persistence::settings_file;

/// 编译期内嵌接口文档：发行版不随 exe 分发 `docs/API.md`，所以直接嵌进二进制。
const API_DOCUMENTATION: &str = include_str!("../../../docs/API.md");

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSettingsDto {
    /// 已保存（下次启动将使用）的端口。
    saved_port: u16,
    /// 当前实际监听端口；API 未启动时为 None。
    actual_port: Option<u16>,
    default_port: u16,
    /// 当前可用的 Base URL；API 未启动时为 None。
    url: Option<String>,
    /// 是否由 `FOLDER_MANAGER_API_PORT` 环境变量覆盖（覆盖时界面设置不会立即生效）。
    env_override: bool,
    /// 已保存端口与当前实际端口不一致（需重启应用才生效）。
    restart_required: bool,
}

#[tauri::command]
pub fn get_api_settings(app: AppHandle) -> AppResult<ApiSettingsDto> {
    Ok(build_dto(&app))
}

#[tauri::command]
pub fn set_api_port(app: AppHandle, port: u16) -> AppResult<ApiSettingsDto> {
    let path = settings_file::resolve_settings_path(&app)?;
    settings_file::write_api_port(&path, port)?;
    Ok(build_dto(&app))
}

#[tauri::command]
pub fn api_documentation(app: AppHandle) -> String {
    let port = app
        .try_state::<ServerHandle>()
        .map(|handle| handle.address().port());
    documentation_with_header(port)
}

fn build_dto(app: &AppHandle) -> ApiSettingsDto {
    let saved_port = settings_file::resolve_settings_path(app)
        .ok()
        .as_deref()
        .and_then(settings_file::read_api_port)
        .unwrap_or(DEFAULT_API_PORT);
    let env_override = server::env_port().is_some();
    let actual_port = app
        .try_state::<ServerHandle>()
        .map(|handle| handle.address().port());

    ApiSettingsDto {
        saved_port,
        actual_port,
        default_port: DEFAULT_API_PORT,
        url: actual_port.map(|port| format!("http://127.0.0.1:{port}/api")),
        env_override,
        restart_required: actual_port.is_some_and(|actual| !env_override && saved_port != actual),
    }
}

/// 在文档开头附一段动态抬头（当前实际监听地址），正文保持原样不改写，
/// 避免污染文档里「默认端口 17890」等固定表述。
fn documentation_with_header(port: Option<u16>) -> String {
    let header = match port {
        Some(port) => format!(
            "> 当前 Folder Manager API 基址：http://127.0.0.1:{port}/api\n\
             > （下方文档示例中的默认端口为 {DEFAULT_API_PORT}；若与本基址不同，以本基址为准。）"
        ),
        None => "> 注意：本地 HTTP API 当前未启动。".to_string(),
    };
    format!("{header}\n\n{API_DOCUMENTATION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documentation_embeds_current_port_and_full_body() {
        let text = documentation_with_header(Some(23456));

        assert!(text.contains("http://127.0.0.1:23456/api"));
        // 正文原样保留：默认端口与对外承诺的 endpoint 都还在
        assert!(text.contains("/api/health"));
        assert!(text.contains("/api/projects"));
        assert!(text.contains(&DEFAULT_API_PORT.to_string()));
    }

    #[test]
    fn documentation_when_server_down_keeps_body() {
        let text = documentation_with_header(None);

        assert!(text.contains("未启动"));
        assert!(text.contains("/api/health"));
    }
}
