// 标签 Repository
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub count: i64,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<Tag>, AppError> {
    let rows = sqlx::query_as::<_, (i64, String, i64)>(
        "SELECT t.id, t.name, COUNT(rt.resource_id) as count \
         FROM tags t LEFT JOIN resource_tags rt ON rt.tag_id = t.id \
         GROUP BY t.id, t.name ORDER BY t.name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, count)| Tag { id, name, count })
        .collect())
}

pub async fn create(pool: &SqlitePool, name: &str) -> Result<i64, AppError> {
    let result = sqlx::query("INSERT INTO tags (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(result.last_insert_rowid())
}

pub async fn rename(pool: &SqlitePool, tid: i64, name: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE tags SET name=? WHERE id=?")
        .bind(name)
        .bind(tid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, tid: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tags WHERE id=?")
        .bind(tid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn merge(pool: &SqlitePool, source_ids: &[i64], target_id: i64) -> Result<(), AppError> {
    for &sid in source_ids {
        if sid == target_id {
            continue;
        }
        // 将源标签的资源关联迁移到目标标签
        sqlx::query(
            "INSERT OR IGNORE INTO resource_tags (resource_id, tag_id) \
             SELECT resource_id, ? FROM resource_tags WHERE tag_id = ?",
        )
        .bind(target_id)
        .bind(sid)
        .execute(pool)
        .await?;
        sqlx::query("DELETE FROM resource_tags WHERE tag_id = ?")
            .bind(sid)
            .execute(pool)
            .await?;
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(sid)
            .execute(pool)
            .await?;
    }
    Ok(())
}
