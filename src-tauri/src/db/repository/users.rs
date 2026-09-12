// 用户 Repository
use sqlx::SqlitePool;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub avatar_path: String,
    pub nickname: String,
    pub bio: String,
    pub is_admin: bool,
    pub permission_level: i64,
}

// 列表页使用的完整用户信息（不含密码哈希）
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserItem {
    pub id: i64,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub bio: String,
    pub avatar_path: String,
    pub is_admin: bool,
    pub permission_level: i64,
    pub created_at: String,
}

type UserRow = (i64, String, String, String, String, String, String, i64, i64);

const USER_COLS: &str =
    "id, username, password_hash, email, avatar_path, nickname, bio, is_admin, permission_level";

fn row_to_user(row: UserRow) -> User {
    User {
        id: row.0,
        username: row.1,
        password_hash: row.2,
        email: row.3,
        avatar_path: row.4,
        nickname: row.5,
        bio: row.6,
        is_admin: row.7 != 0,
        permission_level: row.8,
    }
}

pub async fn count(pool: &SqlitePool) -> Result<i64, AppError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn find_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, AppError> {
    let row = sqlx::query_as::<_, UserRow>(&format!(
        "SELECT {USER_COLS} FROM users WHERE username = ?"
    ))
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(row_to_user))
}

pub async fn find_by_id(pool: &SqlitePool, uid: i64) -> Result<Option<User>, AppError> {
    let row = sqlx::query_as::<_, UserRow>(&format!("SELECT {USER_COLS} FROM users WHERE id = ?"))
        .bind(uid)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(row_to_user))
}

// 注册：用户名/邮箱唯一性检查
pub async fn username_exists(pool: &SqlitePool, username: &str) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

pub async fn email_exists(pool: &SqlitePool, email: &str) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ? AND email != ''")
        .bind(email)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

/// 创建用户。返回 (id, is_admin)：当用户数为 0 时首个注册/创建的用户自动成为管理员
pub async fn create(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
    nickname: &str,
    email: &str,
) -> Result<(i64, bool), AppError> {
    let total = count(pool).await?;
    let is_admin = total == 0;
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, nickname, email, is_admin, permission_level) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(username)
    .bind(password_hash)
    .bind(nickname)
    .bind(email)
    .bind(if is_admin { 1 } else { 0 })
    .bind(5_i64)
    .execute(pool)
    .await?;
    Ok((result.last_insert_rowid(), is_admin))
}

// 管理员创建首个用户（启动引导）
pub async fn create_admin(pool: &SqlitePool, username: &str, password_hash: &str) -> Result<i64, AppError> {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, nickname, is_admin, permission_level) \
         VALUES (?, ?, ?, 1, 5)",
    )
    .bind(username)
    .bind(password_hash)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn update_password(pool: &SqlitePool, uid: i64, password_hash: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash=? WHERE id=?")
        .bind(password_hash)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

// 更新用户资料（昵称/邮箱/简介）
pub async fn update_profile(
    pool: &SqlitePool,
    uid: i64,
    nickname: &str,
    email: &str,
    bio: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET nickname=?, email=?, bio=? WHERE id=?")
        .bind(nickname)
        .bind(email)
        .bind(bio)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

// 更新用户名
pub async fn update_username(pool: &SqlitePool, uid: i64, username: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET username=? WHERE id=?")
        .bind(username)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

// 更新头像路径
pub async fn update_avatar(pool: &SqlitePool, uid: i64, avatar_path: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET avatar_path=? WHERE id=?")
        .bind(avatar_path)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

// 用户列表（管理页）
pub async fn list_all(pool: &SqlitePool) -> Result<Vec<UserItem>, AppError> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, String, i64, i64, String)>(
        "SELECT id, username, nickname, email, bio, avatar_path, is_admin, permission_level, created_at \
         FROM users ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, username, nickname, email, bio, avatar_path, is_admin, permission_level, created_at)| UserItem {
                id, username, nickname, email, bio, avatar_path,
                is_admin: is_admin != 0,
                permission_level,
                created_at,
            },
        )
        .collect())
}

// 批量删除用户，返回被删除用户的头像路径（用于清理文件）
pub async fn delete_many(pool: &SqlitePool, ids: &[i64]) -> Result<Vec<String>, AppError> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT avatar_path FROM users WHERE id IN ({placeholders}) AND avatar_path != ''");
    let mut q = sqlx::query_as::<_, (String,)>(&sql);
    for &id in ids {
        q = q.bind(id);
    }
    let avatars = q.fetch_all(pool).await?.into_iter().map(|(p,)| p).collect();

    let sql = format!("DELETE FROM users WHERE id IN ({placeholders})");
    let mut q = sqlx::query(&sql);
    for &id in ids {
        q = q.bind(id);
    }
    q.execute(pool).await?;
    Ok(avatars)
}

// 管理员编辑用户：用户名/昵称/邮箱/简介/权限值（密码单独更新）
pub async fn admin_update(
    pool: &SqlitePool,
    uid: i64,
    username: &str,
    nickname: &str,
    email: &str,
    bio: &str,
    permission_level: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET username=?, nickname=?, email=?, bio=?, permission_level=? WHERE id=? AND is_admin = 0",
    )
    .bind(username)
    .bind(nickname)
    .bind(email)
    .bind(bio)
    .bind(permission_level.clamp(0, 5))
    .bind(uid)
    .execute(pool)
    .await?;
    Ok(())
}

// 管理员编辑管理员账户：仅基础信息，权限恒为 5
pub async fn admin_update_admin_profile(
    pool: &SqlitePool,
    uid: i64,
    username: &str,
    nickname: &str,
    email: &str,
    bio: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET username=?, nickname=?, email=?, bio=? WHERE id=? AND is_admin = 1")
        .bind(username)
        .bind(nickname)
        .bind(email)
        .bind(bio)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}
