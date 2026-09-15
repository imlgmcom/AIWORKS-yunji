// 功能权限与阅读权限
//
// 功能权限（管理操作）：settings 中的 perm_* 阈值（0-5，默认 5），
//   管理员恒通过；其他用户需 permission_level >= 阈值；阈值 0 = 任何登录用户。
// 阅读权限（资源）：resources.read_level，
//   0 = 公开（无需登录）；>0 需读者等级 >= read_level，或凭访问密码已解锁。
use sqlx::SqlitePool;

use crate::db::repository;
use crate::error::{AppError, CmdResult};
use crate::auth::state::{AuthState, UserSession};

/// 功能权限设置键
/// 文章/合集拆分为两个维度：
///   - *_own：新建、编辑、删除「自己的」内容
///   - 无后缀（manage）：管理「他人的」内容（编辑/删除/批量操作）
pub const FEATURE_ARTICLE_OWN: &str = "perm_article_own";
pub const FEATURE_ARTICLE: &str = "perm_article";
pub const FEATURE_COLLECTION_OWN: &str = "perm_collection_own";
pub const FEATURE_COLLECTION: &str = "perm_collection";
pub const FEATURE_TAG: &str = "perm_tag";
pub const FEATURE_TRASH: &str = "perm_trash";
pub const FEATURE_USER: &str = "perm_user";

/// 当前访问者的阅读等级：未登录 = 0
pub async fn viewer_level(auth: &AuthState) -> i64 {
    auth.current().await.map(|u| u.effective_level()).unwrap_or(0)
}

/// 读取某功能的权限阈值（默认 5，即仅管理员）
async fn feature_threshold(pool: &SqlitePool, key: &str) -> Result<i64, AppError> {
    let raw = repository::settings::get(pool, key).await?;
    Ok(match raw.as_deref() {
        Some(v) => v.parse::<i64>().unwrap_or(5).clamp(0, 5),
        None => 5,
    })
}

/// 要求当前用户满足某功能权限，返回其会话
pub async fn require_feature(
    pool: &SqlitePool,
    auth: &AuthState,
    key: &str,
) -> Result<UserSession, AppError> {
    let user = crate::auth::require_login(auth).await?;
    if user.is_admin {
        return Ok(user);
    }
    let threshold = feature_threshold(pool, key).await?;
    if user.effective_level() >= threshold {
        Ok(user)
    } else {
        Err(AppError::Forbidden)
    }
}

/// 确保当前访问者可以阅读指定资源，否则 Forbidden
pub async fn ensure_readable(
    auth: &AuthState,
    resource: &repository::resources::Resource,
) -> CmdResult<()> {
    // 公开资源
    if resource.read_level <= 0 {
        return Ok(());
    }
    let session = match auth.current().await {
        Some(s) => s,
        None => return Err(AppError::Forbidden),
    };
    // 等级足够（管理员恒足够）
    if session.effective_level() >= resource.read_level {
        return Ok(());
    }
    // 已通过访问密码解锁
    if !resource.access_password.is_empty()
        && auth.is_granted_resource(resource.id).await
    {
        return Ok(());
    }
    Err(AppError::Forbidden)
}
