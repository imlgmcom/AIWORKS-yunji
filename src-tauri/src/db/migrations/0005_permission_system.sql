-- 权限系统：
-- 1. users.permission_level：0-5，管理员（is_admin=1）恒为 5
-- 2. resources.read_level：0=公开（无需登录），>0 需要读者权限值 >= 阅读权限
-- 全新数据库已在 0001 中包含新列，ALTER 会失败，由调用方逐条容错处理
-- 注意：旧 visibility -> read_level 的数据回填由 db::pool::run_migrations 在
-- read_level 列刚添加时执行一次，不写在此处，避免每次启动覆盖用户已修改的 read_level

ALTER TABLE users ADD COLUMN permission_level INTEGER NOT NULL DEFAULT 0;

ALTER TABLE resources ADD COLUMN read_level INTEGER NOT NULL DEFAULT 0;
