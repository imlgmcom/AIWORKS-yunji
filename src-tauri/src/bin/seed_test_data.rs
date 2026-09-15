// 一次性测试数据种子（运行后可删除本文件）
// cargo run --bin seed_test_data          写入
// cargo run --bin seed_test_data -- reset 清除测试数据
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
use argon2::Argon2;
use sqlx::sqlite::SqlitePool;

const N: i64 = 80;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reset = std::env::args().nth(1).as_deref() == Some("reset");
    // 兼容从 src-tauri/ 或项目根目录运行
    let db_url = if std::path::Path::new("data/app.db").exists() {
        "sqlite:data/app.db"
    } else {
        "sqlite:../data/app.db"
    };
    let pool = SqlitePool::connect(db_url).await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    if reset {
        cleanup(&pool).await?;
        println!("测试数据已清除");
        return Ok(());
    }

    // 幂等：已存在则跳过
    let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = 'author01'")
        .fetch_one(&pool)
        .await?;
    if exists.0 > 0 {
        println!("测试数据已存在，跳过（清除请运行：cargo run --bin seed_test_data -- reset）");
        return Ok(());
    }

    // 所有测试账号统一密码 123456
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(b"123456", &salt)
        .unwrap()
        .to_string();

    // 1) 80 个作者账号
    let mut user_ids = Vec::new();
    for i in 1..=N {
        let username = format!("author{:02}", i);
        let nickname = format!("作者{:02}", i);
        let r = sqlx::query(
            "INSERT INTO users (username, password_hash, nickname, is_admin, permission_level) \
             VALUES (?1, ?2, ?3, 0, 0)",
        )
        .bind(&username)
        .bind(&password_hash)
        .bind(&nickname)
        .execute(&pool)
        .await?;
        user_ids.push(r.last_insert_rowid());
    }

    // 2) 80 篇只有标题的文章，每人一篇（read_level=0 公开，保证作者页可见）
    let mut resource_ids = Vec::new();
    for (idx, uid) in user_ids.iter().enumerate() {
        let i = (idx + 1) as i64;
        let title = format!("测试文章 {:03}", i);
        let modifier = format!("-{} minutes", N - i);
        let r = sqlx::query(
            "INSERT INTO resources (title, description, content, status, read_level, user_id, created_at, updated_at) \
             VALUES (?1, '', '', 'normal', 0, ?2, datetime('now', ?3), datetime('now', ?3))",
        )
        .bind(&title)
        .bind(uid)
        .bind(&modifier)
        .execute(&pool)
        .await?;
        resource_ids.push(r.last_insert_rowid());
    }

    // 3) 80 个标签，与文章一一关联（每标签 1 篇）
    let mut tag_ids = Vec::new();
    for i in 1..=N {
        let name = format!("测试标签 {:02}", i);
        let r = sqlx::query("INSERT INTO tags (name) VALUES (?1)")
            .bind(&name)
            .execute(&pool)
            .await?;
        tag_ids.push(r.last_insert_rowid());
    }
    for (rid, tid) in resource_ids.iter().zip(tag_ids.iter()) {
        sqlx::query("INSERT OR IGNORE INTO resource_tags (resource_id, tag_id) VALUES (?1, ?2)")
            .bind(rid)
            .bind(tid)
            .execute(&pool)
            .await?;
    }

    // 4) 80 个公开合集，归属对应作者，并各关联对应文章
    let mut collection_ids = Vec::new();
    for (idx, uid) in user_ids.iter().enumerate() {
        let i = (idx + 1) as i64;
        let title = format!("测试合集 {:02}", i);
        let modifier = format!("-{} minutes", N - i);
        let r = sqlx::query(
            "INSERT INTO collections (title, description, visibility, user_id, created_at, updated_at) \
             VALUES (?1, '', 'public', ?2, datetime('now', ?3), datetime('now', ?3))",
        )
        .bind(&title)
        .bind(uid)
        .bind(&modifier)
        .execute(&pool)
        .await?;
        collection_ids.push(r.last_insert_rowid());
    }
    for (idx, cid) in collection_ids.iter().enumerate() {
        sqlx::query(
            "INSERT OR IGNORE INTO resource_collections (resource_id, collection_id, sort_order) VALUES (?1, ?2, 0)",
        )
        .bind(resource_ids[idx])
        .bind(cid)
        .execute(&pool)
        .await?;
    }

    println!(
        "完成：{} 个作者（账号 author01..author80，密码 123456）、{} 篇文章、{} 个标签、{} 个合集",
        user_ids.len(),
        resource_ids.len(),
        tag_ids.len(),
        collection_ids.len()
    );
    Ok(())
}

async fn cleanup(pool: &SqlitePool) -> anyhow::Result<()> {
    // 先删关联内容（外键级联会处理 join 表，这里显式更稳妥）
    sqlx::query(
        "DELETE FROM resource_tags WHERE resource_id IN (SELECT id FROM resources WHERE title LIKE '测试文章 %')",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "DELETE FROM resource_collections WHERE resource_id IN (SELECT id FROM resources WHERE title LIKE '测试文章 %')",
    )
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM resources WHERE title LIKE '测试文章 %'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tags WHERE name LIKE '测试标签 %'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM collections WHERE title LIKE '测试合集 %'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM users WHERE username LIKE 'author%' AND is_admin = 0")
        .execute(pool)
        .await?;
    Ok(())
}
