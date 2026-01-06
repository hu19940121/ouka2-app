//! 欧卡2中国电台桌面应用
//!
//! 将云听电台转换为欧卡2可用格式的桌面应用

mod commands;
mod radio;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use commands::*;
use radio::{Crawler, StreamServer};
use utils::{FFmpegManager, check_ffmpeg};

/// 应用全局状态
pub struct AppState {
    pub crawler: Crawler,
    pub server: StreamServer,
}

impl AppState {
    pub fn new(data_dir: PathBuf, ffmpeg_path: PathBuf, server_port: u16) -> Self {
        Self {
            crawler: Crawler::new(data_dir),
            server: StreamServer::new(server_port, ffmpeg_path),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 获取应用数据目录
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录");

            // 确保目录存在
            std::fs::create_dir_all(&data_dir).ok();

            log::info!("📁 应用数据目录: {:?}", data_dir);

            // 检测 FFmpeg
            let resource_dir = app.path().resource_dir().ok();
            let ffmpeg_path = FFmpegManager::detect_ffmpeg(resource_dir.as_ref())
                .unwrap_or_else(|| PathBuf::from("ffmpeg"));

            // 创建应用状态
            let state = Arc::new(Mutex::new(AppState::new(data_dir, ffmpeg_path, 3000)));

            // 管理状态
            app.manage(state.clone());

            // 尝试加载已保存的电台数据
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                let state = state_clone.lock().await;
                if let Ok(stations) = state.crawler.load_stations() {
                    if !stations.is_empty() {
                        state.crawler.set_stations(stations.clone()).await;
                        state.server.state().load_stations(stations).await;
                        log::info!("✅ 已加载保存的电台数据");
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 爬虫命令
            get_stations,
            crawl_stations,
            get_province_statistics,
            load_saved_stations,
            // 服务器命令
            start_server,
            stop_server,
            get_server_status,
            // 配置命令
            generate_sii,
            install_sii_to_ets2,
            get_ets2_paths,
            get_app_data_dir,
            // 工具命令
            check_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
