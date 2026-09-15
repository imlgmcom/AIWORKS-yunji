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

#[tauri::command]
pub async fn list_users_page(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
    search: Option<String>,
) -> CmdResult<repository::Paged<repository::users::UserItem>> {
    require_feature(&state.db, &state.auth, FEATURE_USER).await?;
    let (page, page_size) = repository::clamp_page(page, page_size);
    Ok(repository::users::list_all_paged(
        &state.db,
        page_size,
        (page - 1) * page_size,
        search.as_deref(),
    )
    .await?)
}

/// 作者列表（公开，只返回有对当前访问者可见资源的用户）
#[tauri::command]
pub async fn list_authors(
    state: State<'_, AppState>,
) -> CmdResult<Vec<repository::users::AuthorItem>> {
    let level = crate::auth::viewer_level(&state.auth).await;
    Ok(repository::users::list_authors(&state.db, level).await?)
}

/// 作者列表（分页，公开）
#[tauri::command]
pub async fn list_authors_page(
    state: State<'_, AppState>,
    page: i64,
    page_size: i64,
    search: Option<String>,
) -> CmdResult<repository::Paged<repository::users::AuthorItem>> {
    let level = crate::auth::viewer_level(&state.auth).await;
    let (page, page_size) = repository::clamp_page(page, page_size);
    Ok(repository::users::list_authors_paged(
        &state.db,
        level,
        page_size,
        (page - 1) * page_size,
        search.as_deref(),
    )
    .await?)
}

#[derive(Debug, Deserialize)]
pub struct AdminAvatarPayload {
    pub user_id: i64,
    pub ext: String,
    /// PNG/JPEG/WebP 图片字节
    pub data: Vec<u8>,
}

/// 管理员为任意账号设置头像（前端裁剪后的字节）
#[tauri::command]
pub async fn admin_set_user_avatar(
    state: State<'_, AppState>,
    payload: AdminAvatarPayload,
) -> CmdResult<String> {
    let operator = require_feature(&state.db, &state.auth, FEATURE_USER).await?;
    let target = repository::users::find_by_id(&state.db, payload.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    // 管理员账户的头像仅管理员可改（防止用户管理者越权）
    if target.is_admin && !operator.is_admin {
        return Err(AppError::Forbidden);
    }
    let ext = payload.ext.trim_start_matches('.').to_lowercase();
    if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") {
        return Err(AppError::BadRequest("仅支持 jpg/png/webp/gif 图片".into()));
    }
    if payload.data.is_empty() || payload.data.len() > 10 * 1024 * 1024 {
        return Err(AppError::BadRequest("图片为空或超过 10MB".into()));
    }

    let storage = state.storage.get().await;
    let filename = format!("avatar_{}.{}", target.id, ext);
    let saved = storage
        .save(bytes::Bytes::from(payload.data), &filename, "avatars")
        .await
        .map_err(AppError::Storage)?;

    if !target.avatar_path.is_empty() {
        let _ = storage.delete(&target.avatar_path).await;
    }
    repository::users::update_avatar(&state.db, target.id, &saved).await?;
    // 改的是自己：同步当前登录会话，避免界面头像停留旧值
    if target.id == operator.id {
        let path = saved.clone();
        state.auth.update_user(|s| s.avatar_path = path).await;
    }
    Ok(saved)
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

    // 编辑的是自己：同步当前登录会话中的资料
    if target.id == operator.id {
        let un = username.to_string();
        let nn = nickname.to_string();
        let em = email.to_string();
        let bio = payload.bio.clone();
        state.auth.update_user(|s| {
            s.username = un;
            s.nickname = nn;
            s.email = em;
            s.bio = bio;
        }).await;
    }
    Ok(())
}
