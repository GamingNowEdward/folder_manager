use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::error::AppError;

/// API 名称与契约版本（与 `GET /api` 的返回一致）。
pub const API_NAME: &str = "Folder Manager API";
pub const API_VERSION: &str = "1";

/// 统一的错误响应体：所有 endpoint 都用这一种格式。
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
}

/// API 错误：稳定的 `code` + 人类/Agent 可读的 `message` + 正确的 HTTP 状态码。
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message)
    }

    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

/// 领域错误 → API 错误。业务逻辑（domain / application）不感知 HTTP。
impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::InvalidArgument(message) => Self::bad_request("INVALID_ARGUMENT", message),
            AppError::InvalidProjectName => {
                Self::bad_request("INVALID_ARGUMENT", "项目名称不能为空")
            }
            AppError::InvalidFolderName => {
                Self::bad_request("INVALID_ARGUMENT", "文件夹名称不能为空")
            }
            AppError::PathNotAbsolute(path) => Self::bad_request(
                "INVALID_PATH",
                format!("path 必须是绝对路径（Windows 形如 C:\\\\work\\\\project）: {path}"),
            ),
            AppError::PathNotFound(path) => {
                Self::bad_request("PATH_NOT_FOUND", format!("path 不存在: {path}"))
            }
            AppError::PathNotDirectory(path) => {
                Self::bad_request("PATH_NOT_DIRECTORY", format!("path 存在但不是目录: {path}"))
            }
            AppError::ProjectNotFound(id) => {
                Self::not_found("PROJECT_NOT_FOUND", format!("项目不存在: {id}"))
            }
            AppError::FolderNotFound(id) => {
                Self::not_found("FOLDER_NOT_FOUND", format!("文件夹不存在: {id}"))
            }
            AppError::ProjectNameTaken(name) => {
                Self::conflict("PROJECT_NAME_TAKEN", format!("项目名称已存在: {name}"))
            }
            AppError::FolderNameTaken(name) => Self::conflict(
                "FOLDER_NAME_TAKEN",
                format!("该项目中已存在同名文件夹: {name}"),
            ),
            other => Self::internal(other.to_string()),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    async fn error_json(error: ApiError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn maps_domain_errors_to_status_codes() {
        let cases = [
            (
                AppError::ProjectNotFound("p1".to_string()),
                StatusCode::NOT_FOUND,
                "PROJECT_NOT_FOUND",
            ),
            (
                AppError::FolderNotFound("f1".to_string()),
                StatusCode::NOT_FOUND,
                "FOLDER_NOT_FOUND",
            ),
            (
                AppError::PathNotFound("C:\\nope".to_string()),
                StatusCode::BAD_REQUEST,
                "PATH_NOT_FOUND",
            ),
            (
                AppError::PathNotDirectory("C:\\file.txt".to_string()),
                StatusCode::BAD_REQUEST,
                "PATH_NOT_DIRECTORY",
            ),
            (
                AppError::PathNotAbsolute("..\\x".to_string()),
                StatusCode::BAD_REQUEST,
                "INVALID_PATH",
            ),
            (
                AppError::InvalidArgument("bad".to_string()),
                StatusCode::BAD_REQUEST,
                "INVALID_ARGUMENT",
            ),
            (
                AppError::ProjectNameTaken("A".to_string()),
                StatusCode::CONFLICT,
                "PROJECT_NAME_TAKEN",
            ),
            (
                AppError::FolderNameTaken("src".to_string()),
                StatusCode::CONFLICT,
                "FOLDER_NAME_TAKEN",
            ),
            (
                AppError::ReadConfig("io".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
        ];

        for (error, expected_status, expected_code) in cases {
            let (status, body) = error_json(ApiError::from(error)).await;
            assert_eq!(status, expected_status);
            assert_eq!(body["error"]["code"], expected_code);
            assert!(body["error"]["message"]
                .as_str()
                .is_some_and(|it| !it.is_empty()));
        }
    }

    #[test]
    fn body_uses_single_error_shape() {
        let error = ApiError::bad_request("INVALID_ARGUMENT", "x");
        let body = serde_json::to_value(ErrorBody {
            error: ErrorDetail {
                code: error.code,
                message: error.message.clone(),
            },
        })
        .unwrap();

        assert_eq!(
            body,
            serde_json::json!({ "error": { "code": "INVALID_ARGUMENT", "message": "x" } })
        );
    }
}
