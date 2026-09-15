-- 合集归属作者：用于区分"编辑自己的合集"与"管理他人合集"的权限
ALTER TABLE collections ADD COLUMN user_id INTEGER;
CREATE INDEX IF NOT EXISTS idx_collections_user ON collections(user_id);

-- 历史合集归给最早的用户（避免无人负责）
UPDATE collections SET user_id = (SELECT MIN(id) FROM users)
 WHERE user_id IS NULL AND (SELECT COUNT(*) FROM users) > 0;
