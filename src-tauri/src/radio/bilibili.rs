//! B站视频音频获取模块
//!
//! 提供搜索B站视频并获取音频流URL的功能
//! 支持合集连续播放

#![allow(dead_code)]

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// B站 API 客户端
pub struct BilibiliApi {
    client: reqwest::Client,
}

/// 搜索结果中的视频信息
#[derive(Debug, Clone, Deserialize)]
pub struct SearchVideoResult {
    #[serde(default)]
    pub bvid: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub play: u64,
    #[serde(default)]
    pub pic: String,
    // aid 也可能存在
    #[serde(default)]
    pub aid: u64,
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

impl BilibiliApi {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// 搜索视频（带重试）
    pub async fn search_videos(
        &self,
        keyword: &str,
        page: u32,
    ) -> anyhow::Result<Vec<SearchVideoResult>> {
        // 最多重试 3 次
        let mut last_error = None;
        for attempt in 0..3 {
            match self.search_videos_once(keyword, page).await {
                Ok(results) => return Ok(results),
                Err(e) => {
                    log::warn!("   搜索尝试 {} 失败: {}", attempt + 1, e);
                    last_error = Some(e);
                    // 短暂延迟后重试
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }
        Err(last_error.unwrap())
    }

    /// 单次搜索视频
    async fn search_videos_once(
        &self,
        keyword: &str,
        page: u32,
    ) -> anyhow::Result<Vec<SearchVideoResult>> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/search/type?search_type=video&keyword={}&page={}&duration=4",
            urlencoding::encode(keyword),
            page
        );

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .header("Origin", "https://www.bilibili.com")
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        // 检查是否是空响应
        if text.is_empty() {
            anyhow::bail!("B站返回空响应，状态码: {}", status);
        }

        // 尝试解析 JSON
        let search_resp: SearchResponse = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                log::error!("   解析搜索结果失败: {}", e);
                // 输出前 200 个字符帮助调试
                log::error!("   响应内容: {}", &text[..text.len().min(200)]);
                anyhow::bail!("解析搜索结果失败: {}", e);
            }
        };

        if search_resp.code != 0 {
            anyhow::bail!("搜索失败，错误码: {}", search_resp.code);
        }

        // 过滤掉 bvid 为空的结果
        let results = search_resp
            .data
            .and_then(|d| d.result)
            .unwrap_or_default()
            .into_iter()
            .filter(|v| !v.bvid.is_empty())
            .collect();

        Ok(results)
    }

    /// 获取视频详细信息（包含合集信息）
    async fn get_video_info(&self, bvid: &str) -> anyhow::Result<VideoInfoData> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        );

        let resp = self
            .client
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
        let url = format!("https://api.bilibili.com/x/player/pagelist?bvid={}", bvid);

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let page_resp: PageListResponse = resp.json().await?;

        if page_resp.code != 0 {
            anyhow::bail!("获取CID失败，错误码: {}", page_resp.code);
        }

        page_resp
            .data
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

        let resp = self
            .client
            .get(&url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?;

        let play_resp: PlayUrlResponse = resp.json().await?;

        if play_resp.code != 0 {
            anyhow::bail!("获取播放URL失败，错误码: {}", play_resp.code);
        }

        let data = play_resp
            .data
            .ok_or_else(|| anyhow::anyhow!("无播放数据"))?;
        let dash = data.dash.ok_or_else(|| anyhow::anyhow!("无DASH数据"))?;
        let audio_list = dash.audio.ok_or_else(|| anyhow::anyhow!("无音频流"))?;

        // 找到最高质量的音频流
        let best_audio = audio_list
            .iter()
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
    async fn get_related_videos(&self, bvid: &str) -> anyhow::Result<Vec<RecommendVideo>> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/archive/related?bvid={}",
            bvid
        );

        let resp = self
            .client
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
        log::debug!("bilibili get next video: {}", current_bvid);

        // 获取当前视频详情（包含合集信息）
        let video_info = self.get_video_info(current_bvid).await?;

        // 检查是否有合集
        if let Some(ugc_season) = &video_info.ugc_season {
            log::debug!("bilibili season: {}", ugc_season.title);

            // 遍历合集找到当前视频位置
            if let Some(sections) = &ugc_season.sections {
                for section in sections {
                    if let Some(episodes) = &section.episodes {
                        // 找到当前视频的索引
                        if let Some(current_idx) =
                            episodes.iter().position(|ep| ep.bvid == current_bvid)
                        {
                            // 获取下一个视频
                            if current_idx + 1 < episodes.len() {
                                let next_ep = &episodes[current_idx + 1];
                                log::debug!("bilibili season next: {}", next_ep.title);

                                // 获取音频URL
                                let audio_url =
                                    self.get_audio_url(&next_ep.bvid, next_ep.cid).await?;

                                return Ok(CurrentVideo {
                                    bvid: next_ep.bvid.clone(),
                                    title: next_ep.title.clone(),
                                    author: video_info.owner.name.clone(),
                                    audio_url,
                                    cid: next_ep.cid,
                                });
                            } else {
                                log::debug!("bilibili season ended, using related videos");
                            }
                        }
                    }
                }
            }
        } else {
            log::debug!("bilibili no season, using related videos");
        }

        // Fallback: 使用推荐视频
        let related = self.get_related_videos(current_bvid).await?;

        if related.is_empty() {
            anyhow::bail!("无推荐视频");
        }

        // 取第一个推荐视频
        let next_video = &related[0];
        log::debug!("bilibili related video: {}", next_video.title);

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
        log::debug!("bilibili search: {}", keyword);

        // 随机选择页码（1-10页）增加随机性
        let page = rand::random::<u32>() % 10 + 1;

        let videos = self.search_videos(keyword, page).await?;

        if videos.is_empty() {
            anyhow::bail!("未找到相关视频");
        }

        // 随机选择一个视频
        let video = videos
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow::anyhow!("随机选择失败"))?;

        let title = video
            .title
            .replace("<em class=\"keyword\">", "")
            .replace("</em>", "");
        log::debug!("bilibili selected: {} - {}", video.author, title);

        // 获取视频详情（包含 CID）
        let video_info = self.get_video_info(&video.bvid).await?;
        log::debug!("bilibili cid: {}", video_info.cid);

        // 获取音频URL
        let audio_url = self.get_audio_url(&video.bvid, video_info.cid).await?;
        log::debug!("bilibili audio url ready");

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
