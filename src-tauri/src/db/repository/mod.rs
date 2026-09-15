pub mod categories;
pub mod collections;
pub mod images;
pub mod resources;
pub mod settings;
pub mod tags;
pub mod torrent_files;
pub mod users;

use serde::Serialize;

/// 分页查询统一返回结构
#[derive(Debug, Serialize)]
pub struct Paged<T> {
    pub items: Vec<T>,
    pub total: i64,
}

/// 归一化分页参数：page 从 1 开始，page_size 限制在 1..=100
pub fn clamp_page(page: i64, page_size: i64) -> (i64, i64) {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    (page, page_size)
}
