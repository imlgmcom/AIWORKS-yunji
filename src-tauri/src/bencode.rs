// 简易 bencode 解析器，用于解析 .torrent 文件的文件列表
// 只解析 info 字段，不解析 piece hashes 等大字段，避免内存爆炸

use std::collections::BTreeMap;

/// bencode 值
#[derive(Debug, Clone)]
enum BValue {
    Integer(i64),
    Bytes(Vec<u8>),
    List(Vec<BValue>),
    Dict(BTreeMap<Vec<u8>, BValue>),
}

struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn parse_value(&mut self) -> Option<BValue> {
        match self.peek()? {
            b'i' => self.parse_integer(),
            b'l' => self.parse_list(),
            b'd' => self.parse_dict(),
            b'0'..=b'9' => self.parse_bytes(),
            _ => None,
        }
    }

    fn parse_integer(&mut self) -> Option<BValue> {
        if self.peek()? != b'i' {
            return None;
        }
        self.pos += 1;
        let start = self.pos;
        while self.peek()? != b'e' {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.data[start..self.pos]).ok()?;
        let n: i64 = s.parse().ok()?;
        self.pos += 1; // skip 'e'
        Some(BValue::Integer(n))
    }

    fn parse_bytes(&mut self) -> Option<BValue> {
        let start = self.pos;
        while self.peek()? != b':' {
            self.pos += 1;
        }
        let len_str = std::str::from_utf8(&self.data[start..self.pos]).ok()?;
        let len: usize = len_str.parse().ok()?;
        self.pos += 1; // skip ':'
        if self.pos + len > self.data.len() {
            return None;
        }
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Some(BValue::Bytes(bytes))
    }

    fn parse_list(&mut self) -> Option<BValue> {
        if self.peek()? != b'l' {
            return None;
        }
        self.pos += 1;
        let mut list = Vec::new();
        while self.peek()? != b'e' {
            list.push(self.parse_value()?);
        }
        self.pos += 1; // skip 'e'
        Some(BValue::List(list))
    }

    fn parse_dict(&mut self) -> Option<BValue> {
        if self.peek()? != b'd' {
            return None;
        }
        self.pos += 1;
        let mut dict = BTreeMap::new();
        while self.peek()? != b'e' {
            let key = match self.parse_bytes()? {
                BValue::Bytes(b) => b,
                _ => return None,
            };
            let value = self.parse_value()?;
            dict.insert(key, value);
        }
        self.pos += 1; // skip 'e'
        Some(BValue::Dict(dict))
    }
}

/// 解析后的文件信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TorrentFile {
    pub name: String,
    pub size: i64,
}

/// 从 .torrent 字节解析文件列表
///
/// 单文件 torrent: info = { name, length, ... }
/// 多文件 torrent: info = { name, files: [{ path: [...], length }, ...], ... }
pub fn parse_torrent_files(data: &[u8]) -> Vec<TorrentFile> {
    let mut parser = Parser::new(data);
    let root = match parser.parse_value() {
        Some(BValue::Dict(d)) => d,
        _ => return Vec::new(),
    };

    // 取 info 字典
    let info_key = b"info".to_vec();
    let info = match root.get(&info_key) {
        Some(BValue::Dict(d)) => d,
        _ => return Vec::new(),
    };

    // 取 name（用于单文件）
    let name = match info.get(&b"name".to_vec()) {
        Some(BValue::Bytes(b)) => bytes_to_string(b),
        _ => String::new(),
    };

    // 检查是否有 files（多文件）
    match info.get(&b"files".to_vec()) {
        Some(BValue::List(files)) => {
            // 多文件模式
            files
                .iter()
                .filter_map(|f| match f {
                    BValue::Dict(d) => {
                        let length = match d.get(&b"length".to_vec()) {
                            Some(BValue::Integer(n)) => *n,
                            _ => 0,
                        };
                        // path 是字符串列表，拼接成路径
                        let path_str = match d.get(&b"path".to_vec()) {
                            Some(BValue::List(parts)) => parts
                                .iter()
                                .filter_map(|p| match p {
                                    BValue::Bytes(b) => Some(bytes_to_string(b)),
                                    _ => None,
                                })
                                .collect::<Vec<_>>()
                                .join("/"),
                            _ => String::new(),
                        };
                        if path_str.is_empty() {
                            None
                        } else {
                            Some(TorrentFile {
                                name: path_str,
                                size: length,
                            })
                        }
                    }
                    _ => None,
                })
                .collect()
        }
        _ => {
            // 单文件模式
            let length = match info.get(&b"length".to_vec()) {
                Some(BValue::Integer(n)) => *n,
                _ => 0,
            };
            if name.is_empty() {
                Vec::new()
            } else {
                vec![TorrentFile {
                    name,
                    size: length,
                }]
            }
        }
    }
}

