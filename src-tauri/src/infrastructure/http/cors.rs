use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::request::Parts;
use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

/// 允许的 origin 由 `FOLDER_MANAGER_API_CORS_ORIGINS` 配置：
/// - 未设置：默认只允许本机页面来源（`http://localhost` / `http://127.0.0.1` 的任意端口）；
/// - `*`：允许任意 origin（不推荐，仅在本机可信环境下使用）；
/// - 逗号分隔的完整 origin 列表：只允许列表内的 origin。
///
/// curl / Python / Node / Coding Agent 等非浏览器客户端不发送 Origin，
/// 因此不受 CORS 影响，可以直接访问 127.0.0.1。
pub const CORS_ORIGINS_ENV: &str = "FOLDER_MANAGER_API_CORS_ORIGINS";

pub fn allowed_origins_from_env() -> Vec<String> {
    match std::env::var(CORS_ORIGINS_ENV) {
        Ok(raw) => parse_origins(&raw, default_origins()),
        Err(_) => default_origins(),
    }
}

fn default_origins() -> Vec<String> {
    // 开发模式下 Tauri 的 devUrl 是 http://localhost:1420，默认放开本机来源；
    // release 构建默认不放开任何浏览器 origin（非浏览器 Agent 依然可用）。
    if cfg!(debug_assertions) {
        vec![
            "http://localhost".to_string(),
            "http://127.0.0.1".to_string(),
            "tauri://localhost".to_string(),
            "https://tauri.localhost".to_string(),
        ]
    } else {
        Vec::new()
    }
}

fn parse_origins(raw: &str, fallback: Vec<String>) -> Vec<String> {
    if raw.trim() == "*" {
        return vec!["*".to_string()];
    }
    let configured: Vec<String> = raw
        .split(',')
        .map(|origin| origin.trim().trim_end_matches('/').to_string())
        .filter(|origin| !origin.is_empty())
        .collect();
    if configured.is_empty() {
        return fallback;
    }
    configured
}

pub fn layer(allowed: &[String]) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    if allowed.iter().any(|origin| origin == "*") {
        return base.allow_origin(AllowOrigin::any());
    }

    let rules = allowed.to_vec();
    base.allow_origin(AllowOrigin::predicate(
        move |origin: &HeaderValue, _parts: &Parts| {
            origin
                .to_str()
                .map(|origin| origin_allowed(origin, &rules))
                .unwrap_or(false)
        },
    ))
}

/// 解析 `scheme://host[:port]`，无 scheme 的写法返回 `None`。
fn split_origin(origin: &str) -> Option<(&str, &str, Option<&str>)> {
    let (scheme, rest) = origin.trim_end_matches('/').split_once("://")?;
    match rest.split_once(':') {
        Some((host, port)) => Some((scheme, host, Some(port))),
        None => Some((scheme, rest, None)),
    }
}

