// 磁力链接下载 BT 文件：调用 aria2c 获取元数据
// 原项目用 libtorrent，这里用 aria2c 单文件工具，同样简单
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use once_cell::sync::Lazy;

use crate::bencode;
use crate::db::repository;
use crate::error::AppError;
use crate::state::AppState;

/// 任务状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct FetchStatus {
    pub status: String, // idle | pending | running | done | failed
    pub error: String,
    pub count: i64,
    pub torrent_path: String,
}

/// 全局任务状态表：key = "rid_itemId"
static TASK_STATUS: Lazy<Mutex<HashMap<String, FetchStatus>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn status_key(rid: i64, item_id: i64) -> String {
    format!("{}_{}", rid, item_id)
}

pub fn get_status(rid: i64, item_id: i64) -> FetchStatus {
    let key = status_key(rid, item_id);
    TASK_STATUS
        .lock()
        .unwrap()
        .get(&key)
        .cloned()
        .unwrap_or(FetchStatus {
            status: "idle".to_string(),
            error: String::new(),
            count: 0,
            torrent_path: String::new(),
        })
}

fn set_status(rid: i64, item_id: i64, status: &str, error: &str, count: i64, torrent_path: &str) {
    let key = status_key(rid, item_id);
    TASK_STATUS.lock().unwrap().insert(
        key,
        FetchStatus {
            status: status.to_string(),
            error: error.to_string(),
            count,
            torrent_path: torrent_path.to_string(),
        },
    );
}

/// 查找 aria2c 可执行文件路径
/// 优先 exe 同级 tools/aria2c.exe，其次系统 PATH
fn find_aria2c() -> Option<String> {
    // 1. exe 同级 tools/aria2c.exe
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("tools").join("aria2c.exe");
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    // 2. 开发模式：项目根的 tools/aria2c.exe
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dev_candidate = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap_or_else(|| std::path::Path::new(manifest_dir))
        .join("tools")
        .join("aria2c.exe");
    if dev_candidate.exists() {
        return Some(dev_candidate.to_string_lossy().to_string());
    }
    // 3. 系统 PATH
    which::which("aria2c").ok().map(|p| p.to_string_lossy().to_string())
}

/// 校验磁力链接格式
pub fn is_valid_magnet(content: &str) -> (bool, String) {
    let c = content.trim();
    if !c.starts_with("magnet:?") {
        return (false, "磁力链接格式错误：应以 magnet:? 开头".to_string());
    }
    let re = regex::Regex::new(r"[?&]xt=urn:btih:([a-fA-F0-9]{40}|[A-Za-z2-7]{32})").unwrap();
    if !re.is_match(c) {
        return (
            false,
            "磁力链接格式错误：缺少有效的 xt=urn:btih: 参数".to_string(),
        );
    }
    (true, String::new())
}

/// 从磁力链接提取 info_hash（十六进制小写），用于判断下载文件名是否就是 hash
fn extract_info_hash(magnet: &str) -> Option<String> {
    let re = regex::Regex::new(r"[?&]xt=urn:btih:([a-fA-F0-9]{40}|[A-Za-z2-7]{32})").ok()?;
    let caps = re.captures(magnet)?;
    let hash = caps.get(1)?.as_str();
    // 32 字符为 base32（RFC4648，无填充），aria2c 落盘时用的是 hex，需先转换
    if hash.len() == 32 {
        base32_btih_to_hex(hash)
    } else {
        Some(hash.to_lowercase())
    }
}

/// 将 32 字符 base32 的 BitTorrent info hash 转为 40 字符十六进制
fn base32_btih_to_hex(s: &str) -> Option<String> {
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    let mut out = Vec::with_capacity(20);
    for c in s.chars() {
        let v: u32 = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32,
            '2'..='7' => 26 + (c as u32 - '2' as u32),
            _ => return None,
        };
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    if out.len() != 20 {
        return None;
    }
    Some(out.iter().map(|b| format!("{:02x}", b)).collect())
}

