//! 配置相关命令

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use super::custom::merge_custom_stations;
use crate::radio::SiiGenerator;
use crate::AppState;

/// 安装列表配置文件名
const INSTALL_SELECTION_FILE: &str = "install_selection.json";

/// 安装列表状态
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSelectionState {
    pub station_ids: Vec<String>,
    pub has_saved_selection: bool,
}

/// 合并自定义电台到电台列表
async fn get_all_stations(state: &AppState) -> Vec<crate::radio::Station> {
    let mut stations = state.crawler.get_stations().await;
    merge_custom_stations(state.crawler.data_dir(), &mut stations);
    stations
}

/// 从文件加载安装列表
pub(crate) fn load_install_selection_from_file(data_dir: &std::path::Path) -> Option<Vec<String>> {
    let path = data_dir.join(INSTALL_SELECTION_FILE);
    if !path.exists() {
        return None;
    }

    let json = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&json).ok()
}

/// 保存安装列表到文件
pub(crate) fn save_install_selection_to_file(
    data_dir: &std::path::Path,
    station_ids: &[String],
) -> Result<(), String> {
    let path = data_dir.join(INSTALL_SELECTION_FILE);
    let json = serde_json::to_string_pretty(station_ids).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    log::debug!("install selection saved: {:?}", path);
    Ok(())
}

/// 过滤并按传入顺序保留电台
fn filter_stations_by_ids(
    stations: Vec<crate::radio::Station>,
    station_ids: &[String],
) -> Vec<crate::radio::Station> {
    let station_map: HashMap<_, _> = stations
        .into_iter()
        .map(|station| (station.id.clone(), station))
        .collect();

    let mut seen = HashSet::new();
    let mut filtered = Vec::new();

    for station_id in station_ids {
        if !seen.insert(station_id) {
            continue;
        }

        if let Some(station) = station_map.get(station_id) {
            filtered.push(station.clone());
        }
    }

    filtered
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

/// 生成选中电台的 SII 配置文件
#[tauri::command]
pub async fn generate_sii_with_selection(
    station_ids: Vec<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let state = state.lock().await;

    let stations = get_all_stations(&state).await;
    if stations.is_empty() {
        return Err("没有电台数据，请先爬取电台".to_string());
    }

    let selected_stations = filter_stations_by_ids(stations, &station_ids);
    if selected_stations.is_empty() {
        return Err("请至少选择一个电台".to_string());
    }

    let port = *state.server.state().port.read().await;
    let generator = SiiGenerator::new("127.0.0.1", port);
    let content = generator.generate(&selected_stations);

    let path = state.crawler.data_dir().join("live_streams.sii");
    generator
        .save_to_file(&content, &path)
        .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

/// 安装 SII 到欧卡2目录
#[tauri::command]
pub async fn install_sii_to_ets2(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
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

/// 安装选中电台到欧卡2目录
#[tauri::command]
pub async fn install_sii_to_ets2_with_selection(
    station_ids: Vec<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let state = state.lock().await;

    let stations = get_all_stations(&state).await;
    if stations.is_empty() {
        return Err("没有电台数据，请先爬取电台".to_string());
    }

    let selected_stations = filter_stations_by_ids(stations, &station_ids);
    if selected_stations.is_empty() {
        return Err("请至少选择一个电台".to_string());
    }

    let port = *state.server.state().port.read().await;
    let generator = SiiGenerator::new("127.0.0.1", port);
    let content = generator.generate(&selected_stations);

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

/// 读取已保存的安装列表
#[tauri::command]
pub async fn load_install_selection(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<InstallSelectionState, String> {
    let state = state.lock().await;
    let data_dir = state.crawler.data_dir();

    match load_install_selection_from_file(data_dir) {
        Some(station_ids) => Ok(InstallSelectionState {
            station_ids,
            has_saved_selection: true,
        }),
        None => Ok(InstallSelectionState {
            station_ids: Vec::new(),
            has_saved_selection: false,
        }),
    }
}

/// 保存安装列表
#[tauri::command]
pub async fn save_install_selection(
    station_ids: Vec<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    save_install_selection_to_file(state.crawler.data_dir(), &station_ids)
}
