// 应用全局状态
use std::sync::Arc;
use sqlx::SqlitePool;
use std::path::PathBuf;

use crate::auth::AuthState;
use crate::storage::{StorageRef, StorageState};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub auth: Arc<AuthState>,
    pub storage: Arc<StorageState>,
    pub data_dir: PathBuf,
    pub upload_base_dir: PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool, storage: StorageRef, data_dir: PathBuf) -> Self {
        // 通过 trait 方法获取上传根目录（local 后端返回 upload_dir；其他后端回退到 data_dir/uploads）
        let upload_base_dir = storage.upload_dir().to_path_buf();
        Self {
            db,
            auth: Arc::new(AuthState::new()),
            storage: Arc::new(StorageState::new(storage)),
            data_dir,
            upload_base_dir,
        }
    }

    pub async fn reset_storage(&self, storage: StorageRef) {
        self.storage.reset(storage).await;
    }
}
