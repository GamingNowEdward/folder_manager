use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub path: String,
}

impl Folder {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            path: path.into(),
        }
    }
}

/// 添加文件夹的输入（Tauri 快照协议之外的按操作粒度用例共用）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderInput {
    pub name: String,
    pub path: String,
}

impl FolderInput {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
        }
    }
}

/// `add_folder` 的结果；重复 path 为幂等命中，不算失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddFolderOutcome {
    Created(Folder),
    AlreadyPresent(Folder),
}

impl AddFolderOutcome {
    pub fn folder(&self) -> &Folder {
        match self {
            Self::Created(folder) | Self::AlreadyPresent(folder) => folder,
        }
    }

    pub fn created(&self) -> bool {
        matches!(self, Self::Created(_))
    }

    pub fn into_folder(self) -> Folder {
        match self {
            Self::Created(folder) | Self::AlreadyPresent(folder) => folder,
        }
    }
}

/// 路径比较用的规范形式：统一分隔符、去掉结尾分隔符、去掉 Windows 扩展长度前缀、
/// 大小写归一（Windows 语义）。
pub fn normalize_path_for_comparison(path: &str) -> String {
    let trimmed = strip_verbatim_prefix(path.trim());
    let unified = trimmed.replace('/', "\\");
    let without_trailing = unified.trim_end_matches('\\');
    let kept = if without_trailing.is_empty() {
        unified.as_str()
    } else {
        without_trailing
    };
    kept.to_lowercase()
}

/// `\\?\C:\foo` → `C:\foo`，`\\?\UNC\server\share` → `\\server\share`。
/// 扩展长度前缀只是长路径写法，不指向另一个目录，比较时必须视为同一路径。
fn strip_verbatim_prefix(path: &str) -> String {
    let rest = if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        // 还原成普通 UNC 写法（`\\` + `server\share...`）
        return format!(r"\\{}", rest.trim_start_matches('\\'));
    } else {
        match path.strip_prefix(r"\\?\") {
            Some(rest) => rest,
            None => return path.to_string(),
        }
    };
    rest.to_string()
}

pub fn find_folder_by_path<'a>(folders: &'a [Folder], path: &str) -> Option<&'a Folder> {
    let target = normalize_path_for_comparison(path);
    folders
        .iter()
        .find(|folder| normalize_path_for_comparison(&folder.path) == target)
}

/// 向项目添加文件夹：相同 path 幂等（返回已存在项），同名不同 path 视为冲突。
pub fn add_folder(folders: &mut Vec<Folder>, input: FolderInput) -> AppResult<AddFolderOutcome> {
    if let Some(existing) = find_folder_by_path(folders, &input.path) {
        return Ok(AddFolderOutcome::AlreadyPresent(existing.clone()));
    }
    if input.name.is_empty() {
        return Err(AppError::InvalidFolderName);
    }
    if folders.iter().any(|folder| folder.name == input.name) {
        return Err(AppError::FolderNameTaken(input.name));
    }
    let folder = Folder::new(input.name, input.path);
    folders.push(folder.clone());
    Ok(AddFolderOutcome::Created(folder))
}

/// 按 id 移除「文件夹引用」（仅从列表中移除条目）。
/// **不触碰文件系统**：这里没有也无法删除真实目录。
pub fn remove_folder_by_id(folders: &mut Vec<Folder>, id: &str) -> AppResult<Folder> {
    let index = folders
        .iter()
        .position(|folder| folder.id == id)
        .ok_or_else(|| AppError::FolderNotFound(id.to_string()))?;
    Ok(folders.remove(index))
}

/// 按 path 移除「文件夹引用」（仅从列表中移除条目）。
/// **不触碰文件系统**：这里没有也无法删除真实目录。
pub fn remove_folder_by_path(folders: &mut Vec<Folder>, path: &str) -> AppResult<Folder> {
    let target = normalize_path_for_comparison(path);
    let index = folders
        .iter()
        .position(|folder| normalize_path_for_comparison(&folder.path) == target)
        .ok_or_else(|| AppError::FolderNotFound(path.to_string()))?;
    Ok(folders.remove(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folders() -> Vec<Folder> {
        vec![
            Folder {
                id: "f1".to_string(),
                name: "src".to_string(),
                path: "C:\\work\\src".to_string(),
            },
            Folder {
                id: "f2".to_string(),
                name: "docs".to_string(),
                path: "D:\\docs".to_string(),
            },
        ]
    }

    #[test]
    fn compares_paths_case_insensitively_and_ignores_trailing_separator() {
        assert_eq!(
            normalize_path_for_comparison("C:/Work/SRC/"),
            normalize_path_for_comparison("c:\\work\\src")
        );
        assert!(find_folder_by_path(&folders(), "c:/WORK/src/").is_some());
    }

    #[test]
    fn compares_verbatim_and_plain_windows_paths_as_equal() {
        // 扩展长度前缀只是长路径写法，指向同一个目录
        assert_eq!(
            normalize_path_for_comparison(r"\\?\C:\Work\Src"),
            normalize_path_for_comparison(r"c:\work\src")
        );
        assert_eq!(
            normalize_path_for_comparison(r"\\?\UNC\server\share\src"),
            normalize_path_for_comparison(r"\\server\share\src")
        );
        assert_eq!(
            normalize_path_for_comparison(r"\\?\C:\"),
            normalize_path_for_comparison(r"C:\")
        );
    }

    #[test]
    fn strips_verbatim_prefix_for_both_drive_and_unc_forms() {
        assert_eq!(strip_verbatim_prefix(r"\\?\C:\Work"), r"C:\Work");
        assert_eq!(
            strip_verbatim_prefix(r"\\?\UNC\server\share"),
            r"\\server\share"
        );
        assert_eq!(strip_verbatim_prefix(r"C:\Work"), r"C:\Work");
        assert_eq!(strip_verbatim_prefix(r"\\server\share"), r"\\server\share");
    }

    #[test]
    fn add_is_idempotent_for_same_path() {
        let mut list = folders();
        let outcome = add_folder(&mut list, FolderInput::new("src2", "C:\\work\\src")).unwrap();

        assert!(!outcome.created());
        assert_eq!(outcome.folder().id, "f1");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn add_rejects_duplicate_name_with_other_path() {
        let mut list = folders();
        let result = add_folder(&mut list, FolderInput::new("src", "C:\\other\\src"));

        assert!(matches!(result, Err(AppError::FolderNameTaken(_))));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn add_rejects_empty_name() {
        let mut list = folders();
        let result = add_folder(&mut list, FolderInput::new("", "C:\\other"));

        assert!(matches!(result, Err(AppError::InvalidFolderName)));
    }

    #[test]
    fn remove_by_id_and_path() {
        let mut list = folders();
        assert_eq!(remove_folder_by_id(&mut list, "f1").unwrap().name, "src");
        assert_eq!(
            remove_folder_by_path(&mut list, "d:/DOCS/").unwrap().name,
            "docs"
        );
        assert!(list.is_empty());
    }

    #[test]
    fn remove_reports_missing_folder() {
        let mut list = folders();

        assert!(matches!(
            remove_folder_by_id(&mut list, "missing"),
            Err(AppError::FolderNotFound(_))
        ));
        assert!(matches!(
            remove_folder_by_path(&mut list, "C:\\missing"),
            Err(AppError::FolderNotFound(_))
        ));
    }
}
