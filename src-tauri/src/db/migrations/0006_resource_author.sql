-- 0006：资源作者（关联 users.id；用户删除时不级联，作者显示为空）
ALTER TABLE resources ADD COLUMN user_id INTEGER;
CREATE INDEX IF NOT EXISTS idx_resources_user ON resources(user_id);
