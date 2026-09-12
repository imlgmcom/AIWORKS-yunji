-- 为 collection_images 表添加 width/height 列（与 images 表保持一致）
ALTER TABLE collection_images ADD COLUMN width INTEGER NOT NULL DEFAULT 0;
ALTER TABLE collection_images ADD COLUMN height INTEGER NOT NULL DEFAULT 0;