/// 将字节转为字符串，优先 UTF-8，失败则用 lossy
fn bytes_to_string(b: &[u8]) -> String {
    match std::str::from_utf8(b) {
        Ok(s) => s.to_string(),
        Err(_) => String::from_utf8_lossy(b).to_string(),
    }
}

// --- 从 .torrent 生成磁力链接（info_hash = SHA1(info 字典的原始 bencode 字节)）---

/// 跳过一个 bencode 值，返回值结束后的位置
fn skip_value(data: &[u8], mut pos: usize) -> Option<usize> {
    match *data.get(pos)? {
        b'i' => {
            pos += 1;
            while *data.get(pos)? != b'e' {
                pos += 1;
            }
            Some(pos + 1)
        }
        b'l' | b'd' => {
            let open = pos;
            pos += 1;
            // dict 的键是字节串，遍历时跳过键+值
            let is_key = data[open] == b'd';
            let mut expecting_key = true;
            while *data.get(pos)? != b'e' {
                if is_key {
                    if expecting_key {
                        pos = skip_bytes(data, pos)?;
                        expecting_key = false;
                    } else {
                        pos = skip_value(data, pos)?;
                        expecting_key = true;
                    }
                } else {
                    pos = skip_value(data, pos)?;
                }
            }
            Some(pos + 1)
        }
        b'0'..=b'9' => skip_bytes(data, pos),
        _ => None,
    }
}

/// 跳过一个 bencode 字节串，返回结束位置
fn skip_bytes(data: &[u8], mut pos: usize) -> Option<usize> {
    let start = pos;
    while *data.get(pos)? != b':' {
        pos += 1;
    }
    let len: usize = std::str::from_utf8(data.get(start..pos)?).ok()?.parse().ok()?;
    Some(pos + 1 + len)
}

/// 计算种子的 info_hash（对 info 字典的**原始字节**做 SHA-1），返回 40 位十六进制
pub fn torrent_info_hash(data: &[u8]) -> Option<String> {
    if data.first()? != &b'd' {
        return None;
    }
    let mut pos = 1;
    while *data.get(pos)? != b'e' {
        // 读取键（字节串）
        let key_start = pos;
        let mut colon = pos;
        while *data.get(colon)? != b':' {
            colon += 1;
        }
        let key_len: usize = std::str::from_utf8(data.get(key_start..colon)?)
            .ok()?
            .parse()
            .ok()?;
        let value_start = colon + 1 + key_len;
        let key_bytes = data.get(colon + 1..value_start)?;
        // 跳过值
        let value_end = skip_value(data, value_start)?;
        if key_bytes == b"info" {
            return Some(sha1_hex(data.get(value_start..value_end)?));
        }
        pos = value_end;
    }
    None
}

