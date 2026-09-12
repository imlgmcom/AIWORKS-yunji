// 本地磁盘存储后端
use async_trait::async_trait;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::backend::StorageBackend;

pub struct LocalStorage {
    pub upload_dir: PathBuf,
    pub url_prefix: String,
}

impl LocalStorage {
    pub fn new(upload_dir: impl Into<PathBuf>, url_prefix: &str) -> Self {
        let upload_dir = upload_dir.into();
        // 使用同步 std::fs 在构造时确保目录存在（构造函数非 async）
        std::fs::create_dir_all(&upload_dir).ok();
        Self {
            upload_dir,
            url_prefix: url_prefix.trim_end_matches('/').to_string(),
        }
    }

    fn _full_path(&self, rel_path: &str) -> anyhow::Result<PathBuf> {
        let full = self.upload_dir.join(rel_path);
        let canonical = full.canonicalize().unwrap_or_else(|_| full.clone());
        let base = self.upload_dir.canonicalize().unwrap_or_else(|_| self.upload_dir.clone());
        if !canonical.starts_with(&base) {
            anyhow::bail!("非法存储路径: {}", rel_path);
        }
        Ok(full)
    }

    fn _unique_name(&self, filename: &str) -> String {
        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("dat");
        let ts = chrono::Local::now().format("%Y%m%d%H%M%S");
        let rand: u32 = rand::random();
        format!("{}_{}.{}", ts, rand, ext)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn save(&self, data: Bytes, filename: &str, subdir: &str) -> anyhow::Result<String> {
        let unique = self._unique_name(filename);
        let rel = if subdir.is_empty() {
            unique.clone()
        } else {
            format!("{}/{}", subdir, unique)
        };
        let full = self.upload_dir.join(&rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&full, &data).await?;
        Ok(rel.replace('\\', "/"))
    }

    async fn read(&self, path: &str) -> anyhow::Result<Bytes> {
        let full = self._full_path(path)?;
        let data = fs::read(&full).await?;
        Ok(Bytes::from(data))
    }

    async fn delete(&self, path: &str) -> anyhow::Result<bool> {
        let full = self._full_path(path)?;
        match fs::remove_file(&full).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn get_url(&self, path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }
        format!("{}/{}", self.url_prefix, path)
    }

    fn upload_dir(&self) -> &std::path::Path {
        &self.upload_dir
    }
}
