// 认证守卫：命令处理函数调用此函数检查登录状态
use crate::auth::state::{AuthState, UserSession};
use crate::error::AppError;

pub async fn require_login(auth: &AuthState) -> Result<UserSession, AppError> {
    auth.current()
        .await
        .ok_or(AppError::Unauthorized)
}

// 管理员守卫：要求已登录且 is_admin
pub async fn require_admin(auth: &AuthState) -> Result<UserSession, AppError> {
    let user = require_login(auth).await?;
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    Ok(user)
}
