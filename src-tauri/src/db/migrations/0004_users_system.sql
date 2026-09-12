-- 用户系统扩展：多用户注册 + 管理员标识 + 简介
-- 0001 已包含新列时（全新数据库）ALTER 会失败，由调用方容错处理

ALTER TABLE users ADD COLUMN is_admin INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN bio TEXT NOT NULL DEFAULT '';
