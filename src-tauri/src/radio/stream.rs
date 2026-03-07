//! 流媒体转发服务器
//!
//! 使用 axum 创建嵌入式 HTTP 服务器，通过 FFmpeg 将 m3u8 流转换为 MP3

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};

use crate::radio::api::RadioApi;
use crate::radio::models::{ServerStatus, Station};

/// 服务器共享状态
pub struct ServerState {
    /// 电台列表
    pub stations: RwLock<HashMap<String, Station>>,
    /// 活动的 FFmpeg 进程
    pub active_streams: RwLock<HashMap<String, u32>>, // station_id -> process_id
    /// 服务器端口（可动态更新）
    pub port: RwLock<u16>,
    /// FFmpeg 路径
    pub ffmpeg_path: PathBuf,
    /// API 客户端（用于刷新流地址）
    pub api: RadioApi,
}

impl ServerState {
    pub fn new(port: u16, ffmpeg_path: PathBuf) -> Self {
        Self {
            stations: RwLock::new(HashMap::new()),
            active_streams: RwLock::new(HashMap::new()),
            port: RwLock::new(port),
            ffmpeg_path,
            api: RadioApi::new(),
        }
    }

    /// 加载电台数据
    pub async fn load_stations(&self, stations: Vec<Station>) {
        let mut map = self.stations.write().await;
        map.clear();
        for station in stations {
            map.insert(station.id.clone(), station);
        }
    }

    /// 获取服务器状态
    pub async fn get_status(&self) -> ServerStatus {
        ServerStatus {
            running: true,
            port: *self.port.read().await,
            active_streams: self.active_streams.read().await.len(),
            total_stations: self.stations.read().await.len(),
        }
    }
}

/// 流媒体服务器
pub struct StreamServer {
    port: u16,
    state: Arc<ServerState>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    is_running: bool,
}

impl StreamServer {
    /// 创建新的服务器实例
    pub fn new(port: u16, ffmpeg_path: PathBuf) -> Self {
        Self {
            port,
            state: Arc::new(ServerState::new(port, ffmpeg_path)),
            shutdown_tx: None,
            is_running: false,
        }
    }

    /// 检查服务器是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// 获取共享状态
    pub fn state(&self) -> Arc<ServerState> {
        self.state.clone()
    }

    /// 启动服务器
    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.is_running {
            return Ok(());
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(tx);

        let state = self.state.clone();
        
        // 尝试绑定端口，如果被占用就自动切换
        let mut port = self.port;
        let max_attempts = 10; // 最多尝试 10 个端口
        let mut listener = None;
        
        for attempt in 0..max_attempts {
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => {
                    if attempt > 0 {
                        log::info!("📌 端口 {} 被占用，自动切换到端口 {}", self.port, port);
                    }
                    listener = Some(l);
                    break;
                }
                Err(e) => {
                    log::warn!("⚠️ 端口 {} 不可用: {}", port, e);
                    port += 1;
                }
            }
        }
        
        let listener = listener.ok_or_else(|| {
            anyhow::anyhow!("无法找到可用端口 (尝试了 {} 到 {})", self.port, self.port + max_attempts as u16 - 1)
        })?;
        
        // 更新实际使用的端口
        self.port = port;
        
        // 同时更新 state 中的端口
        {
            let mut state_port = self.state.port.write().await;
            *state_port = port;
        }
        
        log::info!("🚀 流媒体服务器启动: http://127.0.0.1:{}", port);

        // 构建路由
        let app = Router::new()
            .route("/stream/:id", get(handle_stream))
            .route("/health", get(handle_health))
            .route("/api/stations", get(handle_stations_api))
            .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
            .with_state(state);

        // 在后台运行服务器
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    rx.await.ok();
                })
                .await
                .ok();
        });

        self.is_running = true;
        Ok(())
    }

    /// 停止服务器
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            // 先杀死所有活动的 FFmpeg 进程
            let active_streams = self.state.active_streams.read().await;
            for (station_id, pid) in active_streams.iter() {
                log::info!("   🔪 终止流: {} (PID: {})", station_id, pid);
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string()])
                        .output();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("kill")
                        .args(["-9", &pid.to_string()])
                        .output();
                }
            }
            drop(active_streams);
            
            // 清空活动流列表
            self.state.active_streams.write().await.clear();
            
            // 发送停止信号
            let _ = tx.send(());
            
            // 等待一小段时间让端口释放
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            self.is_running = false;
            log::info!("🛑 流媒体服务器已停止");
        }
    }
}

use crate::radio::bilibili::BilibiliApi;

/// 郭德纲电台当前播放的 BVID（用于续播）
static GUODEGANG_CURRENT_BVID: std::sync::OnceLock<tokio::sync::RwLock<Option<String>>> = std::sync::OnceLock::new();