/// 提交磁力获取任务（异步，立即返回）
pub fn submit(state: &AppState, rid: i64, item_id: i64) {
    let state = state.clone();
    set_status(rid, item_id, "pending", "", 0, "");
    tauri::async_runtime::spawn(async move {
        set_status(rid, item_id, "running", "", 0, "");
        match fetch(&state, rid, item_id).await {
            Ok(_) => {}
            Err(e) => {
                set_status(rid, item_id, "failed", &e.to_string(), 0, "");
            }
        }
    });
}

/// 实际执行磁力下载
async fn fetch(state: &AppState, rid: i64, item_id: i64) -> Result<(), AppError> {
    // 获取磁力链接
    let item = repository::resources::get_item(&state.db, rid, item_id).await?;
    let magnet_link = item.content.trim().to_string();
    if magnet_link.is_empty() {
        set_status(rid, item_id, "failed", "磁力种子项无磁力链接", 0, "");
        return Ok(());
    }

    // 校验格式
    let (ok, err) = is_valid_magnet(&magnet_link);
    if !ok {
        set_status(rid, item_id, "failed", &err, 0, "");
        return Ok(());
    }

    // 查找 aria2c
    let aria2c = find_aria2c().ok_or_else(|| {
        AppError::BadRequest(
            "未找到 aria2c，请将 aria2c.exe 放到应用目录的 tools/ 文件夹".to_string(),
        )
    })?;

    // 临时目录存放生成的 .torrent
    let tmp_dir = std::env::temp_dir().join(format!("yunji_magnet_{}_{}", rid, item_id));
    tokio::fs::create_dir_all(&tmp_dir).await.ok();

    let info_hash = extract_info_hash(&magnet_link).unwrap_or_default();

    // 调用 aria2c 下载元数据（Windows 下隐藏控制台窗口）
    let start = Instant::now();
    let mut cmd = std::process::Command::new(&aria2c);
    cmd.arg("--bt-metadata-only=true")
        .arg("--bt-save-metadata=true")
        .arg("--follow-torrent=false")
        .arg("--dir")
        .arg(&tmp_dir)
        .arg("--seed-time=0")
        .arg("--timeout=120")
        .arg(&magnet_link);
    // Windows: CREATE_NO_WINDOW = 0x08000000，隐藏控制台弹窗
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output = tokio::process::Command::from(cmd)
        .output()
        .await
        .map_err(|e| AppError::BadRequest(format!("aria2c 启动失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        set_status(
            rid,
            item_id,
            "failed",
            &format!("aria2c 执行失败: {}", stderr),
            0,
            "",
        );
        return Ok(());
    }

    // 找到生成的 .torrent 文件
    let torrent_file = find_generated_torrent(&tmp_dir, &info_hash).await;
    let torrent_path = match torrent_file {
        Some(p) => p,
        None => {
            set_status(rid, item_id, "failed", "未找到生成的 .torrent 文件", 0, "");
            return Ok(());
        }
    };

    // 读取 .torrent 内容
    let torrent_bytes = tokio::fs::read(&torrent_path)
        .await
        .map_err(AppError::Io)?;

    // 解析文件列表
    let files = bencode::parse_torrent_files(&torrent_bytes);
    let count = files.len() as i64;

    // aria2c 实际下载得到的临时文件名（通常为 <info_hash>.torrent）
    let gen_stem = torrent_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    // 决定最终 .torrent 文件名：
    // 1) 下载名就是 hash（aria2c 默认行为）→ 用磁力链接备注 dn；
    //    没有备注则用文章标题；再没有用兜底名
    // 2) 下载名本身是正常种子名 → 下载是什么名就存什么名
    let hash = extract_info_hash(&magnet_link).unwrap_or_default();
    let dn_name = extract_dn(&magnet_link).filter(|s| !s.trim().is_empty());
    let article_title = if dn_name.is_none() {
        repository::resources::get(&state.db, rid)
            .await
            .ok()
            .map(|r| r.title)
            .filter(|t| !t.trim().is_empty())
    } else {
        None
    };
    let base_name = if !hash.is_empty() && gen_stem.eq_ignore_ascii_case(&hash) {
        dn_name
            .or(article_title)
            .unwrap_or_else(|| format!("resource_{}", rid))
    } else if !gen_stem.is_empty() {
        gen_stem
    } else {
        format!("resource_{}_{}", rid, item_id)
    };
    let desired_name = format!("{}.torrent", sanitize_filename(&base_name));

    // 保存到 storage（同名自动追加 _N，区分同一文章的多个磁力链接）
    let storage = state.storage.get().await;
    let bytes = bytes::Bytes::from(torrent_bytes.clone());
    let saved_path = storage
        .save_named(bytes, &desired_name, "magnets")
        .await
        .map_err(AppError::Storage)?;

    // 前端展示名必须与物理保存的文件名完全一致
    let torrent_fname = std::path::Path::new(&saved_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&desired_name)
        .to_string();

    // 更新 resource_items
    repository::resources::update_item_torrent(
        &state.db,
        item_id,
        &saved_path,
        &torrent_fname,
    )
    .await?;

    // 缓存文件列表
    if count > 0 {
        let file_entries: Vec<(String, i64)> =
            files.iter().map(|f| (f.name.clone(), f.size)).collect();
        repository::torrent_files::save(&state.db, rid, item_id, &file_entries).await?;
    }

    // 清理临时目录
    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;

    let elapsed = start.elapsed().as_secs();
    let _ = elapsed;

    set_status(rid, item_id, "done", "", count, &saved_path);
    Ok(())
}

/// 在临时目录中查找生成的 .torrent 文件
async fn find_generated_torrent(dir: &std::path::Path, info_hash: &str) -> Option<std::path::PathBuf> {
    // 先按 info_hash 找
    if !info_hash.is_empty() {
        let p = dir.join(format!("{}.torrent", info_hash));
        if p.exists() {
            return Some(p);
        }
        // 也可能是大写
        let p = dir.join(format!("{}.torrent", info_hash.to_uppercase()));
        if p.exists() {
            return Some(p);
        }
    }
    // 遍历目录找 .torrent
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("torrent") {
                return Some(path);
            }
        }
    }
    None
}

