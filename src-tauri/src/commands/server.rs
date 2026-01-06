//! 流媒体服务器相关命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::radio::ServerStatus;
use crate::AppState;

/// 启动流媒体服务器
#[tauri::command]
pub async fn start_server(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().await;
    
    // 确保电台数据已加载到服务器
    let stations = state.crawler.get_stations().await;
    if stations.is_empty() {
        // 尝试从文件加载
        if let Ok(loaded) = state.crawler.load_stations() {
            state.crawler.set_stations(loaded.clone()).await;
            state.server.state().load_stations(loaded).await;
            log::info!("📻 从文件加载了电台数据");
        }
    } else {
        state.server.state().load_stations(stations).await;
    }
    
    // 启动服务器
    state.server.start().await.map_err(|e| e.to_string())?;
    
    let status = state.server.state().get_status().await;
    log::info!("🚀 服务器已启动，共 {} 个电台可用", status.total_stations);
    
    Ok(())
}

/// 停止流媒体服务器
#[tauri::command]
pub async fn stop_server(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.server.stop();
    log::info!("🛑 服务器已停止");
    Ok(())
}

/// 获取服务器状态
#[tauri::command]
pub async fn get_server_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<ServerStatus, String> {
    let state = state.lock().await;
    Ok(state.server.state().get_status().await)
}
