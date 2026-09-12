// 存储后端 trait
use async_trait::async_trait;
use bytes::Bytes;
use std::path::Path;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn save(&self, data: Bytes, filename: &str, subdir: &str) -> anyhow::Result<String>;

    /// 按调用方给定的文件名保存（不自动加时间戳/随机后缀）。
    /// 若同名文件已存在，自动在文件名尾部追加 _1、_2… 区分，返回相对存储路径。
    async fn save_named(
        &self,
        data: Bytes,
        filename: &str,
        subdir: &str,
    ) -> anyhow::Result<String> {
        let base_dir = self.upload_dir();
        let dir = if subdir.is_empty() {
            base_dir.to_path_buf()
        } else {
            base_dir.join(subdir.trim_matches('/'))
        };
        tokio::fs::create_dir_all(&dir).await?;

        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("file");
        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("torrent");

        let mut candidate = format!("{}.{}", stem, ext);
        let mut n: usize = 1;
        while dir.join(&candidate).exists() {
            candidate = format!("{}_{}.{}", stem, n, ext);
            n += 1;
        }

        tokio::fs::write(dir.join(&candidate), &data).await?;

        let rel = if subdir.is_empty() {
            candidate
        } else {
            format!("{}/{}", subdir.trim_matches('/'), candidate)
        };
        Ok(rel.replace('\\', "/"))
    }

    async fn read(&self, path: &str) -> anyhow::Result<Bytes>;
    async fn delete(&self, path: &str) -> anyhow::Result<bool>;
    fn get_url(&self, path: &str) -> String;
    /// 返回本地存储根目录（仅 local 后端有意义；其他后端返回空 Path）
    fn upload_dir(&self) -> &Path;
}
