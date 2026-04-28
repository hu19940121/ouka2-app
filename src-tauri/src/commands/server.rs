//! 流媒体服务器相关命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use super::custom::merge_custom_stations;
use crate::radio::ServerStatus;
use crate::AppState;

/// 启动流媒体服务器
#[tauri::command]
pub async fn start_server(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().await;

    // 确保电台数据已加载到服务器，并合并自定义电台。
    let mut stations = state.crawler.get_stations().await;
    if stations.is_empty() {
        if let Ok(loaded) = state.crawler.load_stations() {
            log::debug!("从文件加载电台数据");
            state.crawler.set_stations(loaded.clone()).await;
            stations = loaded;
        }
    }

    let before_len = stations.len();
    merge_custom_stations(state.crawler.data_dir(), &mut stations);
    let custom_count = stations.len().saturating_sub(before_len);
    if custom_count > 0 {
        log::debug!("合并自定义电台: {}", custom_count);
    }

    state.server.state().load_stations(stations).await;

    // 启动服务器
    state.server.start().await.map_err(|e| e.to_string())?;

    let status = state.server.state().get_status().await;
    log::info!("服务器已启动，可用电台: {}", status.total_stations);

    Ok(())
}

/// 停止流媒体服务器
#[tauri::command]
pub async fn stop_server(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.server.stop().await;
    log::info!("服务器已停止");
    Ok(())
}

/// 停止当前所有活动流，但保持流媒体服务器运行
#[tauri::command]
pub async fn stop_active_streams(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let state = state.lock().await;
    state.server.stop_active_streams().await;
    log::debug!("已请求停止所有活动流");
    Ok(())
}

/// 获取服务器状态
#[tauri::command]
pub async fn get_server_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ServerStatus, String> {
    let state = state.lock().await;
    let is_running = state.server.is_running();
    let server_state = state.server.state();
    let port = *server_state.port.read().await;
    let active_streams = server_state.active_streams.read().await.len();
    let total_stations = server_state.stations.read().await.len();

    Ok(ServerStatus {
        running: is_running,
        port,
        active_streams,
        total_stations,
    })
}
