//! 欧卡2中国电台桌面应用
//!
//! 将云听电台转换为欧卡2可用格式的桌面应用

mod commands;
mod diagnostics;
mod radio;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use commands::custom::merge_custom_stations;
use commands::*;
use diagnostics::DiagnosticLogger;
use radio::{Crawler, StreamServer};
use utils::{check_ffmpeg, FFmpegManager};

/// 应用全局状态
pub struct AppState {
    pub crawler: Crawler,
    pub server: StreamServer,
    pub logger: DiagnosticLogger,
}

impl AppState {
    pub fn new(
        data_dir: PathBuf,
        ffmpeg_path: PathBuf,
        server_port: u16,
        logger: DiagnosticLogger,
    ) -> Self {
        Self {
            crawler: Crawler::new(data_dir),
            server: StreamServer::new(server_port, ffmpeg_path, logger.clone()),
            logger,
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
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");

            // 确保目录存在
            std::fs::create_dir_all(&data_dir).ok();

            log::debug!("app data dir: {:?}", data_dir);
            let logger = DiagnosticLogger::new();
            logger.attach_app(app.handle().clone());
            logger.info("app", "应用启动，诊断日志已初始化");

            // 检测 FFmpeg
            let resource_dir = app.path().resource_dir().ok();
            let ffmpeg_path = FFmpegManager::detect_ffmpeg(resource_dir.as_ref())
                .unwrap_or_else(|| PathBuf::from("ffmpeg"));
            logger.info("ffmpeg", format!("FFmpeg 路径: {}", ffmpeg_path.display()));

            // 创建应用状态
            let state = Arc::new(Mutex::new(AppState::new(
                data_dir,
                ffmpeg_path,
                3000,
                logger,
            )));

            // 管理状态
            app.manage(state.clone());

            // 尝试加载已保存的电台数据
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                let state = state_clone.lock().await;
                if let Ok(stations) = state.crawler.load_stations() {
                    if !stations.is_empty() {
                        state.crawler.set_stations(stations.clone()).await;
                        let mut stations_for_server = stations;
                        merge_custom_stations(state.crawler.data_dir(), &mut stations_for_server);
                        state
                            .server
                            .state()
                            .load_stations(stations_for_server)
                            .await;
                        log::debug!("loaded saved stations");
                        state.logger.info("app", "已加载本地保存的电台数据");
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
            stop_active_streams,
            get_server_status,
            get_diagnostic_logs,
            clear_diagnostic_logs,
            // 配置命令
            generate_sii,
            generate_sii_with_selection,
            install_sii_to_ets2,
            install_sii_to_ets2_with_selection,
            get_ets2_paths,
            get_app_data_dir,
            load_install_selection,
            save_install_selection,
            // 自定义电台命令
            add_custom_station,
            remove_custom_station,
            update_custom_station,
            load_custom_stations,
            // 工具命令
            check_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
