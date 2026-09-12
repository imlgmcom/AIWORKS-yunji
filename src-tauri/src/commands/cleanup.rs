// 附件清理：扫描磁盘上未被任何数据库记录/正文引用的孤儿文件，由管理员确认后批量删除
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};
use std::time::UNIX_EPOCH;

use serde::Serialize;
use tauri::State;

use crate::auth::require_admin;
use crate::error::CmdResult;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct OrphanFile {
    /// 相对存储根的路径（正斜杠），与数据库中存储格式一致
    pub path: String,
    pub name: String,
    /// thumbnail | gallery | content | file | torrent | magnet | avatar | other
    pub category: &'static str,
    pub size: u64,
    /// 最后修改时间（Unix 秒）
    pub modified: i64,
    pub is_image: bool,
}

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub files: Vec<OrphanFile>,
    pub total_size: u64,
}

#[derive(Debug, Serialize)]
pub struct FailedFile {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub deleted: Vec<String>,
    pub failed: Vec<FailedFile>,
}

/// 汇总数据库中所有对存储文件的引用（含回收站中的文章/合集，可恢复因此不算孤儿）
async fn collect_referenced(state: &State<'_, AppState>) -> Result<HashSet<String>, crate::error::AppError> {
    let mut refs: HashSet<String> = HashSet::new();

    macro_rules! collect_paths {
        ($sql:expr) => {{
            let rows: Vec<String> = sqlx::query_scalar($sql).fetch_all(&state.db).await?;
            for p in rows {
                let p = normalize_rel(&p);
                if !p.is_empty() {
                    refs.insert(p);
                }
            }
        }};
    }

    // 缩略图 / 正文（含已软删文章）
    collect_paths!("SELECT thumbnail_path FROM resources WHERE thumbnail_path<>''");
    // 附件项（种子/文件）
    collect_paths!("SELECT file_path FROM resource_items WHERE file_path<>''");
    // 图集
    collect_paths!("SELECT file_path FROM images WHERE file_path<>''");
    collect_paths!("SELECT file_path FROM collection_images WHERE file_path<>''");
    // 合集缩略图（含已软删合集）
    collect_paths!("SELECT thumbnail_path FROM collections WHERE thumbnail_path<>''");
    // 用户头像
    collect_paths!("SELECT avatar_path FROM users WHERE avatar_path<>''");

    // 文章正文中以 Markdown/HTML 引用的相对路径
    let contents: Vec<String> =
        sqlx::query_scalar("SELECT content FROM resources").fetch_all(&state.db).await?;
    for text in &contents {
        extract_content_refs(text, &mut refs);
    }
    refs.remove("");
    Ok(refs)
}

/// 从 Markdown/HTML 正文中提取存储相对路径（含 percent-decoded 形式）
fn extract_content_refs(text: &str, out: &mut HashSet<String>) {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(
            r#"(?:thumbnails|gallery|content|files|torrents|magnets|avatars)/[^\s"'()<>\\\]]*"#,
        )
        .expect("valid regex")
    });
    for m in re.find_iter(text) {
        let raw = m.as_str().trim_end_matches(['.', ',', ';', '!', '?', ':']);
        out.insert(normalize_rel(raw));
        if let Ok(decoded) = urlencoding::decode(raw) {
            out.insert(normalize_rel(&decoded));
        }
    }
}

/// 归一化为数据库/磁盘统一使用的相对路径格式（正斜杠、去前导分隔符）
fn normalize_rel(p: &str) -> String {
    p.replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}

fn category_of(rel: &str) -> &'static str {
    match rel.split('/').next().unwrap_or("") {
        "thumbnails" => "thumbnail",
        "gallery" => "gallery",
        "content" => "content",
        "files" => "file",
        "torrents" => "torrent",
        "magnets" => "magnet",
        "avatars" => "avatar",
        _ => "other",
    }
}

fn is_image_name(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg"
            )
        })
}

