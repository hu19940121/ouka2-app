//! 配置相关命令

use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use super::custom::merge_custom_stations;
use crate::radio::SiiGenerator;
use crate::AppState;

/// 合并自定义电台到电台列表
async fn get_all_stations(state: &AppState) -> Vec<crate::radio::Station> {
    let mut stations = state.crawler.get_stations().await;
    merge_custom_stations(state.crawler.data_dir(), &mut stations);
    stations
}

/// 生成 SII 配置文件
#[tauri::command]
pub async fn generate_sii(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().await;

    let stations = get_all_stations(&state).await;
    if stations.is_empty() {
        return Err("没有电台数据，请先爬取电台".to_string());
    }

    let port = *state.server.state().port.read().await;
    let generator = SiiGenerator::new("127.0.0.1", port);
    let content = generator.generate(&stations);

    // 保存到数据目录
    let path = state.crawler.data_dir().join("live_streams.sii");
    generator
        .save_to_file(&content, &path)
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

/// 安装 SII 到欧卡2目录
#[tauri::command]
pub async fn install_sii_to_ets2(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let state = state.lock().await;

    let stations = get_all_stations(&state).await;
    if stations.is_empty() {
        return Err("没有电台数据，请先爬取电台".to_string());
    }

    let port = *state.server.state().port.read().await;
    let generator = SiiGenerator::new("127.0.0.1", port);
    let content = generator.generate(&stations);

    let path = generator
        .install_to_ets2(&content)
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

/// 获取欧卡2文档目录
#[tauri::command]
pub fn get_ets2_paths() -> Vec<String> {
    SiiGenerator::detect_ets2_paths()
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

/// 获取应用数据目录
#[tauri::command]
pub async fn get_app_data_dir(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().await;
    Ok(state.crawler.data_dir().to_string_lossy().to_string())
}
