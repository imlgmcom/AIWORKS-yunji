-- 分类表（树形结构，通过 parent_id 实现多级子分类）
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER,
    name TEXT NOT NULL,
    slug TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id, sort_order);
CREATE INDEX IF NOT EXISTS idx_categories_deleted ON categories(deleted_at);

-- 资源-分类关联（一个资源只能属于一个分类）
CREATE TABLE IF NOT EXISTS resource_categories (
    resource_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (resource_id),
    FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_rc_cat ON resource_categories(category_id);
