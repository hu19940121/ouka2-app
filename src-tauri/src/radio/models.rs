//! 数据模型定义

use serde::{Deserialize, Serialize};

/// 电台信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    /// 电台ID
    pub id: String,
    /// 电台名称
    pub name: String,
    /// 副标题
    #[serde(default)]
    pub subtitle: String,
    /// 电台图片URL
    #[serde(default)]
    pub image: String,
    /// 所属省份
    pub province: String,
    /// 低质量播放地址 (m3u8)
    #[serde(default)]
    pub play_url_low: Option<String>,
    /// 低质量MP3播放地址
    #[serde(default)]
    pub mp3_play_url_low: Option<String>,
    /// 高质量MP3播放地址
    #[serde(default)]
    pub mp3_play_url_high: Option<String>,
}

impl Station {
    /// 获取最佳可用的流地址
    pub fn get_best_stream_url(&self) -> Option<&str> {
        self.mp3_play_url_high
            .as_deref()
            .or(self.mp3_play_url_low.as_deref())
            .or(self.play_url_low.as_deref())
    }
}

/// 省份信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Province {
    /// 省份代码（API 返回整数，我们转换为字符串）
    #[serde(deserialize_with = "deserialize_province_code")]
    pub province_code: String,
    /// 省份名称
    pub province_name: String,
}

/// 反序列化省份代码（可能是整数或字符串）
fn deserialize_province_code<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    
    struct ProvinceCodeVisitor;
    
    impl<'de> Visitor<'de> for ProvinceCodeVisitor {
        type Value = String;
        
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or integer")
        }
        
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
        
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
        
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.to_string())
        }
    }
    
    deserializer.deserialize_any(ProvinceCodeVisitor)
}

/// 云听 API 响应结构
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: Option<String>,
    pub data: Option<T>,
}

/// 云听电台原始数据
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawStation {
    pub content_id: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub play_url_low: Option<String>,
    #[serde(default)]
    pub mp3_play_url_low: Option<String>,
    #[serde(default)]
    pub mp3_play_url_high: Option<String>,
}

impl RawStation {
    /// 转换为 Station 结构
    pub fn into_station(self, province: &str) -> Station {
        Station {
            id: self.content_id,
            name: self.title,
            subtitle: self.subtitle.unwrap_or_default(),
            image: self.image.unwrap_or_default(),
            province: province.to_string(),
            play_url_low: self.play_url_low,
            mp3_play_url_low: self.mp3_play_url_low,
            mp3_play_url_high: self.mp3_play_url_high,
        }
    }
}

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub active_streams: usize,
    pub total_stations: usize,
}

/// 爬虫进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlProgress {
    pub current: usize,
    pub total: usize,
    pub province: String,
    pub stations_found: usize,
}
