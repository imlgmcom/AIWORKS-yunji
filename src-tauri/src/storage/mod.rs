pub mod backend;
pub mod local;

pub use backend::StorageBackend;
pub use local::LocalStorage;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::repository::settings;

pub type StorageRef = Arc<dyn StorageBackend>;

pub async fn get_storage(pool: &sqlx::SqlitePool, data_dir: &std::path::Path) -> anyhow::Result<StorageRef> {
    let upload_dir_name = settings::get(pool, "local_upload_dir")
        .await?
        .unwrap_or_else(|| "uploads".to_string());
    let url_prefix = settings::get(pool, "local_url_prefix")
        .await?
        .unwrap_or_else(|| "yunji-files".to_string());

    let upload_dir = if std::path::Path::new(&upload_dir_name).is_absolute() {
        std::path::PathBuf::from(&upload_dir_name)
    } else {
        data_dir.join(&upload_dir_name)
    };

    Ok(Arc::new(LocalStorage::new(upload_dir, &url_prefix)) as StorageRef)
}

pub struct StorageState {
    inner: RwLock<StorageRef>,
}

impl StorageState {
    pub fn new(storage: StorageRef) -> Self {
        Self {
            inner: RwLock::new(storage),
        }
    }

    pub async fn get(&self) -> StorageRef {
        self.inner.read().await.clone()
    }

    pub async fn reset(&self, storage: StorageRef) {
        *self.inner.write().await = storage;
    }
}
