use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

/// Windows 单条路径的合理上限（NTFS 长路径上限 32767，这里取更保守的值）。
const MAX_PATH_LENGTH: usize = 4096;

/// 把用户/Agent 传来的路径解析为“真实存在的目录”的规范绝对路径。
///
/// 安全边界：
/// - 只接受绝对路径，不接受相对路径（因此不存在相对于进程工作目录的路径穿越）；
/// - 通过 `canonicalize` 解析 `.` / `..` / 符号链接，返回值一定落在真实目录上；
/// - 结果仅用于「查询目录名」与「作为字符串写入 config.json」，
///   本函数不会创建、打开或删除任何文件系统内容。
pub fn resolve_existing_directory(raw: &str) -> AppResult<PathBuf> {
    if raw.trim().is_empty() {
        return Err(AppError::InvalidArgument("path 不能为空".to_string()));
    }
    if raw.len() > MAX_PATH_LENGTH {
        return Err(AppError::InvalidArgument(format!(
            "path 过长（上限 {MAX_PATH_LENGTH} 字符）"
        )));
    }

    let path = Path::new(raw.trim());
    if !is_absolute(path) {
        return Err(AppError::PathNotAbsolute(raw.trim().to_string()));
    }

    let metadata =
        fs::metadata(path).map_err(|_| AppError::PathNotFound(path.display().to_string()))?;
    if !metadata.is_dir() {
        return Err(AppError::PathNotDirectory(path.display().to_string()));
    }

    let canonical = fs::canonicalize(path)
        .map_err(|error| AppError::Io(format!("{}: {error}", path.display())))?;
    if !canonical.is_dir() {
        return Err(AppError::PathNotDirectory(canonical.display().to_string()));
    }
    Ok(strip_verbatim_prefix(canonical))
}

/// Windows 的 `canonicalize` 会返回扩展长度前缀（`\\?\C:\...`）。
/// 写进配置前去掉它，保持与用户手工添加的路径（`C:\...`）一致、可读。
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    const VERBATIM: &str = r"\\?\";
    const VERBATIM_UNC: &str = r"\\?\UNC\";

    let text = path.to_string_lossy().to_string();
    if let Some(rest) = text.strip_prefix(VERBATIM_UNC) {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    match text.strip_prefix(VERBATIM) {
        Some(rest) => PathBuf::from(rest),
        None => path,
    }
}

/// 校验配置中已存在的路径（不做存在性检查，避免历史数据因目录被移动而报错）。
pub fn normalize_stored_path(raw: &str) -> AppResult<String> {
    if raw.trim().is_empty() {
        return Err(AppError::InvalidArgument("path 不能为空".to_string()));
    }
    if raw.len() > MAX_PATH_LENGTH {
        return Err(AppError::InvalidArgument(format!(
            "path 过长（上限 {MAX_PATH_LENGTH} 字符）"
        )));
    }
    let path = Path::new(raw.trim());
    if !is_absolute(path) {
        return Err(AppError::PathNotAbsolute(raw.trim().to_string()));
    }
    Ok(raw.trim().to_string())
}

/// 从规范化后的目录路径取目录名（`C:\work\src` → `src`）。
pub fn directory_name(path: &Path) -> Option<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
}

/// 判断路径是否与给定路径相同（Windows 语义：忽略大小写与结尾分隔符）。
#[cfg(test)]
pub fn paths_match(left: &str, right: &str) -> bool {
    use crate::domain::folder::normalize_path_for_comparison;

    normalize_path_for_comparison(left) == normalize_path_for_comparison(right)
}

/// 在给定路径列表中查找与目标路径相同的项。
#[cfg(test)]
pub fn find_matching_path<'a>(candidates: &'a [String], target: &str) -> Option<&'a String> {
    use crate::domain::folder::normalize_path_for_comparison;

    let normalized = normalize_path_for_comparison(target);
    candidates
        .iter()
        .find(|candidate| normalize_path_for_comparison(candidate) == normalized)
}

/// 是否是绝对路径。Windows 下 `C:\x`、`\\server\share`、`\\?\C:\x` 都算绝对路径；
/// 反斜杠形式需要在目标平台之外也能识别，因此额外做一次分隔符统一。
fn is_absolute(path: &Path) -> bool {
    if path.is_absolute() {
        return true;
    }
    // 非 Windows 平台（CI 上的 fmt/clippy 不需要，但保持行为一致）：
    // `\foo` 与 `C:/foo` 这样的 Windows 写法在这里补齐判断。
    let text = path.to_string_lossy().replace('/', "\\");
    let bytes = text.as_bytes();
    let drive_absolute =
        bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\';
    drive_absolute || text.starts_with("\\\\")
}

/// 把路径拆成组件，供测试断言"没有残留 `..`"使用（不含 Prefix / RootDir）。
#[cfg(test)]
fn component_names(path: &Path) -> Vec<String> {
    use std::path::Component;

    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().to_string()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_existing_directory_to_canonical_path() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();

        let resolved = resolve_existing_directory(&format!("{}\\", nested.display())).unwrap();

        assert_eq!(
            resolved,
            strip_verbatim_prefix(fs::canonicalize(&nested).unwrap())
        );
        assert_eq!(directory_name(&resolved).unwrap(), "nested");
        // 不应该把 Windows 扩展长度前缀（\\?\）写进配置
        assert!(!resolved.to_string_lossy().starts_with(r"\\?\"));
    }

    #[test]
    fn resolves_dot_dot_components_instead_of_allowing_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let inner = dir.path().join("inner");
        fs::create_dir(&inner).unwrap();

        let resolved = resolve_existing_directory(&format!("{}\\..", inner.display())).unwrap();

        assert_eq!(
            resolved,
            strip_verbatim_prefix(fs::canonicalize(dir.path()).unwrap())
        );
        assert!(!component_names(&resolved).contains(&"..".to_string()));
    }

    #[test]
    fn rejects_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing");

        assert!(matches!(
            resolve_existing_directory(&missing.display().to_string()),
            Err(AppError::PathNotFound(_))
        ));
    }

    #[test]
    fn rejects_file_path() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, "x").unwrap();

        assert!(matches!(
            resolve_existing_directory(&file.display().to_string()),
            Err(AppError::PathNotDirectory(_))
        ));
    }

    #[test]
    fn rejects_relative_and_empty_paths() {
        assert!(matches!(
            resolve_existing_directory("..\\..\\windows"),
            Err(AppError::PathNotAbsolute(_))
        ));
        assert!(matches!(
            resolve_existing_directory("   "),
            Err(AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            normalize_stored_path("relative\\dir"),
            Err(AppError::PathNotAbsolute(_))
        ));
    }

    #[test]
    fn matches_paths_ignoring_case_and_trailing_separator() {
        assert!(paths_match("C:\\Work\\Src\\", "c:/work/src"));

        let candidates = vec!["C:\\Work".to_string()];
        assert!(find_matching_path(&candidates, "c:\\work\\").is_some());
        assert!(find_matching_path(&candidates, "C:\\Other").is_none());
    }

    #[test]
    fn accepts_windows_absolute_forms() {
        assert!(is_absolute(Path::new("C:\\work")));
        assert!(is_absolute(Path::new("C:/work")));
        assert!(is_absolute(Path::new("\\\\server\\share")));
        assert!(!is_absolute(Path::new("work\\src")));
    }
}
