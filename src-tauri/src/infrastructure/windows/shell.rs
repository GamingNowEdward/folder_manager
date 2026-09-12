use std::path::Path;
use std::process::Command;

use crate::error::{AppError, AppResult};

pub fn open_folder(path: &str) -> AppResult<()> {
    if !Path::new(path).exists() {
        return Err(AppError::PathNotFound(path.to_string()));
    }
    Command::new("explorer")
        .arg(path)
        .spawn()
        .map_err(|error| AppError::OpenFolder(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_path() {
        let result = open_folder("Z:\\definitely\\missing\\folder");
        assert!(matches!(result, Err(AppError::PathNotFound(_))));
    }
}