/// 从种子字节构建磁力链接：magnet:?xt=urn:btih:<hash>&dn=<名称>&tr=<tracker...>
pub fn build_magnet_link(data: &[u8]) -> Option<String> {
    let hash = torrent_info_hash(data)?;

    let mut parser = Parser::new(data);
    let root = match parser.parse_value() {
        Some(BValue::Dict(d)) => d,
        _ => return None,
    };
    let info = match root.get(&b"info".to_vec()) {
        Some(BValue::Dict(d)) => d,
        _ => return None,
    };
    let name = match info.get(&b"name".to_vec()) {
        Some(BValue::Bytes(b)) => bytes_to_string(b),
        _ => String::new(),
    };

    let mut trackers: Vec<String> = Vec::new();
    if let Some(BValue::Bytes(b)) = root.get(&b"announce".to_vec()) {
        trackers.push(bytes_to_string(b));
    }
    if let Some(BValue::List(tiers)) = root.get(&b"announce-list".to_vec()) {
        for tier in tiers {
            if let BValue::List(urls) = tier {
                for u in urls {
                    if let BValue::Bytes(b) = u {
                        trackers.push(bytes_to_string(b));
                    }
                }
            }
        }
    }

    let mut magnet = format!("magnet:?xt=urn:btih:{}", hash);
    if !name.trim().is_empty() {
        magnet.push_str("&dn=");
        magnet.push_str(&urlencoding::encode(&name));
    }
    let mut seen = std::collections::HashSet::new();
    for tr in trackers {
        let tr = tr.trim().to_string();
        if tr.is_empty() || !seen.insert(tr.clone()) {
            continue;
        }
        magnet.push_str("&tr=");
        magnet.push_str(&urlencoding::encode(&tr));
    }
    Some(magnet)
}

/// SHA-1（FIPS 180-4），返回 40 位小写十六进制
fn sha1_hex(msg: &[u8]) -> String {
    const H0: [u32; 5] = [
        0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0,
    ];
    let mut h = H0;

    // 预处理填充
    let bit_len = (msg.len() as u64).wrapping_mul(8);
    let mut padded = msg.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let tmp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = tmp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    h.iter().map(|v| format!("{:08x}", v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_known_vectors() {
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            sha1_hex(b"abc"),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        // 跨分组（FIPS 标准向量，56 字节）
        let msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            sha1_hex(msg),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
    }

    #[test]
    fn info_hash_of_synthetic_torrent() {
        // 手工构造一个最小单文件种子（key 按字典序）
        let info = b"d6:lengthi12345e4:name8:test.iso12:piece lengthi16384e6:pieces20:0123456789abcdefghije";
        let expected = sha1_hex(info);
        let mut torrent = b"d8:announce19:http://tracker/x:80".to_vec();
        torrent.extend_from_slice(b"4:info");
        torrent.extend_from_slice(info);
        torrent.push(b'e');

        let hash = torrent_info_hash(&torrent).expect("应解析出 info_hash");
        assert_eq!(hash, expected);
        assert_eq!(hash.len(), 40);

        let magnet = build_magnet_link(&torrent).expect("应生成磁力链接");
        assert!(magnet.starts_with(&format!("magnet:?xt=urn:btih:{}", expected)));
        assert!(magnet.contains("&dn=test.iso"));
        assert!(magnet.contains("&tr=http%3A%2F%2Ftracker%2Fx%3A80"));
    }

    #[test]
    fn info_hash_with_announce_list() {
        let info = b"d4:name5:a.iso6:lengthi1e12:piece lengthi1e6:pieces1:xe";
        let expected = sha1_hex(info);
        let mut t = b"d13:announce-listll11:udp://h1:80el11:udp://h2:80ee".to_vec();
        t.extend_from_slice(b"4:info");
        t.extend_from_slice(info);
        t.push(b'e');
        let magnet = build_magnet_link(&t).unwrap();
        assert!(magnet.contains(&format!("xt=urn:btih:{}", expected)));
        assert!(magnet.contains("tr=udp%3A%2F%2Fh1%3A80"));
        assert!(magnet.contains("tr=udp%3A%2F%2Fh2%3A80"));
    }
}