/// 提取磁力链接 dn 参数（显示名/备注），URL 解码
fn extract_dn(magnet: &str) -> Option<String> {
    let re = regex::Regex::new(r"[?&]dn=([^&]+)").ok()?;
    let caps = re.captures(magnet)?;
    let encoded = caps.get(1)?.as_str();
    let decoded = percent_decode(encoded);
    if decoded.trim().is_empty() {
        None
    } else {
        Some(decoded)
    }
}

/// 清理文件名：Windows 非法字符 \ / : * ? " < > | 及控制字符替换为 _，
/// 去掉首尾空白与结尾的点/空格，并限制长度（含 .torrent 后缀不超过文件系统上限）
fn sanitize_filename(name: &str) -> String {
    const MAX_STEM_CHARS: usize = 120;
    let cleaned: String = name
        .trim()
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\u{0}'..='\u{1f}' => '_',
            c => c,
        })
        .take(MAX_STEM_CHARS)
        .collect();
    // Windows 不允许文件名以点或空格结尾
    let cleaned = cleaned.trim_end_matches(['.', ' ']).to_string();
    if cleaned.is_empty() {
        "torrent".to_string()
    } else {
        cleaned
    }
}

/// 简易 percent 解码（处理 dn 参数中的 %XX 编码）
fn percent_decode(s: &str) -> String {
    let mut result = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let h1 = chars[i + 1].to_digit(16);
            let h2 = chars[i + 2].to_digit(16);
            if let (Some(h1), Some(h2)) = (h1, h2) {
                result.push(((h1 << 4) | h2) as u8);
                i += 3;
                continue;
            }
        }
        if chars[i] == '+' {
            result.push(b' ');
        } else {
            // 简单处理：取 UTF-8 编码的第一个字节（dn 通常是 ASCII）
            let mut buf = [0u8; 4];
            let encoded = chars[i].encode_utf8(&mut buf);
            result.extend_from_slice(encoded.as_bytes());
        }
        i += 1;
    }
    String::from_utf8_lossy(&result).to_string()
}