fn get_current_bvid_lock() -> &'static tokio::sync::RwLock<Option<String>> {
    GUODEGANG_CURRENT_BVID.get_or_init(|| tokio::sync::RwLock::new(None))
}

/// 处理流媒体请求
async fn handle_stream(
    Path(station_id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    // 🎙️ 郭德纲电台：动态搜索B站视频并随机播放
    if station_id == "guodegang_radio" {
        return handle_guodegang_radio(state).await;
    }
    
    // 如果切换到其他频道，清除郭德纲电台状态
    {
        let mut current_bvid = get_current_bvid_lock().write().await;
        if current_bvid.is_some() {
            log::info!("🔄 切换频道，清除郭德纲电台状态");
            *current_bvid = None;
        }
    }

    // 查找电台
    let station = {
        let stations = state.stations.read().await;
        stations.get(&station_id).cloned()
    };

    let station = match station {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, "电台未找到").into_response();
        }
    };

    log::info!("🎵 开始转发: {}", station.name);

    // 获取流地址：自定义电台直接用缓存地址，普通电台刷新
    let stream_url = if station.is_custom {
        log::info!("   📌 自定义电台，使用配置地址");
        match station.get_best_stream_url() {
            Some(url) => url.to_string(),
            None => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "自定义电台无流地址").into_response();
            }
        }
    } else {
        // 刷新流地址
        match state
            .api
            .refresh_stream_url(&station_id, &station.province)
            .await
        {
            Ok(Some(url)) => {
                log::info!("   ✅ 获取到新地址");
                url
            }
            Ok(None) => {
                // 使用缓存的地址
                log::warn!("   ⚠️ 刷新失败，使用缓存地址");
                match station.get_best_stream_url() {
                    Some(url) => url.to_string(),
                    None => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "无可用流地址").into_response();
                    }
                }
            }
            Err(e) => {
                log::error!("   ❌ 刷新流地址失败: {}", e);
                match station.get_best_stream_url() {
                    Some(url) => url.to_string(),
                    None => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "无可用流地址").into_response();
                    }
                }
            }
        }
    };

    log::info!(
        "   📡 流地址: {}...",
        &stream_url[..stream_url.len().min(80)]
    );

    // 启动 FFmpeg 进程
    let ffmpeg_path = &state.ffmpeg_path;

    let mut child = match spawn_ffmpeg(ffmpeg_path, &stream_url) {
        Ok(child) => child,
        Err(e) => {
            log::error!("   ❌ 启动 FFmpeg 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("启动 FFmpeg 失败: {}", e),
            )
                .into_response();
        }
    };

    // 记录活动进程
    if let Some(pid) = child.id() {
        state
            .active_streams
            .write()
            .await
            .insert(station_id.clone(), pid);
    }

    // 获取输出流
    let stdout = child.stdout.take().expect("无法获取 stdout");

    // 创建流式响应
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, std::io::Error>>(32);

    // 在后台读取 FFmpeg 输出
    let station_id_clone = station_id.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut buffer = [0u8; 4096];

        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if tx.send(Ok(buffer[..n].to_vec())).await.is_err() {
                        break; // 接收端已关闭
                    }
                }
                Err(e) => {
                    log::error!("读取 FFmpeg 输出错误: {}", e);
                    let _ = tx.send(Err(e)).await;
                    break;
                }
            }
        }

        // 清理
        let _ = child.kill().await;
        state_clone
            .active_streams
            .write()
            .await
            .remove(&station_id_clone);
        log::info!("🔇 {} 流已关闭", station_id_clone);
    });

    // 构建响应
    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header(
            "icy-name",
            urlencoding::encode(&station.name).to_string(),
        )
        .body(body)
        .unwrap()
}

