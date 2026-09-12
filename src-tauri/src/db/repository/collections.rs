// 合集 Repository
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub visibility: String,
    pub access_password: String,
    pub thumbnail_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub resource_count: i64,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Collection>, AppError> {
    let rows = sqlx::query_as::<
        _,
        (i64, String, String, String, String, String, String, String, Option<String>, i64),
    >(
        "SELECT c.id, c.title, c.description, c.visibility, c.access_password, c.thumbnail_path, \
         c.created_at, c.updated_at, c.deleted_at, \
         (SELECT COUNT(*) FROM resource_collections rc WHERE rc.collection_id = c.id) as resource_count \
         FROM collections c WHERE c.deleted_at IS NULL ORDER BY c.created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, title, description, visibility, access_password, thumbnail_path, created_at, updated_at, deleted_at, resource_count)| {
                Collection {
                    id, title, description, visibility, access_password, thumbnail_path,
                    created_at, updated_at, deleted_at, resource_count,
                }
            },
        )
        .collect())
}

pub async fn get(pool: &SqlitePool, cid: i64) -> Result<Collection, AppError> {
    let row = sqlx::query_as::<
        _,
        (i64, String, String, String, String, String, String, String, Option<String>, i64),
    >(
        "SELECT c.id, c.title, c.description, c.visibility, c.access_password, c.thumbnail_path, \
         c.created_at, c.updated_at, c.deleted_at, \
         (SELECT COUNT(*) FROM resource_collections rc WHERE rc.collection_id = c.id) as resource_count \
         FROM collections c WHERE c.id = ? AND c.deleted_at IS NULL",
    )
    .bind(cid)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or(AppError::NotFound)?;
    let (id, title, description, visibility, access_password, thumbnail_path, created_at, updated_at, deleted_at, resource_count) = row;

    Ok(Collection {
        id, title, description, visibility, access_password, thumbnail_path,
        created_at, updated_at, deleted_at, resource_count,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCollection {
    pub title: String,
    pub description: String,
    pub visibility: Option<String>,
    pub access_password: String,
    pub thumbnail_path: String,
}

pub async fn create(pool: &SqlitePool, payload: &CreateCollection) -> Result<i64, AppError> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let result = sqlx::query(
        "INSERT INTO collections (title, description, visibility, access_password, thumbnail_path, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.visibility.as_deref().unwrap_or("private"))
    .bind(&payload.access_password)
    .bind(&payload.thumbnail_path)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn update(pool: &SqlitePool, cid: i64, payload: &CreateCollection) -> Result<(), AppError> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query(
        "UPDATE collections SET title=?, description=?, visibility=?, access_password=?, thumbnail_path=?, updated_at=? \
         WHERE id=? AND deleted_at IS NULL",
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.visibility.as_deref().unwrap_or("private"))
    .bind(&payload.access_password)
    .bind(&payload.thumbnail_path)
    .bind(&now)
    .bind(cid)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn soft_delete(pool: &SqlitePool, cid: i64) -> Result<(), AppError> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    sqlx::query("UPDATE collections SET deleted_at=? WHERE id=? AND deleted_at IS NULL")
        .bind(&now)
        .bind(cid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn restore(pool: &SqlitePool, cid: i64) -> Result<(), AppError> {
    sqlx::query("UPDATE collections SET deleted_at=NULL WHERE id=?")
        .bind(cid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn permanent_delete(pool: &SqlitePool, cid: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM collections WHERE id=?")
        .bind(cid)
        .execute(pool)
        .await?;
    Ok(())
}

// --- 资源-合集关联 ---

pub async fn sync_resources(pool: &SqlitePool, cid: i64, resource_ids: &[i64]) -> Result<(), AppError> {
    sqlx::query("DELETE FROM resource_collections WHERE collection_id=?")
        .bind(cid)
        .execute(pool)
        .await?;
    for (i, &rid) in resource_ids.iter().enumerate() {
        sqlx::query("INSERT OR IGNORE INTO resource_collections (resource_id, collection_id, sort_order) VALUES (?, ?, ?)")
            .bind(rid)
            .bind(cid)
            .bind(i as i64)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn list_resource_ids(pool: &SqlitePool, cid: i64) -> Result<Vec<i64>, AppError> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT resource_id FROM resource_collections WHERE collection_id=? ORDER BY sort_order",
    )
    .bind(cid)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// 为资源同步合集关联（以资源的视角：该资源属于哪些合集）
pub async fn sync_resource_collections(
    pool: &SqlitePool,
    rid: i64,
    collection_ids: &[i64],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM resource_collections WHERE resource_id=?")
        .bind(rid)
        .execute(pool)
        .await?;
    for (i, &cid) in collection_ids.iter().enumerate() {
        sqlx::query("INSERT OR IGNORE INTO resource_collections (resource_id, collection_id, sort_order) VALUES (?, ?, ?)")
            .bind(rid)
            .bind(cid)
            .bind(i as i64)
            .execute(pool)
            .await?;
    }
    Ok(())
}
