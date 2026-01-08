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
    /// 服务器端口
    pub port: u16,
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
            port,
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
            port: self.port,
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
        let port = self.port;

        // 构建路由
        let app = Router::new()
            .route("/stream/:id", get(handle_stream))
            .route("/health", get(handle_health))
            .route("/api/stations", get(handle_stations_api))
            .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
            .with_state(state);

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        log::info!("🚀 流媒体服务器启动: http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;

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
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            self.is_running = false;
            log::info!("🛑 流媒体服务器已停止");
        }
    }
}

/// B站测试音频 URL (写死用于测试)
const BILIBILI_TEST_AUDIO_URL: &str = "https://xy111x2x118x34xy.mcdn.bilivideo.cn:8082/v1/resource/35262169982-1-30232.m4s?agrr=0&build=0&buvid=313FEBD1-FE42-EC1F-B185-568B724F7DD238598infoc&bvc=vod&bw=71972&deadline=1767885032&dl=0&e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M%3D&f=u_0_0&gen=playurlv3&mid=340568785&nbs=1&nettype=0&og=cos&oi=1879754545&orderid=0%2C3&os=cosbv&platform=pc&qn_dyeid=e631e7824ae7ae1700190638695facc8&sign=4f8adc&traceid=trLjrErAiDpMdx_0_e_N&uipk=5&uparams=e%2Ctrid%2Cdeadline%2Cuipk%2Coi%2Cnbs%2Cos%2Cplatform%2Cmid%2Cgen%2Cog&upsig=8f4384088f36ec283468b7cd1ae2ff46";

/// 处理流媒体请求
async fn handle_stream(
    Path(station_id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
    // 🎬 B站测试频道：使用写死的音频 URL
    if station_id == "bilibili_test" {
        log::info!("🎬 B站测试频道 - 使用写死的音频 URL");
        return handle_bilibili_stream(state, "B站测试频道", BILIBILI_TEST_AUDIO_URL).await;
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

    // 刷新流地址
    let stream_url = match state
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
    let mut list: Vec<_> = stations
        .values()
        .map(|s| {
            let mut s = s.clone();
            // 添加本地流地址
            s.mp3_play_url_high = Some(format!("http://127.0.0.1:{}/stream/{}", state.port, s.id));
            s
        })
        .collect();
    
    // 添加 B站测试频道
    list.push(Station {
        id: "bilibili_test".to_string(),
        name: "🎬 B站测试频道".to_string(),
        subtitle: "测试 B站视频音频播放".to_string(),
        image: "https://www.bilibili.com/favicon.ico".to_string(),
        province: "test".to_string(),
        play_url_low: None,
        mp3_play_url_low: None,
        mp3_play_url_high: Some(format!("http://127.0.0.1:{}/stream/bilibili_test", state.port)),
    });
    
    axum::Json(list)
}

/// 处理 B站音频流
async fn handle_bilibili_stream(
    state: Arc<ServerState>,
    name: &str,
    audio_url: &str,
) -> Response {
    log::info!("   📡 B站音频地址: {}...", &audio_url[..audio_url.len().min(80)]);

    // 启动 FFmpeg 进程 - B站音频需要特殊处理
    let ffmpeg_path = &state.ffmpeg_path;

    let mut child = match spawn_ffmpeg_for_bilibili(ffmpeg_path, audio_url) {
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
            .insert("bilibili_test".to_string(), pid);
    }

    // 获取输出流
    let stdout = child.stdout.take().expect("无法获取 stdout");

    // 创建流式响应
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, std::io::Error>>(32);

    // 在后台读取 FFmpeg 输出
    let state_clone = state.clone();
    let name_owned = name.to_string();
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
            .remove("bilibili_test");
        log::info!("🔇 {} 流已关闭", name_owned);
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
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    let child = cmd.spawn()?;
    Ok(child)
}