/// 启动 FFmpeg 转码进程
fn spawn_ffmpeg(ffmpeg_path: &PathBuf, stream_url: &str) -> anyhow::Result<Child> {
    let mut cmd = Command::new(ffmpeg_path);
    
    cmd.args([
        "-reconnect",
        "1",
        "-reconnect_streamed",
        "1",
        "-reconnect_delay_max",
        "5",
        "-i",
        stream_url,
        "-vn",
        "-acodec",
        "libmp3lame",
        "-ab",
        "128k",
        "-ar",
        "44100",
        "-ac",
        "2",
        "-f",
        "mp3",
        "-fflags",
        "+nobuffer+discardcorrupt",
        "-flags",
        "low_delay",
        "-flush_packets",
        "1",
        "pipe:1",
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .kill_on_drop(true);
    
    // Windows: 隐藏控制台窗口
    #[cfg(target_os = "windows")]
    {
        #[allow(unused_imports)]
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    let child = cmd.spawn()?;
    Ok(child)
}

/// 健康检查端点
async fn handle_health(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let status = state.get_status().await;
    axum::Json(status)
}

/// 电台列表 API
async fn handle_stations_api(State(state): State<Arc<ServerState>>) -> impl IntoResponse {
    let stations = state.stations.read().await;
    let port = *state.port.read().await;
    let mut list: Vec<_> = stations
        .values()
        .map(|s| {
            let mut s = s.clone();
            // 添加本地流地址
            s.mp3_play_url_high = Some(format!("http://127.0.0.1:{}/stream/{}", port, s.id));
            s
        })
        .collect();
    
    // 添加郭德纲电台
    list.push(Station {
        id: "guodegang_radio".to_string(),
        name: "郭德纲电台".to_string(),
        subtitle: "随机播放B站郭德纲相声".to_string(),
        image: "https://i0.hdslb.com/bfs/face/a6a0bb6eb6a52b96f5ea0e5b6a0a6ff3d74e55cb.jpg".to_string(),
        province: "bilibili".to_string(),
        play_url_low: None,
        mp3_play_url_low: None,
        mp3_play_url_high: Some(format!("http://127.0.0.1:{}/stream/guodegang_radio", port)),
        is_custom: false,
    });
    
    axum::Json(list)
}

/// 处理郭德纲电台请求
async fn handle_guodegang_radio(state: Arc<ServerState>) -> Response {
    let bilibili_api = BilibiliApi::new();
    
    // 检查是否有正在播放的状态（用于续播）
    let current_bvid = {
        let lock = get_current_bvid_lock().read().await;
        lock.clone()
    };
    
    let video = if let Some(bvid) = current_bvid {
        // 有当前播放状态，尝试获取下一个视频
        log::info!("郭德纲电台 - 获取下一个节目 (当前: {})", bvid);
        
        match bilibili_api.get_next_video(&bvid).await {
            Ok(video) => {
                log::info!("   ➡️ 下一个: {} - {}", video.author, video.title);
                video
            }
            Err(e) => {
                log::warn!("   ⚠️ 获取下一个失败: {}，重新随机搜索", e);
                // 失败时重新随机搜索
                match bilibili_api.get_random_audio("郭德纲 相声").await {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("   ❌ 随机搜索也失败了: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, format!("获取节目失败: {}", e)).into_response();
                    }
                }
            }
        }
    } else {
        // 没有播放状态，首次随机搜索
        log::info!("郭德纲电台 - 首次随机搜索节目...");
        
        match bilibili_api.get_random_audio("郭德纲 相声").await {
            Ok(video) => {
                log::info!("   🎲 随机选中: {} - {}", video.author, video.title);
                video
            }
            Err(e) => {
                log::error!("   ❌ 获取节目失败: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("获取节目失败: {}", e)).into_response();
            }
        }
    };
    
    // 更新当前播放状态
    {
        let mut lock = get_current_bvid_lock().write().await;
        *lock = Some(video.bvid.clone());
    }
    
    let title = format!("郭德纲电台: {}", video.title);
    handle_bilibili_stream_with_callback(state, &title, &video.audio_url, &video.bvid).await
}

/// 处理 B站音频流（带自动续播）
async fn handle_bilibili_stream_with_callback(
    state: Arc<ServerState>,
    name: &str,
    audio_url: &str,
    bvid: &str,
) -> Response {
    log::info!("   📡 B站音频地址: {}...", &audio_url[..audio_url.len().min(80)]);

    // 创建流式响应
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, std::io::Error>>(32);

    // 在后台处理播放和自动续播
    let state_clone = state.clone();
    let name_owned = name.to_string();
    let bvid_owned = bvid.to_string();
    let audio_url_owned = audio_url.to_string();
    
    tokio::spawn(async move {
        let mut current_bvid = bvid_owned;
        let mut current_audio_url = audio_url_owned;
        let mut current_name = name_owned;
        
        // 循环播放：播放完一个自动获取下一个
        loop {
            log::info!("🎵 开始播放: {}", current_name);
            
            // 启动 FFmpeg 进程
            let ffmpeg_path = &state_clone.ffmpeg_path;
            let mut child = match spawn_ffmpeg_for_bilibili(ffmpeg_path, &current_audio_url) {
                Ok(child) => child,
                Err(e) => {
                    log::error!("   ❌ 启动 FFmpeg 失败: {}", e);
                    break;
                }
            };

            // 记录活动进程
            if let Some(pid) = child.id() {
                state_clone
                    .active_streams
                    .write()
                    .await
                    .insert("guodegang_radio".to_string(), pid);
            }

            // 获取输出流
            let stdout = match child.stdout.take() {
                Some(s) => s,
                None => {
                    log::error!("无法获取 FFmpeg stdout");
                    break;
                }
            };

            // 读取并发送音频数据
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut buffer = [0u8; 4096];
            let mut client_disconnected = false;

            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => break, // EOF - 当前视频播放完毕
                    Ok(n) => {
                        if tx.send(Ok(buffer[..n].to_vec())).await.is_err() {
                            client_disconnected = true;
                            break; // 客户端已断开
                        }
                    }
                    Err(e) => {
                        log::error!("读取 FFmpeg 输出错误: {}", e);
                        break;
                    }
                }
            }

            // 等待 FFmpeg 进程完全退出并获取状态
            let exit_status = match child.wait().await {
                Ok(status) => status,
                Err(e) => {
                    log::error!("   ❌ 无法获取 FFmpeg 退出状态: {}", e);
                    // 假装它是成功的，继续下一个，避免死锁
                    std::os::unix::process::ExitStatusExt::from_raw(0) 
                }
            };
            
            // 移除活动流标记（清理内存应在 wait 之后，但在逻辑流转之前）
            state_clone
                .active_streams
                .write()
                .await
                .remove("guodegang_radio");

            // 如果客户端断开，停止循环
            if client_disconnected {
                log::info!("🔇 客户端断开连接，停止播放");
                break;
            }

            // 检查 FFmpeg 是否异常退出
            if !exit_status.success() {
                log::warn!("   ⚠️ 当前节目 FFmpeg 异常退出 (Code: {:?})，可能是源失效或网络问题", exit_status.code());
                // 如果是异常退出，暂停 3 秒再试，防止死循环刷爆 API
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            } else {
                log::info!("🔄 {} 播放完毕，获取下一个...", current_name);
            }

            // 获取下一个视频
            let bilibili_api = BilibiliApi::new();
            match bilibili_api.get_next_video(&current_bvid).await {
                Ok(next_video) => {
                    log::info!("   ➡️ 下一个: {} - {}", next_video.author, next_video.title);
                    
                    // 更新当前播放状态
                    {
                        let mut lock = get_current_bvid_lock().write().await;
                        *lock = Some(next_video.bvid.clone());
                    }
                    
                    // 更新循环变量
                    current_bvid = next_video.bvid;
                    current_audio_url = next_video.audio_url;
                    current_name = format!("郭德纲电台: {}", next_video.title);
                }
                Err(e) => {
                    log::error!("   ❌ 获取下一个视频失败: {}", e);
                    // 尝试重新随机搜索
                    match bilibili_api.get_random_audio("郭德纲 相声").await {
                        Ok(video) => {
                            log::info!("   🎲 重新随机: {} - {}", video.author, video.title);
                            {
                                let mut lock = get_current_bvid_lock().write().await;
                                *lock = Some(video.bvid.clone());
                            }
                            current_bvid = video.bvid;
                            current_audio_url = video.audio_url;
                            current_name = format!("郭德纲电台: {}", video.title);
                        }
                        Err(e2) => {
                            log::error!("   ❌ 重新随机也失败: {}，停止播放", e2);
                            break;
                        }
                    }
                }
            }
        }
        
        log::info!("🔇 郭德纲电台播放结束");
    });

    // 构建响应
    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header("icy-name", urlencoding::encode(name).to_string())
        .body(body)
        .unwrap()
}

/// 启动 FFmpeg 转码进程 (B站音频专用)
/// B站的 m4s 格式需要添加 User-Agent 和 Referer
fn spawn_ffmpeg_for_bilibili(ffmpeg_path: &PathBuf, audio_url: &str) -> anyhow::Result<Child> {
    let mut cmd = Command::new(ffmpeg_path);
    
    cmd.args([
        // 添加 User-Agent
        "-user_agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        // 添加 Referer (B站防盗链)
        "-headers",
        "Referer: https://www.bilibili.com/\r\n",
        // 限制读取速度为原生速率 (关键！防止服务器全速转码导致逻辑超前)
        "-re",
        // 重连设置
        "-reconnect",
        "1",
        "-reconnect_streamed",
        "1",
        "-reconnect_delay_max",
        "5",
        // 输入
        "-i",
        audio_url,
        // 只要音频
        "-vn",
        // 编码设置
        "-acodec",
        "libmp3lame",
        "-ab",
        "128k",
        "-ar",
        "44100",
        "-ac",
        "2",
        "-f",
        "mp3",
        // 低延迟设置
        "-fflags",
        "+nobuffer+discardcorrupt",
        "-flags",
        "low_delay",
        "-flush_packets",
        "1",
        "pipe:1",
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .kill_on_drop(true);
    
    // Windows: 隐藏控制台窗口
    #[cfg(target_os = "windows")]
    {
        #[allow(unused_imports)]
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    let child = cmd.spawn()?;
    Ok(child)
}
