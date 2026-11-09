use crate::error::{AppError, AppResult};
use crate::infrastructure::paths::resolve_existing_directory;
use crate::infrastructure::windows::shell;

#[tauri::command]
pub fn open_folder(path: String) -> AppResult<()> {
    shell::open_folder(&path)
}

/// 拖入路径的过滤结果：只保留真实目录。
#[derive(serde::Serialize)]
pub struct DroppedDirectories {
    /// canonical 后的目录绝对路径（已去掉 Windows 的 `\\?\` 前缀）。
    pub directories: Vec<String>,
    /// 存在但明确是文件的数量。
    pub files: usize,
    /// 无法解析的路径数量：不存在、无权限、相对路径、超长等。
    pub invalid: usize,
}

/// 把拖入的路径筛选为真实存在的目录。
///
/// 不返回 `Result`：单个坏路径不应让整次拖入失败，只按类别计数。
/// 判定与规范化复用 `resolve_existing_directory`，保证拖入、手工添加、HTTP API
/// 三条来源写入的路径写法一致。
#[tauri::command]
pub fn filter_directories(paths: Vec<String>) -> DroppedDirectories {
    let mut directories = Vec::with_capacity(paths.len());
    let mut files = 0usize;
    let mut invalid = 0usize;

    for path in paths {
        match resolve_existing_directory(&path) {
            Ok(directory) => directories.push(directory.to_string_lossy().to_string()),
            Err(AppError::PathNotDirectory(_)) => files += 1,
            Err(_) => invalid += 1,
        }
    }

    DroppedDirectories {
        directories,
        files,
        invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn as_string(path: &Path) -> String {
        path.to_string_lossy().to_string()
    }

    fn with_trailing_separator(path: &Path) -> String {
        format!("{}{}", path.display(), std::path::MAIN_SEPARATOR)
    }

    #[test]
    fn keeps_only_directories() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let file = root.path().join("note.txt");
        fs::write(&file, "x").unwrap();

        let result = filter_directories(vec![
            as_string(&first),
            as_string(&file),
            as_string(&second),
            as_string(&root.path().join("missing")),
            "relative\\dir".to_string(),
        ]);

        assert_eq!(result.directories.len(), 2);
        assert_eq!(result.files, 1);
        assert_eq!(result.invalid, 2);
    }

    #[test]
    fn normalizes_directory_paths() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        fs::create_dir(&target).unwrap();

        let result = filter_directories(vec![
            with_trailing_separator(&target),
            as_string(&target.join("..")),
        ]);

        assert_eq!(result.directories.len(), 2);
        for directory in &result.directories {
            assert!(!directory.starts_with(r"\\?\"));
            assert!(!directory.contains(".."));
        }
    }

    #[test]
    fn empty_input_returns_empty_result() {
        let result = filter_directories(Vec::new());

        assert!(result.directories.is_empty());
        assert_eq!(result.files, 0);
        assert_eq!(result.invalid, 0);
    }
}
