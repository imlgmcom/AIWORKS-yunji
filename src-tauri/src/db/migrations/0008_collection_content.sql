-- 合集支持 Markdown 正文内容（像文章一样）
ALTER TABLE collections ADD COLUMN content TEXT NOT NULL DEFAULT '';
