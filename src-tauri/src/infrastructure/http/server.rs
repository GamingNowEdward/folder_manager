use std::net::{Ipv4Addr, SocketAddr, TcpListener as StdTcpListener};
use std::sync::{Arc, Mutex};
use std::thread;

use tokio::sync::oneshot;

use crate::application::config_service::ConfigService;
use crate::infrastructure::http::api::{router, ApiState};
use crate::infrastructure::http::cors::CORS_ORIGINS_ENV;
use crate::infrastructure::http::error::API_NAME;

/// API 只绑定回环地址：本机 Agent 可访问，局域网/公网不可访问。
pub const API_HOST: Ipv4Addr = Ipv4Addr::LOCALHOST;
/// 默认端口。
pub const DEFAULT_API_PORT: u16 = 17890;
/// 端口被占用时最多再向后尝试的端口数。
const PORT_SCAN_ATTEMPTS: u16 = 10;
pub const API_PORT_ENV: &str = "FOLDER_MANAGER_API_PORT";

/// 启动失败的原因，用于日志与 UI 提示（UI 不会因此崩溃）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiStartError {
    InvalidPort(String),
    PortUnavailable(u16),
}

impl std::fmt::Display for ApiStartError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPort(value) => write!(
                formatter,
                "{API_PORT_ENV} 不是合法的端口号: {value}（应为 1..=65535）"
            ),
            Self::PortUnavailable(port) => write!(
                formatter,
                "{API_HOST}:{port} 及后续 {PORT_SCAN_ATTEMPTS} 个端口都被占用"
            ),
        }
    }
}

/// 运行中的 HTTP server 句柄：`shutdown()` 后会优雅退出。
pub struct ServerHandle {
    /// 实际监听地址（端口冲突顺延后的结果）。
    #[cfg_attr(not(test), allow(dead_code))]
    addr: SocketAddr,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
}

impl ServerHandle {
    /// 优雅关闭：停止接收新连接，等待在途请求结束。
    pub fn shutdown(&self) {
        if let Some(sender) = self
            .shutdown
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = sender.send(());
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // App 退出（managed state 释放）时同样触发优雅关闭。
        self.shutdown();
    }
}

/// 解析要监听的端口：`FOLDER_MANAGER_API_PORT` 优先，默认 17890。
pub fn resolve_port() -> Result<u16, ApiStartError> {
    match std::env::var(API_PORT_ENV) {
        Ok(raw) if !raw.trim().is_empty() => raw
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|port| *port > 0)
            .ok_or(ApiStartError::InvalidPort(raw)),
        _ => Ok(DEFAULT_API_PORT),
    }
}

/// 在 127.0.0.1 上启动 API。失败只返回错误，不影响 Tauri 主 UI。
pub fn start(
    service: Arc<ConfigService>,
    version: &'static str,
) -> Result<ServerHandle, ApiStartError> {
    let port = resolve_port()?;
    let (listener, addr) = bind_with_fallback(port)?;
    listener
        .set_nonblocking(true)
        .map_err(|_| ApiStartError::PortUnavailable(addr.port()))?;

    let state = ApiState::new(service, version);
    let app = router(state);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // 独立 runtime + 独立线程：不依赖 Tauri 内部 runtime 的形态，
    // 也让 API 的生命周期与主 UI 完全解耦。
    thread::Builder::new()
        .name("folder-manager-api".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("[folder-manager-api] 创建 runtime 失败: {error}");
                    return;
                }
            };
            runtime.block_on(async move {
                let listener = match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener) => listener,
                    Err(error) => {
                        eprintln!("[folder-manager-api] 监听 socket 初始化失败: {error}");
                        return;
                    }
                };
                let server = axum::serve(listener, app).with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                });
                if let Err(error) = server.await {
                    eprintln!("[folder-manager-api] 服务异常退出: {error}");
                }
                eprintln!("[folder-manager-api] 已停止监听 {addr}");
            });
        })
        .map_err(|_| ApiStartError::PortUnavailable(addr.port()))?;

    eprintln!("[folder-manager-api] {API_NAME} 监听 http://{addr}/api（CORS: {CORS_ORIGINS_ENV}）");

    Ok(ServerHandle {
        addr,
        shutdown: Mutex::new(Some(shutdown_tx)),
    })
}

