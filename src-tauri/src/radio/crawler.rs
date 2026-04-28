//! 电台爬虫模块
//!
//! 从云听网站爬取所有电台数据

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::radio::api::RadioApi;
use crate::radio::models::{CrawlProgress, Station};

/// 电台爬虫
pub struct Crawler {
    api: RadioApi,
    data_dir: PathBuf,
    stations: Arc<RwLock<Vec<Station>>>,
}

impl Crawler {
    /// 创建新的爬虫实例
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            api: RadioApi::new(),
            data_dir,
            stations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 爬取所有电台
    ///
    /// # 参数
    /// - `progress_callback`: 进度回调函数
    pub async fn crawl_all<F>(&self, mut progress_callback: F) -> anyhow::Result<Vec<Station>>
    where
        F: FnMut(CrawlProgress),
    {
        let mut all_stations: Vec<Station> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        // 1. 获取央广电台
        log::debug!("fetch central stations");
        progress_callback(CrawlProgress {
            current: 0,
            total: 1,
            province: "央广".to_string(),
            stations_found: 0,
        });

        let central_stations = self.api.get_stations("0", "0").await?;
        for raw in central_stations {
            if !seen_ids.contains(&raw.content_id) {
                seen_ids.insert(raw.content_id.clone());
                all_stations.push(raw.into_station("央广"));
            }
        }
        log::debug!("central stations: {}", all_stations.len());

        // 2. 获取所有省份
        log::debug!("fetch province list");
        let provinces = self.api.get_provinces().await?;
        let total_provinces = provinces.len();
        log::debug!("provinces: {}", total_provinces);

        // 3. 遍历每个省份获取电台
        for (i, province) in provinces.iter().enumerate() {
            log::debug!("fetch province stations: {}", province.province_name);
            progress_callback(CrawlProgress {
                current: i + 1,
                total: total_provinces,
                province: province.province_name.clone(),
                stations_found: all_stations.len(),
            });

            match self.api.get_stations(&province.province_code, "0").await {
                Ok(stations) => {
                    let mut count = 0;
                    for raw in stations {
                        if !seen_ids.contains(&raw.content_id) {
                            seen_ids.insert(raw.content_id.clone());
                            all_stations.push(raw.into_station(&province.province_name));
                            count += 1;
                        }
                    }
                    log::debug!("province stations: {} -> {}", province.province_name, count);
                }
                Err(e) => {
                    log::error!("获取 {} 电台失败: {}", province.province_name, e);
                }
            }

            // 避免请求过快
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        log::debug!("crawl completed: {}", all_stations.len());

        // 保存到缓存
        {
            let mut stations = self.stations.write().await;
            *stations = all_stations.clone();
        }

        // 保存到文件
        self.save_stations(&all_stations)?;

        Ok(all_stations)
    }

    /// 保存电台数据到文件
    pub fn save_stations(&self, stations: &[Station]) -> anyhow::Result<()> {
        let path = self.data_dir.join("stations.json");

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(stations)?;
        std::fs::write(&path, json)?;

        log::debug!("stations saved: {:?}", path);
        Ok(())
    }

    /// 从文件加载电台数据
    pub fn load_stations(&self) -> anyhow::Result<Vec<Station>> {
        let path = self.data_dir.join("stations.json");

        if !path.exists() {
            log::debug!("stations file not found: {:?}", path);
            return Ok(Vec::new());
        }

        let json = std::fs::read_to_string(&path)?;
        let stations: Vec<Station> = serde_json::from_str(&json)?;

        log::debug!("stations loaded: {}", stations.len());
        Ok(stations)
    }

    /// 获取缓存的电台列表
    pub async fn get_stations(&self) -> Vec<Station> {
        self.stations.read().await.clone()
    }

    /// 设置电台列表（从加载的数据）
    pub async fn set_stations(&self, stations: Vec<Station>) {
        let mut s = self.stations.write().await;
        *s = stations;
    }

    /// 获取数据目录
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// 获取 API 引用（用于刷新流地址）
    pub fn api(&self) -> &RadioApi {
        &self.api
    }
}

/// 统计各省份电台数量
pub fn get_province_stats(stations: &[Station]) -> Vec<(String, usize)> {
    use std::collections::HashMap;

    let mut stats: HashMap<String, usize> = HashMap::new();
    for station in stations {
        *stats.entry(station.province.clone()).or_insert(0) += 1;
    }

    let mut result: Vec<_> = stats.into_iter().collect();
    result.sort_by(|a, b| {
        // 央广排第一
        if a.0 == "央广" {
            return std::cmp::Ordering::Less;
        }
        if b.0 == "央广" {
            return std::cmp::Ordering::Greater;
        }
        // 其他按数量降序
        b.1.cmp(&a.1)
    });

    result
}