/// 多线程递归遍历：目录任务队列 + 活跃 worker 计数（Condvar 防止队列暂时为空时提前退出）
fn walk_files(root: &Path) -> Vec<(String, u64, i64)> {
    struct Queue {
        dirs: VecDeque<PathBuf>,
        active: usize,
    }
    let state = Mutex::new(Queue {
        dirs: VecDeque::from([root.to_path_buf()]),
        active: 0,
    });
    let cv = Condvar::new();
    let found = Mutex::new(Vec::new());

    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, 8);

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let dir = {
                    let mut q = state.lock().unwrap();
                    loop {
                        if let Some(d) = q.dirs.pop_front() {
                            q.active += 1;
                            break d;
                        } else if q.active > 0 {
                            q = cv.wait(q).unwrap();
                        } else {
                            // 队列空且无 worker 在处理：全部结束，唤醒其他 worker 退出
                            cv.notify_all();
                            return;
                        }
                    }
                };

                let mut subdirs = Vec::new();
                let mut local = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for ent in entries.flatten() {
                        let path = ent.path();
                        let meta = match std::fs::metadata(&path) {
                            Ok(m) => m,
                            Err(_) => continue,
                        };
                        if meta.is_dir() {
                            subdirs.push(path);
                        } else if meta.is_file() {
                            if let Ok(rel) = path.strip_prefix(root) {
                                let rel = rel.to_string_lossy().replace('\\', "/");
                                let mtime = meta
                                    .modified()
                                    .ok()
                                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs() as i64)
                                    .unwrap_or(0);
                                local.push((rel, meta.len(), mtime));
                            }
                        }
                    }
                }

                {
                    let mut q = state.lock().unwrap();
                    q.dirs.extend(subdirs);
                    q.active -= 1;
                    found.lock().unwrap().append(&mut local);
                    cv.notify_all();
                }
            });
        }
    });

    found.into_inner().unwrap()
}

/// 扫描存储目录中的孤儿文件（磁盘有文件、但数据库任何地方都不再引用）
#[tauri::command]
pub async fn scan_orphan_files(state: State<'_, AppState>) -> CmdResult<ScanResult> {
    require_admin(&state.auth).await?;

    let referenced = collect_referenced(&state).await?;
    let root = {
        let storage = state.storage.get().await;
        storage.upload_dir().to_path_buf()
    };
    if !root.exists() {
        return Ok(ScanResult { files: vec![], total_size: 0 });
    }

    let walk_result = tauri::async_runtime::spawn_blocking(move || walk_files(&root))
        .await
        .map_err(|e| crate::error::AppError::BadRequest(format!("扫描任务失败: {}", e)))?;

    let mut files: Vec<OrphanFile> = walk_result
        .into_iter()
        .filter(|(rel, _, _)| !referenced.contains(rel))
        .map(|(rel, size, modified)| {
            let name = Path::new(&rel)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&rel)
                .to_string();
            let is_image = is_image_name(&name);
            OrphanFile {
                category: category_of(&rel),
                is_image,
                path: rel,
                name,
                size,
                modified,
            }
        })
        .collect();

    // 类别 → 修改时间倒序，方便排查
    files.sort_by(|a, b| a.category.cmp(b.category).then(b.modified.cmp(&a.modified)));
    let total_size = files.iter().map(|f| f.size).sum();
    Ok(ScanResult { files, total_size })
}

/// 判断相对路径是否安全（不允许绝对路径、盘符、跨目录跳转）
fn is_safe_rel(rel: &str) -> bool {
    if rel.is_empty() || rel.starts_with('/') || rel.contains(":\\") || rel.contains("..") {
        return false;
    }
    !Path::new(rel).is_absolute()
}