/// 规则匹配：
/// - `*`：任意 origin；
/// - `example.com`（只有主机名）：匹配该主机名的任意 scheme/端口；
/// - `https://host`：匹配同 scheme + 同主机名的任意端口；
/// - `https://host:8443`：精确匹配（scheme + 主机名 + 端口）；
/// - `*.localhost`：额外允许该主机的任意子域（Tauri 的 `tauri.localhost`）。
pub fn origin_allowed(origin: &str, rules: &[String]) -> bool {
    let origin_parts = split_origin(origin);
    rules.iter().any(|rule| match split_origin(rule) {
        // 只写主机名的规则：匹配同主机名的任意 scheme / 端口
        None => match origin_parts {
            Some((_, host, _)) => rule == "*" || host == rule,
            None => false,
        },
        Some((rule_scheme, rule_host, rule_port)) => {
            let Some((scheme, host, port)) = split_origin(origin) else {
                return false;
            };
            if scheme != rule_scheme {
                return false;
            }
            let subdomain = rule_host
                .strip_prefix("*.")
                .is_some_and(|base| host.ends_with(&format!(".{base}")));
            if host != rule_host && !subdomain {
                return false;
            }
            match rule_port {
                Some(expected) => port == Some(expected),
                None => true,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(list: &[&str]) -> Vec<String> {
        list.iter().map(|item| item.to_string()).collect()
    }

    #[test]
    fn parses_configured_origins() {
        let origins = parse_origins(
            " http://localhost:5173 , https://tool.example.com/ ",
            default_origins(),
        );

        assert_eq!(
            origins,
            vec![
                "http://localhost:5173".to_string(),
                "https://tool.example.com".to_string()
            ]
        );
    }

    #[test]
    fn wildcard_is_explicit() {
        assert_eq!(parse_origins("*", default_origins()), vec!["*".to_string()]);
        assert_eq!(
            parse_origins("  *  ", default_origins()),
            vec!["*".to_string()]
        );
    }

    #[test]
    fn empty_configuration_falls_back_to_defaults() {
        let fallback = vec!["http://localhost".to_string()];
        assert_eq!(parse_origins("   ", fallback.clone()), fallback);
    }

    #[test]
    fn defaults_only_allow_local_origins() {
        let origins = default_origins();

        if cfg!(debug_assertions) {
            assert!(origins.iter().any(|it| it == "http://localhost"));
            assert!(origins.iter().any(|it| it == "http://127.0.0.1"));
            assert!(!origins.iter().any(|it| it == "*"));
        } else {
            assert!(origins.is_empty());
        }
    }

    #[test]
    fn matches_scheme_and_host_rules_with_any_port() {
        let rules = rules(&["http://localhost"]);

        assert!(origin_allowed("http://localhost:1420", &rules));
        assert!(origin_allowed("http://localhost", &rules));
        // scheme 或主机名不同都不允许
        assert!(!origin_allowed("https://localhost:1420", &rules));
        assert!(!origin_allowed("http://evil.example.com", &rules));
    }

    #[test]
    fn matches_exact_origin_rules_with_port() {
        let rules = rules(&["https://tool.example.com:8443"]);

        assert!(origin_allowed("https://tool.example.com:8443", &rules));
        assert!(!origin_allowed("https://tool.example.com", &rules));
        assert!(!origin_allowed("https://tool.example.com:9443", &rules));
        assert!(!origin_allowed("http://tool.example.com:8443", &rules));
    }

    #[test]
    fn matches_subdomains_of_wildcard_rules() {
        let rules = rules(&["https://*.localhost"]);

        assert!(origin_allowed("https://tauri.localhost", &rules));
        assert!(!origin_allowed("tauri://localhost", &rules));
        assert!(!origin_allowed("https://localhost", &rules));
        assert!(!origin_allowed("https://other.example.com", &rules));
    }

    #[test]
    fn matches_host_only_rules() {
        let rules = rules(&["localhost"]);

        assert!(origin_allowed("http://localhost:1420", &rules));
        assert!(origin_allowed("http://localhost", &rules));
        assert!(!origin_allowed("http://evil.example.com", &rules));
    }

    #[test]
    fn wildcard_allows_everything() {
        assert!(origin_allowed("http://evil.example.com", &rules(&["*"])));
    }

    #[test]
    fn rejects_everything_without_rules() {
        assert!(!origin_allowed("http://localhost:1420", &[]));
    }

    #[tokio::test]
    async fn layer_answers_preflight_for_allowed_origin() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/probe", axum::routing::post(|| async { "ok" }))
            .layer(layer(&rules(&["http://localhost"])));
        let request = Request::builder()
            .method("OPTIONS")
            .uri("/probe")
            .header("origin", "http://localhost:1420")
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let allow_origin = response
            .headers()
            .get("access-control-allow-origin")
            .map(|value| value.to_str().unwrap().to_string());

        assert_eq!(allow_origin.as_deref(), Some("http://localhost:1420"));
    }

    #[tokio::test]
    async fn layer_omits_header_for_foreign_origin() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let app = axum::Router::new()
            .route("/probe", axum::routing::post(|| async { "ok" }))
            .layer(layer(&rules(&["http://localhost"])));
        let request = Request::builder()
            .method("OPTIONS")
            .uri("/probe")
            .header("origin", "http://evil.example.com")
            .header("access-control-request-method", "POST")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_none());
    }
}
