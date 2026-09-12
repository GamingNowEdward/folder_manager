use serde::ser::Serializer;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("无法获取 exe 路径: {0}")]
    ExePath(String),
    #[error("无法获取 exe 目录")]
    ExeDir,
    #[error("无法获取数据目录: {0}")]
    AppDataDir(String),
    #[error("无法创建配置目录: {0}")]
    CreateConfigDir(String),
    #[error("读取配置文件失败: {0}")]
    ReadConfig(String),
    #[error("解析配置文件失败: {0}")]
    ParseConfig(String),
    #[error("配置版本不受支持: {0}")]
    UnsupportedVersion(u64),
    #[error("序列化配置失败: {0}")]
    Serialize(String),
    #[error("写入配置文件失败: {0}")]
    WriteConfig(String),
    #[error("备份配置文件失败: {0}")]
    BackupConfig(String),
    #[error("锁获取失败: {0}")]
    Lock(String),
    #[error("路径不存在: {0}")]
    PathNotFound(String),
    #[error("打开文件夹失败: {0}")]
    OpenFolder(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
