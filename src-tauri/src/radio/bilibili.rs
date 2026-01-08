//! B站视频音频获取模块
//!
//! 提供搜索B站视频并获取音频流URL的功能

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// B站 API 客户端
pub struct BilibiliApi {
    client: reqwest::Client,
}

/// 搜索结果中的视频信息
#[derive(Debug, Clone, Deserialize)]
pub struct SearchVideoResult {
    pub bvid: String,
    pub title: String,
    pub author: String,
    pub duration: String,
    pub play: u64,
    pub pic: String,
}

/// 分页列表响应
#[derive(Debug, Deserialize)]
struct PageListResponse {
    code: i32,
    data: Option<Vec<PageInfo>>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    cid: u64,
    part: String,
    duration: u64,
}

/// 播放URL响应
#[derive(Debug, Deserialize)]
struct PlayUrlResponse {
    code: i32,
    data: Option<PlayUrlData>,
}

#[derive(Debug, Deserialize)]
struct PlayUrlData {
    dash: Option<DashInfo>,
}

#[derive(Debug, Deserialize)]
struct DashInfo {
    audio: Option<Vec<AudioStream>>,
}

#[derive(Debug, Deserialize)]
struct AudioStream {
    id: u32,
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    #[serde(rename = "backupUrl")]
    backup_url: Option<Vec<String>>,
    // 兼容两种命名风格
    base_url_alt: Option<String>,
    backup_url_alt: Option<Vec<String>>,
}

/// 搜索响应
#[derive(Debug, Deserialize)]
struct SearchResponse {
    code: i32,
    data: Option<SearchData>,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    result: Option<Vec<SearchVideoResult>>,
}

/// 当前播放的视频信息
#[derive(Debug, Clone, Serialize)]
pub struct CurrentVideo {
    pub bvid: String,
    pub title: String,
    pub author: String,
    pub audio_url: String,
}

impl BilibiliApi {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();
        
        Self { client }
    }

    /// 搜索视频
    pub async fn search_videos(&self, keyword: &str, page: u32) -> anyhow::Result<Vec<SearchVideoResult>> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/search/type?search_type=video&keyword={}&page={}&duration=4",
            urlencoding::encode(keyword),
            page
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let search_resp: SearchResponse = resp.json().await?;
        
        if search_resp.code != 0 {
            anyhow::bail!("搜索失败，错误码: {}", search_resp.code);
        }

        Ok(search_resp.data
            .and_then(|d| d.result)
            .unwrap_or_default())
    }

    /// 获取视频的 CID
    pub async fn get_video_cid(&self, bvid: &str) -> anyhow::Result<u64> {
        let url = format!(
            "https://api.bilibili.com/x/player/pagelist?bvid={}",
            bvid
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let page_resp: PageListResponse = resp.json().await?;
        
        if page_resp.code != 0 {
            anyhow::bail!("获取CID失败，错误码: {}", page_resp.code);
        }

        page_resp.data
            .and_then(|pages| pages.first().map(|p| p.cid))
            .ok_or_else(|| anyhow::anyhow!("无法获取视频CID"))
    }

    /// 获取音频流URL
    /// 优先使用 backupUrl（根据用户反馈这个更快）
    pub async fn get_audio_url(&self, bvid: &str, cid: u64) -> anyhow::Result<String> {
        // fnval=16 获取 DASH 格式（音视频分离）
        // 不能用 platform=html5，那个只返回 MP4 格式
        let url = format!(
            "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&fnval=16&fnver=0&fourk=1",
            bvid, cid
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let play_resp: PlayUrlResponse = resp.json().await?;
        
        if play_resp.code != 0 {
            anyhow::bail!("获取播放URL失败，错误码: {}", play_resp.code);
        }

        let data = play_resp.data.ok_or_else(|| anyhow::anyhow!("无播放数据"))?;
        let dash = data.dash.ok_or_else(|| anyhow::anyhow!("无DASH数据"))?;
        let audio_list = dash.audio.ok_or_else(|| anyhow::anyhow!("无音频流"))?;

        // 找到最高质量的音频流
        let best_audio = audio_list.iter()
            .max_by_key(|a| a.id)
            .ok_or_else(|| anyhow::anyhow!("音频流列表为空"))?;

        // 优先使用 backupUrl（用户说这个更快）
        if let Some(backup_urls) = &best_audio.backup_url {
            if let Some(url) = backup_urls.first() {
                return Ok(url.clone());
            }
        }

        // 其次使用 baseUrl
        if let Some(base_url) = &best_audio.base_url {
            return Ok(base_url.clone());
        }

        anyhow::bail!("无法获取音频URL")
    }

    /// 搜索并随机选择一个视频，返回其音频URL
    /// 模拟电台效果：搜索关键词的视频，随机选一个播放
    pub async fn get_random_audio(&self, keyword: &str) -> anyhow::Result<CurrentVideo> {
        log::info!("🔍 搜索B站视频: {}", keyword);
        
        // 随机选择页码（1-10页）增加随机性
        let page = rand::random::<u32>() % 10 + 1;
        
        let videos = self.search_videos(keyword, page).await?;
        
        if videos.is_empty() {
            anyhow::bail!("未找到相关视频");
        }

        // 随机选择一个视频
        let video = videos.choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow::anyhow!("随机选择失败"))?;

        log::info!("🎲 随机选中: {} - {}", video.author, video.title);

        // 获取 CID
        let cid = self.get_video_cid(&video.bvid).await?;
        log::info!("📋 获取CID: {}", cid);

        // 获取音频URL
        let audio_url = self.get_audio_url(&video.bvid, cid).await?;
        log::info!("🎵 获取音频URL成功");

        Ok(CurrentVideo {
            bvid: video.bvid.clone(),
            title: video.title.replace("<em class=\"keyword\">", "").replace("</em>", ""),
            author: video.author.clone(),
            audio_url,
        })
    }
}

impl Default for BilibiliApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let api = BilibiliApi::new();
        let results = api.search_videos("郭德纲", 1).await;
        println!("{:?}", results);
    }
}
