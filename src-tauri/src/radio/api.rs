//! 云听电台 API 封装
//!
//! 实现与 radio.cn 的 API 通信，包括签名生成和请求发送

use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use crate::radio::models::{ApiResponse, Province, RawStation};

/// API 密钥（从云听网站前端JS中提取）
const API_KEY: &str = "f0fc4c668392f9f9a447e48584c214ee";
/// API 基础URL
const BASE_URL: &str = "https://ytmsout.radio.cn";

/// 云听电台 API 客户端
pub struct RadioApi {
    client: Client,
}

impl RadioApi {
    /// 创建新的 API 客户端
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client }
    }

    /// 生成 API 签名
    ///
    /// 签名算法：
    /// 1. 按键名排序参数
    /// 2. 拼接为 key=value&key=value 格式
    /// 3. 追加 timestamp 和 key
    /// 4. MD5 哈希并转大写
    pub fn generate_sign(params: &HashMap<String, String>, timestamp: i64) -> String {
        // 按键排序
        let mut sorted_keys: Vec<_> = params.keys().collect();
        sorted_keys.sort();

        // 拼接参数
        let param_str: String = sorted_keys
            .iter()
            .map(|k| format!("{}={}", k, params.get(*k).unwrap()))
            .collect::<Vec<_>>()
            .join("&");

        // 构建签名字符串
        let sign_text = if param_str.is_empty() {
            format!("timestamp={}&key={}", timestamp, API_KEY)
        } else {
            format!("{}&timestamp={}&key={}", param_str, timestamp, API_KEY)
        };

        // MD5 哈希并转大写
        let digest = md5::compute(sign_text.as_bytes());
        format!("{:X}", digest)
    }

    /// 发起 API 请求
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: HashMap<String, String>,
    ) -> anyhow::Result<T> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let sign = Self::generate_sign(&params, timestamp);

        // 构建 URL
        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = if query_string.is_empty() {
            format!("{}{}", BASE_URL, endpoint)
        } else {
            format!("{}{}?{}", BASE_URL, endpoint, query_string)
        };

        log::info!("   🔗 请求: {}", url);

        // 发送请求
        let response = match self
            .client
            .get(&url)
            .header("equipmentId", "0000")
            .header("platformCode", "WEB")
            .header("Content-Type", "application/json")
            .header("timestamp", timestamp.to_string())
            .header("sign", sign)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("   ❌ HTTP 请求失败: {}", e);
                return Err(e.into());
            }
        };

        log::info!("   ✅ HTTP 状态: {}", response.status());

        let text = response.text().await?;
        
        let data: ApiResponse<T> = match serde_json::from_str(&text) {
            Ok(d) => d,
            Err(e) => {
                log::error!("   ❌ JSON 解析失败: {}", e);
                log::error!("   响应内容: {}", &text[..text.len().min(500)]);
                return Err(e.into());
            }
        };

        if data.code != 0 {
            log::error!("   ❌ API 错误: {} - {:?}", data.code, data.message);
            anyhow::bail!(
                "API 错误: {} - {}",
                data.code,
                data.message.unwrap_or_default()
            );
        }

        data.data.ok_or_else(|| anyhow::anyhow!("API 返回数据为空"))
    }

    /// 获取所有省份列表
    pub async fn get_provinces(&self) -> anyhow::Result<Vec<Province>> {
        self.request("/web/appProvince/list/all", HashMap::new())
            .await
    }

    /// 获取电台列表
    ///
    /// # 参数
    /// - `province_code`: 省份代码，"0" 表示央广电台
    /// - `category_id`: 分类ID，"0" 表示全部
    pub async fn get_stations(
        &self,
        province_code: &str,
        category_id: &str,
    ) -> anyhow::Result<Vec<RawStation>> {
        let mut params = HashMap::new();
        params.insert("provinceCode".to_string(), province_code.to_string());
        params.insert("categoryId".to_string(), category_id.to_string());

        self.request("/web/appBroadcast/list", params).await
    }

    /// 刷新电台流地址
    ///
    /// 因为流地址可能会过期，需要实时获取最新的地址
    pub async fn refresh_stream_url(
        &self,
        station_id: &str,
        province: &str,
    ) -> anyhow::Result<Option<String>> {
        let province_code = Self::get_province_code(province);

        // 先在对应省份查找
        let stations = self.get_stations(&province_code, "0").await?;
        if let Some(station) = stations.iter().find(|s| s.content_id == station_id) {
            if let Some(url) = station
                .mp3_play_url_high
                .as_ref()
                .or(station.mp3_play_url_low.as_ref())
                .or(station.play_url_low.as_ref())
            {
                return Ok(Some(url.clone()));
            }
        }

        // 如果没找到，尝试在央广台查找
        if province_code != "0" {
            let central_stations = self.get_stations("0", "0").await?;
            if let Some(station) = central_stations.iter().find(|s| s.content_id == station_id) {
                if let Some(url) = station
                    .mp3_play_url_high
                    .as_ref()
                    .or(station.mp3_play_url_low.as_ref())
                    .or(station.play_url_low.as_ref())
                {
                    return Ok(Some(url.clone()));
                }
            }
        }

        Ok(None)
    }

    /// 获取省份代码映射
    fn get_province_code(province: &str) -> String {
        match province {
            "央广" | "国家" => "0",
            "安徽" => "340000",
            "北京" => "110000",
            "重庆" => "500000",
            "福建" => "350000",
            "甘肃" => "620000",
            "广东" => "440000",
            "广西" => "450000",
            "贵州" => "520000",
            "海南" => "460000",
            "河北" => "130000",
            "河南" => "410000",
            "黑龙江" => "230000",
            "湖北" => "420000",
            "湖南" => "430000",
            "吉林" => "220000",
            "江苏" => "320000",
            "江西" => "360000",
            "辽宁" => "210000",
            "内蒙古" => "150000",
            "宁夏" => "640000",
            "青海" => "630000",
            "山东" => "370000",
            "山西" => "140000",
            "陕西" => "610000",
            "上海" => "310000",
            "四川" => "510000",
            "西藏" => "540000",
            "新疆" => "650000",
            "新疆兵团" => "660000",
            "云南" => "530000",
            "浙江" => "330000",
            _ => "0",
        }
        .to_string()
    }
}

impl Default for RadioApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sign() {
        let mut params = HashMap::new();
        params.insert("categoryId".to_string(), "0".to_string());
        params.insert("provinceCode".to_string(), "0".to_string());

        let timestamp = 1704067200000i64; // 固定时间戳用于测试
        let sign = RadioApi::generate_sign(&params, timestamp);

        // 签名应该是32位大写十六进制字符串
        assert_eq!(sign.len(), 32);
        assert!(sign.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
