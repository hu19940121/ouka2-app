//! 自定义电台相关命令

use std::collections::HashSet;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use crate::radio::Station;
use crate::AppState;

/// 自定义电台数据文件名
const CUSTOM_STATIONS_FILE: &str = "custom_stations.json";

/// 从文件加载自定义电台
pub(crate) fn load_custom_stations_from_file(data_dir: &std::path::Path) -> Vec<Station> {
    let path = data_dir.join(CUSTOM_STATIONS_FILE);
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// 将保存的自定义电台合并进现有列表，避免重复 ID。
pub(crate) fn merge_custom_stations(data_dir: &std::path::Path, stations: &mut Vec<Station>) {
    let custom_stations = load_custom_stations_from_file(data_dir);
    if custom_stations.is_empty() {
        return;
    }

    let existing_ids: HashSet<_> = stations.iter().map(|station| station.id.clone()).collect();
    stations.extend(
        custom_stations
            .into_iter()
            .filter(|station| !existing_ids.contains(&station.id)),
    );
}

/// 保存自定义电台到文件
fn save_custom_stations_to_file(
    data_dir: &std::path::Path,
    stations: &[Station],
) -> Result<(), String> {
    let path = data_dir.join(CUSTOM_STATIONS_FILE);
    let json = serde_json::to_string_pretty(stations).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    log::debug!("custom stations saved: {:?}", path);
    Ok(())
}

/// 加载自定义电台列表
#[tauri::command]
pub async fn load_custom_stations(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Station>, String> {
    let state = state.lock().await;
    let data_dir = state.crawler.data_dir();
    let custom_stations = load_custom_stations_from_file(data_dir);
    log::debug!("custom stations loaded: {}", custom_stations.len());

    // 同步到服务器状态
    let server_state = state.server.state();
    for station in &custom_stations {
        let mut stations_map = server_state.stations.write().await;
        stations_map.insert(station.id.clone(), station.clone());
    }

    Ok(custom_stations)
}

/// 添加自定义电台
#[tauri::command]
pub async fn add_custom_station(
    name: String,
    url: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Station, String> {
    if name.trim().is_empty() {
        return Err("电台名称不能为空".to_string());
    }
    if url.trim().is_empty() {
        return Err("流地址不能为空".to_string());
    }

    let state = state.lock().await;
    let data_dir = state.crawler.data_dir().clone();

    // 生成唯一 ID
    let id = format!(
        "custom_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let station = Station {
        id: id.clone(),
        name: name.trim().to_string(),
        subtitle: format!("自定义电台 · {}", url.trim()),
        image: String::new(),
        province: "自定义".to_string(),
        play_url_low: Some(url.trim().to_string()),
        mp3_play_url_low: None,
        mp3_play_url_high: None,
        is_custom: true,
    };

    // 加载现有自定义电台并追加
    let mut custom_stations = load_custom_stations_from_file(&data_dir);
    custom_stations.push(station.clone());
    save_custom_stations_to_file(&data_dir, &custom_stations)?;

    // 同步到服务器状态
    let server_state = state.server.state();
    {
        let mut stations_map = server_state.stations.write().await;
        stations_map.insert(station.id.clone(), station.clone());
    }

    log::info!("添加自定义电台: {}", station.name);
    Ok(station)
}

/// 删除自定义电台
#[tauri::command]
pub async fn remove_custom_station(
    id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    let data_dir = state.crawler.data_dir().clone();

    // 从文件中移除
    let mut custom_stations = load_custom_stations_from_file(&data_dir);
    let before_len = custom_stations.len();
    custom_stations.retain(|s| s.id != id);

    if custom_stations.len() == before_len {
        return Err("未找到该自定义电台".to_string());
    }

    save_custom_stations_to_file(&data_dir, &custom_stations)?;

    // 从服务器状态中移除
    let server_state = state.server.state();
    {
        let mut stations_map = server_state.stations.write().await;
        stations_map.remove(&id);
    }

    log::info!("删除自定义电台: {}", id);
    Ok(())
}

/// 更新自定义电台
#[tauri::command]
pub async fn update_custom_station(
    id: String,
    name: String,
    url: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Station, String> {
    if name.trim().is_empty() {
        return Err("电台名称不能为空".to_string());
    }
    if url.trim().is_empty() {
        return Err("流地址不能为空".to_string());
    }

    let state = state.lock().await;
    let data_dir = state.crawler.data_dir().clone();

    let mut custom_stations = load_custom_stations_from_file(&data_dir);
    let station = custom_stations
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or("未找到该自定义电台")?;

    station.name = name.trim().to_string();
    station.subtitle = format!("自定义电台 · {}", url.trim());
    station.play_url_low = Some(url.trim().to_string());

    let updated = station.clone();
    save_custom_stations_to_file(&data_dir, &custom_stations)?;

    // 同步到服务器状态
    let server_state = state.server.state();
    {
        let mut stations_map = server_state.stations.write().await;
        stations_map.insert(id.clone(), updated.clone());
    }

    log::info!("更新自定义电台: {}", updated.name);
    Ok(updated)
}
