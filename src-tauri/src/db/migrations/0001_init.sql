-- 云记数据库初始化 Schema
-- 全新设计，丢弃旧 Flask 版的遗留字段

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT '',
    avatar_path TEXT NOT NULL DEFAULT '',
    nickname TEXT NOT NULL DEFAULT '',
    bio TEXT NOT NULL DEFAULT '',
    is_admin INTEGER NOT NULL DEFAULT 0,
    permission_level INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 资源表（统一 article 类型，附件通过 resource_items 存储）
CREATE TABLE IF NOT EXISTS resources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'normal' CHECK (status IN ('normal','invalid','archived')),
    read_level INTEGER NOT NULL DEFAULT 0,
    access_password TEXT NOT NULL DEFAULT '',
    thumbnail_path TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_resources_created ON resources(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_resources_updated ON resources(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_resources_deleted ON resources(deleted_at);
CREATE INDEX IF NOT EXISTS idx_resources_read_level ON resources(read_level);
CREATE INDEX IF NOT EXISTS idx_resources_status ON resources(status);

-- 标签表
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 资源-标签关联
CREATE TABLE IF NOT EXISTS resource_tags (
    resource_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (resource_id, tag_id),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_resource_tags_tag ON resource_tags(tag_id);

-- 合集表
CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    visibility TEXT NOT NULL DEFAULT 'private' CHECK (visibility IN ('public','private')),
    access_password TEXT NOT NULL DEFAULT '',
    thumbnail_path TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_collections_deleted ON collections(deleted_at);
CREATE INDEX IF NOT EXISTS idx_collections_visibility ON collections(visibility);

-- 资源-合集关联
CREATE TABLE IF NOT EXISTS resource_collections (
    resource_id INTEGER NOT NULL,
    collection_id INTEGER NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (resource_id, collection_id),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE,
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_rc_collection ON resource_collections(collection_id, sort_order);

-- 资源附件项（link/direct_link/magnet_torrent/ed2k/file）
CREATE TABLE IF NOT EXISTS resource_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('link','direct_link','magnet_torrent','ed2k','file')),
    content TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    file_path TEXT NOT NULL DEFAULT '',
    file_name TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_resource_items_resource ON resource_items(resource_id, sort_order);

-- 资源图片（图集）
CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 0,
    height INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_images_resource ON images(resource_id, sort_order);

-- 合集图片
CREATE TABLE IF NOT EXISTS collection_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    collection_id INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 0,
    height INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_colimg_collection ON collection_images(collection_id, sort_order);

-- 资源文件缓存（磁力种子的文件列表，按 item_id 独立缓存）
CREATE TABLE IF NOT EXISTS resource_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    size INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_resource_files_item ON resource_files(resource_id, item_id, sort_order);

-- 访问授权（访客通过密码访问私密资源/合集）
CREATE TABLE IF NOT EXISTS access_grants (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_token TEXT NOT NULL,
    resource_id INTEGER,
    collection_id INTEGER,
    granted_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL DEFAULT (datetime('now', '+30 days'))
);
CREATE INDEX IF NOT EXISTS idx_grants_session ON access_grants(session_token);
CREATE INDEX IF NOT EXISTS idx_grants_expires ON access_grants(expires_at);

-- 设置表（key-value）
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- 默认设置
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('site_name', '云记'),
    ('local_upload_dir', 'uploads'),
    ('local_url_prefix', 'yunji-files');
