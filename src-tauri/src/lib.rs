// 云记 - Tauri 应用入口
mod auth;
mod bencode;
mod commands;
mod db;
mod error;
mod magnet;
mod state;
mod storage;

use std::path::PathBuf;
use tauri::Manager;

use crate::state::AppState;
use crate::storage::get_storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // 获取数据目录：exe 同级 data/
            let exe_dir = std::env::current_exe()
                .map_err(|e| e.to_string())?
                .parent()
                .ok_or("无法确定 exe 目录")?
                .to_path_buf();
            let data_dir = exe_dir.join("data");

            // 开发模式：使用项目根的 data/
            let data_dir = if cfg!(debug_assertions) {
                let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                manifest_dir.parent().unwrap_or(&manifest_dir).join("data")
            } else {
                data_dir
            };

            std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

            // 初始化数据库
            let db_path = data_dir.join("app.db");
            let pool = tauri::async_runtime::block_on(async {
                db::create_pool(&db_path)
                    .await
                    .map_err(|e| e.to_string())
            })?;

            // 运行迁移
            tauri::async_runtime::block_on(async {
                db::run_migrations(&pool)
                    .await
                    .map_err(|e| e.to_string())
            })?;

            // 初始化存储
            let storage = tauri::async_runtime::block_on(async {
                get_storage(&pool, &data_dir)
                    .await
                    .map_err(|e| e.to_string())
            })?;

            // 首次启动创建管理员
            tauri::async_runtime::block_on(async {
                let count = db::repository::users::count(&pool).await.map_err(|e| e.to_string())?;
                if count == 0 {
                    let hash = auth::hash_password("admin123")
                        .map_err(|e| e.to_string())?;
                    db::repository::users::create_admin(&pool, "admin", &hash)
                        .await
                        .map_err(|e| e.to_string())?;
                    println!("已创建默认管理员: admin / admin123");
                }
                Ok::<(), String>(())
            })?;

            // 创建 AppState
            let app_state = AppState::new(pool, storage, data_dir);
            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // auth
            commands::auth::auth_login,
            commands::auth::auth_logout,
            commands::auth::auth_me,
            commands::auth::auth_register,
            commands::auth::auth_update_profile,
            commands::auth::auth_change_username,
            commands::auth::auth_upload_avatar,
            commands::auth::auth_change_password,
            // users（管理）
            commands::users::list_users,
            commands::users::delete_users,
            commands::users::reset_user_password,
            commands::users::update_user,
            // resources
            commands::resources::list_resources,
            commands::resources::get_resource,
            commands::resources::create_resource,
            commands::resources::update_resource,
            commands::resources::batch_update_resources,
            commands::resources::delete_resource,
            commands::resources::restore_resource,
            commands::resources::permanent_delete_resource,
            commands::resources::list_trash_resources,
            commands::resources::check_resource_access,
            commands::resources::upload_resource_file,
            // images
            commands::images::list_resource_images,
            commands::images::upload_resource_image,
            commands::images::delete_resource_image,
            commands::images::reorder_resource_images,
            commands::images::set_resource_thumbnail,
            commands::images::clear_resource_thumbnail,
            commands::images::list_collection_images,
            commands::images::upload_collection_image,
            commands::images::delete_collection_image,
            commands::images::reorder_collection_images,
            // tags
            commands::tags::list_tags,
            commands::tags::create_tag,
            commands::tags::rename_tag,
            commands::tags::delete_tag,
            commands::tags::merge_tags,
            // collections
            commands::collections::list_collections,
            commands::collections::get_collection,
            commands::collections::create_collection,
            commands::collections::update_collection,
            commands::collections::delete_collection,
            commands::collections::restore_collection,
            commands::collections::permanent_delete_collection,
            commands::collections::sync_collection_resources,
            commands::collections::list_collection_resource_ids,
            // settings
            commands::settings::get_public_settings,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::get_upload_base_dir,
            // categories（树形分类）
            commands::categories::list_categories,
            commands::categories::list_categories_flat,
            commands::categories::get_category,
            commands::categories::create_category,
            commands::categories::update_category,
            commands::categories::delete_category,
            commands::categories::restore_category,
            commands::categories::permanent_delete_category,
            commands::categories::move_category,
            commands::categories::save_category_order,
            // torrent
            commands::torrent::get_resource_files,
            commands::torrent::save_resource_files,
            commands::torrent::clear_resource_files,
            commands::torrent::parse_torrent_file,
            commands::torrent::fetch_torrent,
            commands::torrent::get_fetch_status,
            commands::torrent::check_resource_files,
            commands::torrent::reveal_file_in_explorer,
            commands::torrent::open_file_with_default,
            // cleanup（附件清理）
            commands::cleanup::scan_orphan_files,
            commands::cleanup::delete_orphan_files,
            // collector（网页采集）
            commands::collector::fetch_webpage,
            commands::collector::list_collectors,
            commands::collector::import_collector,
            commands::collector::collector_set_thumbnail,
            commands::collector::collector_add_images,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
