pub mod api;
pub mod cors;
pub mod error;
pub mod server;

#[cfg(test)]
pub mod testing;

/// 真实 HTTP 请求级别的契约测试。
#[cfg(test)]
mod integration_tests;