/// 删除管理员选定的孤儿文件；删除前会重新校验引用关系，避免删掉扫描后新产生的引用
#[tauri::command]
pub async fn delete_orphan_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> CmdResult<DeleteResult> {
    require_admin(&state.auth).await?;

    let referenced = collect_referenced(&state).await?;
    let storage = state.storage.get().await;

    let mut candidates: Vec<String> = Vec::new();
    let mut skipped: Vec<FailedFile> = Vec::new();
    for raw in paths {
        let rel = normalize_rel(&raw);
        if !is_safe_rel(&rel) {
            skipped.push(FailedFile { path: raw, error: "非法路径".into() });
            continue;
        }
        if referenced.contains(&rel) {
            skipped.push(FailedFile { path: raw, error: "文件已被重新引用，已跳过".into() });
            continue;
        }
        candidates.push(rel);
    }

    let mut deleted: Vec<String> = Vec::new();
    let mut failed = skipped;
    // 用 JoinSet 限制最多 8 个并发删除
    let mut set = tokio::task::JoinSet::new();
    for rel in candidates {
        let storage = storage.clone();
        set.spawn(async move {
            let r = storage.delete(&rel).await;
            (rel, r)
        });
        if set.len() >= 8 {
            if let Some(joined) = set.join_next().await {
                match joined {
                    Ok((rel, Ok(true))) => deleted.push(rel),
                    Ok((rel, Ok(false))) => {
                        failed.push(FailedFile { path: rel, error: "文件不存在".into() })
                    }
                    Ok((rel, Err(e))) => {
                        failed.push(FailedFile { path: rel, error: e.to_string() })
                    }
                    Err(e) => tracing::warn!("删除任务异常: {}", e),
                }
            }
        }
    }
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((rel, Ok(true))) => deleted.push(rel),
            Ok((rel, Ok(false))) => failed.push(FailedFile { path: rel, error: "文件不存在".into() }),
            Ok((rel, Err(e))) => failed.push(FailedFile { path: rel, error: e.to_string() }),
            Err(e) => tracing::warn!("删除任务异常: {}", e),
        }
    }

    // 清理空目录（不报错）
    let root = storage.upload_dir().to_path_buf();
    tauri::async_runtime::spawn_blocking(move || remove_empty_dirs(&root))
        .await
        .ok();

    Ok(DeleteResult { deleted, failed })
}

/// 自底向上删除空目录
fn remove_empty_dirs(root: &Path) {
    let mut dirs = Vec::new();
    fn collect(root: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(root) {
            for ent in entries.flatten() {
                if ent.path().is_dir() {
                    collect(&ent.path(), out);
                    out.push(ent.path());
                }
            }
        }
    }
    collect(root, &mut dirs);
    for d in dirs {
        let _ = std::fs::remove_dir(&d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_refs_from_markdown() {
        let md = "# 标题\n\
                  ![封面](thumbnails/20260101_abc.jpg)\n\
                  见 [附件](files/20260102_x.zip)。\n\
                  <img src=\"content/20260103_y.png\">\n\
                  外链不算：[u](https://a.com/files/x.png)\n\
                  句号在括号外：[a](content/20260104_z.txt).";
        let mut refs = HashSet::new();
        extract_content_refs(md, &mut refs);
        assert!(refs.contains("thumbnails/20260101_abc.jpg"));
        assert!(refs.contains("files/20260102_x.zip"));
        assert!(refs.contains("content/20260103_y.png"));
        assert!(refs.contains("content/20260104_z.txt"));
        assert!(!refs.iter().any(|r| r.contains("a.com")));
    }

    #[test]
    fn content_refs_percent_encoded() {
        let md = "![a](content/my%20file%20%5B1%5D.png)";
        let mut refs = HashSet::new();
        extract_content_refs(md, &mut refs);
        assert!(refs.contains("content/my file [1].png"));
        assert!(refs.contains("content/my%20file%20%5B1%5D.png"));
    }

    #[test]
    fn unsafe_rel_rejected() {
        assert!(!is_safe_rel("../secret.txt"));
        assert!(!is_safe_rel("/abs/a.png"));
        assert!(!is_safe_rel("C:\\Windows\\a.png"));
        assert!(is_safe_rel("content/2026_x.png"));
    }
}