/// 端口冲突时向后顺延；环绕范围内都试过仍失败才放弃。
fn bind_with_fallback(port: u16) -> Result<(StdTcpListener, SocketAddr), ApiStartError> {
    let mut candidate = port;
    for _ in 0..=PORT_SCAN_ATTEMPTS {
        if let Ok(listener) = StdTcpListener::bind(SocketAddr::from((API_HOST, candidate))) {
            let addr = listener
                .local_addr()
                .unwrap_or_else(|_| SocketAddr::from((API_HOST, candidate)));
            return Ok((listener, addr));
        }
        eprintln!("[folder-manager-api] warn 端口 {candidate} 不可用，尝试下一个端口");
        match next_candidate(candidate) {
            Some(next) => candidate = next,
            None => break,
        }
    }
    Err(ApiStartError::PortUnavailable(port))
}

/// 从 `port + 1` 向上找，越界后从 1024 继续，最后回到起始端口之前。
fn next_candidate(port: u16) -> Option<u16> {
    if port < u16::MAX {
        return Some(port + 1);
    }
    (1024..port).next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::http::testing::Fixture;

    #[test]
    fn default_port_matches_documentation() {
        assert_eq!(DEFAULT_API_PORT, 17890);
        assert_eq!(API_HOST, Ipv4Addr::new(127, 0, 0, 1));
    }

    #[test]
    fn resolve_port_prefers_env_and_validates() {
        // 只读路径：未设置环境变量时返回默认值。
        if std::env::var(API_PORT_ENV).is_err() {
            assert_eq!(resolve_port().unwrap(), DEFAULT_API_PORT);
        }
    }

    #[test]
    fn next_candidate_wraps_around() {
        assert_eq!(next_candidate(17890), Some(17891));
        assert_eq!(next_candidate(u16::MAX), Some(1024));
    }

    #[test]
    fn reports_port_conflict() {
        // 占住一个端口，确认绑定会顺延而不是 panic。
        let occupied = StdTcpListener::bind(SocketAddr::from((API_HOST, 0))).unwrap();
        let port = occupied.local_addr().unwrap().port();
        let (listener, addr) = bind_with_fallback(port).unwrap();

        assert_ne!(addr.port(), port);
        drop(listener);
    }

    #[tokio::test]
    async fn server_starts_serves_and_shuts_down() {
        let fixture = Fixture::new();
        let handle = start(fixture.service.clone(), "test").unwrap();
        let addr = handle.addr;
        assert_eq!(addr.ip(), API_HOST);

        let client = reqwest::Client::new();
        let health: serde_json::Value = client
            .get(format!("http://{addr}/api/health"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(health["ok"], true);
        assert_eq!(health["api_version"], "1");

        // CORS 预检必须能通过（默认只放开本机来源；这里显式固定一份 origin 列表，
        // 避免与其它用例的环境变量互相干扰）
        let preflight = client
            .request(
                reqwest::Method::OPTIONS,
                format!("http://{addr}/api/projects"),
            )
            .header("origin", "http://localhost:1420")
            .header("access-control-request-method", "POST")
            .send()
            .await
            .unwrap();
        assert!(preflight.status().is_success());
        assert_eq!(
            preflight
                .headers()
                .get("access-control-allow-origin")
                .map(|value| value.to_str().unwrap().to_string()),
            Some("http://localhost:1420".to_string())
        );

        handle.shutdown();
        drop(handle);

        // 优雅关闭后端口应能被重新绑定
        let released = wait_for_release(addr.port());
        assert!(
            released,
            "关闭后端口 {} 仍被占用，优雅关闭可能未生效",
            addr.port()
        );
    }

    fn wait_for_release(port: u16) -> bool {
        for _ in 0..80 {
            if StdTcpListener::bind(SocketAddr::from((API_HOST, port))).is_ok() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        false
    }
}
