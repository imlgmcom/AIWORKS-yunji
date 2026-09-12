// 用户管理命令（需要"用户管理"功能权限）
use serde::Deserialize;
use tauri::State;

use crate::auth::{hash_password, require_feature, FEATURE_USER};
use crate::db::repository;
use crate::error::{AppError, CmdResult};
use crate::state::AppState;

#[tauri::command]
pub async fn list_users(state: State<'_, AppState>) -> CmdResult<Vec<repository::users::UserItem>> {
    require_feature(&state.db, &state.auth, FEATURE_USER).await?;
    Ok(repository::users::list_all(&state.db).await?)
}

#[derive(Debug, Deserialize)]
pub struct DeleteUsersPayload {
    pub ids: Vec<i64>,
}

#[tauri::command]
pub async fn delete_users(
    state: State<'_, AppState>,
    payload: DeleteUsersPayload,
) -> CmdResult<()> {
    let operator = require_feature(&state.db, &state.auth, FEATURE_USER).await?;
    let ids = &payload.ids;
    if ids.is_empty() {
        return Ok(());
    }
    // 不能删除自己
    if ids.contains(&operator.id) {
        return Err(AppError::BadRequest("不能删除当前登录的账户".into()));
    }

    // 不允许删除管理员账户
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT COUNT(*) FROM users WHERE id IN ({placeholders}) AND is_admin = 1");
    let mut q = sqlx::query_scalar::<_, i64>(&sql);
    for &id in ids {
        q = q.bind(id);
    }
    let admin_count = q.fetch_one(&state.db).await?;
    if admin_count > 0 {
        return Err(AppError::BadRequest("不能删除管理员账户".into()));
    }

    // 删除用户并清理头像文件
    let avatars = repository::users::delete_many(&state.db, ids).await?;
    let storage = state.storage.get().await;
    for path in avatars {
        let _ = storage.delete(&path).await;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordPayload {
    pub user_id: i64,
    pub new_password: String,
}

#[tauri::command]
pub async fn reset_user_password(
    state: State<'_, AppState>,
    payload: ResetPasswordPayload,
) -> CmdResult<()> {
    require_feature(&state.db, &state.auth, FEATURE_USER).await?;
    if !(6..=128).contains(&payload.new_password.len()) {
        return Err(AppError::BadRequest("密码长度需为 6-128 位".into()));
    }
    let exists = repository::users::find_by_id(&state.db, payload.user_id)
        .await?
        .is_some();
    if !exists {
        return Err(AppError::NotFound);
    }
    let hash = hash_password(&payload.new_password)?;
    repository::users::update_password(&state.db, payload.user_id, &hash).await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserPayload {
    pub user_id: i64,
    pub username: String,
    pub nickname: String,
    pub email: String,
    pub bio: String,
    pub permission_level: i64,
    /// 留空/不传表示不修改密码
    pub new_password: Option<String>,
}

/// 管理员编辑用户的全部信息
#[tauri::command]
pub async fn update_user(
    state: State<'_, AppState>,
    payload: AdminUpdateUserPayload,
) -> CmdResult<()> {
    let operator = require_feature(&state.db, &state.auth, FEATURE_USER).await?;

    let username = payload.username.trim();
    let nickname = payload.nickname.trim();
    let email = payload.email.trim();

    if username.len() < 2 || username.len() > 32 {
        return Err(AppError::BadRequest("用户名长度需为 2-32 个字符".into()));
    }
    if nickname.is_empty() || nickname.len() > 32 {
        return Err(AppError::BadRequest("昵称为空或过长（最多 32 字符）".into()));
    }
    if !email.is_empty() {
        if email.len() > 100 || !email.contains('@') {
            return Err(AppError::BadRequest("邮箱格式不正确".into()));
        }
    }
    if payload.bio.chars().count() > 500 {
        return Err(AppError::BadRequest("简介过长（最多 500 字符）".into()));
    }
    if let Some(pwd) = &payload.new_password {
        if !(6..=128).contains(&pwd.len()) {
            return Err(AppError::BadRequest("密码长度需为 6-128 位".into()));
        }
    }

    let target = repository::users::find_by_id(&state.db, payload.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 仅管理员可以编辑管理员账户（防止用户管理者越权）
    if target.is_admin && !operator.is_admin {
        return Err(AppError::Forbidden);
    }

    // 用户名唯一性（排除自身）
    if username != target.username {
        if let Some(other) = repository::users::find_by_username(&state.db, username).await? {
            if other.id != target.id {
                return Err(AppError::BadRequest("用户名已被占用".into()));
            }
        }
    }
    // 邮箱唯一性（排除自身）
    if !email.is_empty() && email != target.email {
        let dup: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE email = ? AND id != ?",
        )
        .bind(email)
        .bind(target.id)
        .fetch_one(&state.db)
        .await?;
        if dup > 0 {
            return Err(AppError::BadRequest("邮箱已被其他账户使用".into()));
        }
    }

    if target.is_admin {
        // 管理员账户：基础信息可改，权限恒为 5
        repository::users::admin_update_admin_profile(
            &state.db,
            target.id,
            username,
            nickname,
            email,
            &payload.bio,
        )
        .await?;
    } else {
        repository::users::admin_update(
            &state.db,
            target.id,
            username,
            nickname,
            email,
            &payload.bio,
            payload.permission_level,
        )
        .await?;
    }

    if let Some(pwd) = payload.new_password {
        if !pwd.is_empty() {
            let hash = hash_password(&pwd)?;
            repository::users::update_password(&state.db, target.id, &hash).await?;
        }
    }
    Ok(())
}
