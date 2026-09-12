// 统一错误类型，转换为 Tauri 命令可返回的 String
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未登录")]
    Unauthorized,
    #[error("无权访问")]
    Forbidden,
    #[error("资源不存在")]
    NotFound,
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Storage(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> String {
        e.to_string()
    }
}

// Tauri 命令返回类型别名
pub type CmdResult<T> = Result<T, AppError>;
