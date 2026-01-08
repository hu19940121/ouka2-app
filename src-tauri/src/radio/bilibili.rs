//! B站视频音频获取模块
//!
//! 提供搜索B站视频并获取音频流URL的功能
//! 支持合集连续播放

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

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

/// 视频详情响应
#[derive(Debug, Deserialize)]
struct VideoInfoResponse {
    code: i32,
    data: Option<VideoInfoData>,
}

#[derive(Debug, Deserialize)]
struct VideoInfoData {
    bvid: String,
    title: String,
    owner: VideoOwner,
    cid: u64,
    ugc_season: Option<UgcSeason>,
}

#[derive(Debug, Deserialize)]
struct VideoOwner {
    name: String,
    mid: u64,
}

/// 合集信息
#[derive(Debug, Deserialize)]
struct UgcSeason {
    id: u64,
    title: String,
    sections: Option<Vec<UgcSection>>,
}

#[derive(Debug, Deserialize)]
struct UgcSection {
    episodes: Option<Vec<UgcEpisode>>,
}

#[derive(Debug, Clone, Deserialize)]
struct UgcEpisode {
    aid: u64,
    bvid: String,
    title: String,
    cid: u64,
}

/// 推荐视频响应
#[derive(Debug, Deserialize)]
struct RecommendResponse {
    code: i32,
    data: Option<Vec<RecommendVideo>>,
}

#[derive(Debug, Deserialize)]
struct RecommendVideo {
    bvid: String,
    title: String,
    owner: VideoOwner,
    cid: u64,
}

/// 当前播放的视频信息
#[derive(Debug, Clone, Serialize)]
pub struct CurrentVideo {
    pub bvid: String,
    pub title: String,
    pub author: String,
    pub audio_url: String,
    pub cid: u64,
}

/// 郭德纲电台播放状态
pub struct GuodegangRadioState {
    /// 当前播放的视频 BVID
    pub current_bvid: Option<String>,
    /// 是否正在播放
    pub is_playing: bool,
}

impl Default for GuodegangRadioState {
    fn default() -> Self {
        Self {
            current_bvid: None,
            is_playing: false,
        }
    }
}

/// 全局电台状态
pub type RadioState = Arc<RwLock<GuodegangRadioState>>;

/// 创建新的电台状态
pub fn new_radio_state() -> RadioState {
    Arc::new(RwLock::new(GuodegangRadioState::default()))
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

    /// 获取视频详细信息（包含合集信息）
    pub async fn get_video_info(&self, bvid: &str) -> anyhow::Result<VideoInfoData> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let info_resp: VideoInfoResponse = resp.json().await?;
        
        if info_resp.code != 0 {
            anyhow::bail!("获取视频信息失败，错误码: {}", info_resp.code);
        }

        info_resp.data.ok_or_else(|| anyhow::anyhow!("无视频信息"))
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
    /// 优先使用 backupUrl（用户反馈这个更快）
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

        // 优先使用 backupUrl
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

    /// 获取推荐视频列表
    pub async fn get_related_videos(&self, bvid: &str) -> anyhow::Result<Vec<RecommendVideo>> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/archive/related?bvid={}",
            bvid
        );

        let resp = self.client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let rec_resp: RecommendResponse = resp.json().await?;
        
        if rec_resp.code != 0 {
            anyhow::bail!("获取推荐视频失败，错误码: {}", rec_resp.code);
        }

        Ok(rec_resp.data.unwrap_or_default())
    }

    /// 获取下一个视频
    /// 1. 优先从合集中获取下一个
    /// 2. 没有合集则使用推荐视频
    pub async fn get_next_video(&self, current_bvid: &str) -> anyhow::Result<CurrentVideo> {
        log::info!("🔄 获取下一个视频 (当前: {})", current_bvid);
        
        // 获取当前视频详情（包含合集信息）
        let video_info = self.get_video_info(current_bvid).await?;
        
        // 检查是否有合集
        if let Some(ugc_season) = &video_info.ugc_season {
            log::info!("   📚 视频在合集中: {}", ugc_season.title);
            
            // 遍历合集找到当前视频位置
            if let Some(sections) = &ugc_season.sections {
                for section in sections {
                    if let Some(episodes) = &section.episodes {
                        // 找到当前视频的索引
                        if let Some(current_idx) = episodes.iter()
                            .position(|ep| ep.bvid == current_bvid) 
                        {
                            // 获取下一个视频
                            if current_idx + 1 < episodes.len() {
                                let next_ep = &episodes[current_idx + 1];
                                log::info!("   ➡️ 合集下一个: {}", next_ep.title);
                                
                                // 获取音频URL
                                let audio_url = self.get_audio_url(&next_ep.bvid, next_ep.cid).await?;
                                
                                return Ok(CurrentVideo {
                                    bvid: next_ep.bvid.clone(),
                                    title: next_ep.title.clone(),
                                    author: video_info.owner.name.clone(),
                                    audio_url,
                                    cid: next_ep.cid,
                                });
                            } else {
                                log::info!("   ⚠️ 已是合集最后一个，使用推荐视频");
                            }
                        }
                    }
                }
            }
        } else {
            log::info!("   ℹ️ 视频不在合集中，使用推荐视频");
        }
        
        // Fallback: 使用推荐视频
        let related = self.get_related_videos(current_bvid).await?;
        
        if related.is_empty() {
            anyhow::bail!("无推荐视频");
        }
        
        // 取第一个推荐视频
        let next_video = &related[0];
        log::info!("   ➡️ 推荐视频: {}", next_video.title);
        
        let audio_url = self.get_audio_url(&next_video.bvid, next_video.cid).await?;
        
        Ok(CurrentVideo {
            bvid: next_video.bvid.clone(),
            title: next_video.title.clone(),
            author: next_video.owner.name.clone(),
            audio_url,
            cid: next_video.cid,
        })
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

        let title = video.title
            .replace("<em class=\"keyword\">", "")
            .replace("</em>", "");
        log::info!("🎲 随机选中: {} - {}", video.author, title);

        // 获取视频详情（包含 CID）
        let video_info = self.get_video_info(&video.bvid).await?;
        log::info!("📋 获取CID: {}", video_info.cid);

        // 获取音频URL
        let audio_url = self.get_audio_url(&video.bvid, video_info.cid).await?;
        log::info!("🎵 获取音频URL成功");

        Ok(CurrentVideo {
            bvid: video.bvid.clone(),
            title,
            author: video.author.clone(),
            audio_url,
            cid: video_info.cid,
        })
    }
}

impl Default for BilibiliApi {
    fn default() -> Self {
        Self::new()
    }
}
