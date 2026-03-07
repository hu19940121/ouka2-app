//! 爬虫相关命令

use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

use super::custom::merge_custom_stations;
use crate::radio::{CrawlProgress, Station, get_province_stats};
use crate::AppState;

/// 获取电台列表
#[tauri::command]
pub async fn get_stations(state: State<'_, Arc<Mutex<AppState>>>) -> Result<Vec<Station>, String> {
    let state = state.lock().await;
    let stations = state.crawler.get_stations().await;
    Ok(stations)
}

/// 爬取电台数据
#[tauri::command]
pub async fn crawl_stations(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Station>, String> {
    // 获取 data_dir，然后立即释放锁
    let data_dir = {
        let s = state.lock().await;
        s.crawler.data_dir().clone()
    };

    // 创建一个临时的爬虫实例进行爬取（不持有锁）
    let crawler = crate::radio::Crawler::new(data_dir);

    // 爬取电台，发送进度事件
    let app_clone = app.clone();
    let stations = crawler
        .crawl_all(move |progress: CrawlProgress| {
            log::info!("📻 进度: {}/{} - {} (已找到 {} 个电台)", 
                progress.current, progress.total, progress.province, progress.stations_found);
            let _ = app_clone.emit("crawl-progress", &progress);
        })
        .await
        .map_err(|e| {
            log::error!("❌ 爬取失败: {}", e);
            e.to_string()
        })?;

    log::info!("✅ 爬取完成，共 {} 个电台", stations.len());

    // 添加郭德纲电台
    let mut stations = stations;
    stations.push(Station {
        id: "guodegang_radio".to_string(),
        name: "郭德纲电台".to_string(),
        subtitle: "随机播放B站郭德纲相声".to_string(),
        image: "https://i0.hdslb.com/bfs/face/a6a0bb6eb6a52b96f5ea0e5b6a0a6ff3d74e55cb.jpg".to_string(),
        province: "bilibili".to_string(),
        play_url_low: None,
        mp3_play_url_low: None,
        mp3_play_url_high: Some("http://127.0.0.1:3000/stream/guodegang_radio".to_string()),
        is_custom: false,
    });

    // 重新获取锁来更新状态
    {
        let s = state.lock().await;
        s.crawler.set_stations(stations.clone()).await;
        let mut stations_for_server = stations.clone();
        merge_custom_stations(s.crawler.data_dir(), &mut stations_for_server);
        s.server.state().load_stations(stations_for_server).await;
    }

    Ok(stations)
}

/// 获取各省份电台统计
#[tauri::command]
pub async fn get_province_statistics(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<(String, usize)>, String> {
    let state = state.lock().await;
    let stations = state.crawler.get_stations().await;
    Ok(get_province_stats(&stations))
}

/// 加载已保存的电台数据
#[tauri::command]
pub async fn load_saved_stations(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Station>, String> {
    let state = state.lock().await;

    let mut stations = state.crawler.load_stations().map_err(|e| e.to_string())?;

    // 添加郭德纲电台（B站随机相声）
    stations.push(Station {
        id: "guodegang_radio".to_string(),
        name: "郭德纲电台".to_string(),
        subtitle: "随机播放B站郭德纲相声".to_string(),
        image: "https://i0.hdslb.com/bfs/face/a6a0bb6eb6a52b96f5ea0e5b6a0a6ff3d74e55cb.jpg".to_string(),
        province: "bilibili".to_string(),
        play_url_low: None,
        mp3_play_url_low: None,
        mp3_play_url_high: Some("http://127.0.0.1:3000/stream/guodegang_radio".to_string()),
        is_custom: false,
    });

    // 更新缓存
    state.crawler.set_stations(stations.clone()).await;

    // 更新服务器
    let mut stations_for_server = stations.clone();
    merge_custom_stations(state.crawler.data_dir(), &mut stations_for_server);
    state.server.state().load_stations(stations_for_server).await;

    Ok(stations)
}
