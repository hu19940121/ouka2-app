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
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};

use crate::radio::api::RadioApi;
use crate::radio::models::{ServerStatus, Station};

static NEXT_STREAM_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// 单次播放请求对应的活动流信息。
pub struct ActiveStream {
    pub station_id: String,
    pub process_id: u32,
}

fn next_stream_request_id(station_id: &str) -> String {
    let id = NEXT_STREAM_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", station_id, id)
}

fn kill_stream_process(process_id: u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &process_id.to_string()])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-9", &process_id.to_string()])
            .output();
    }
}

/// 服务器共享状态
pub struct ServerState {
    /// 电台列表
    pub stations: RwLock<HashMap<String, Station>>,
    /// 活动的 FFmpeg 进程
    pub active_streams: RwLock<HashMap<String, ActiveStream>>, // request_id -> stream
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

    /// 停止当前所有活动流，但不关闭 HTTP 服务器。
    pub async fn stop_active_streams(&self) {
        let active_streams: Vec<_> = {
            let mut streams = self.active_streams.write().await;
            streams.drain().collect()
        };
        let count = active_streams.len();

        for (request_id, stream) in active_streams {
            log::debug!(
                "stop stream: {} / {} (pid: {})",
                request_id,
                stream.station_id,
                stream.process_id
            );
            kill_stream_process(stream.process_id);
        }

        if count > 0 {
            log::debug!("stopped active streams: {}", count);
        }
    }

    /// 停止指定电台的旧活动流，用于收敛 WebView 对同一音频源发出的重复请求。
    pub async fn stop_streams_for_station(&self, station_id: &str) -> bool {
        let active_streams: Vec<_> = {
            let mut streams = self.active_streams.write().await;
            let request_ids: Vec<_> = streams
                .iter()
                .filter(|(_, stream)| stream.station_id == station_id)
                .map(|(request_id, _)| request_id.clone())
                .collect();

            request_ids
                .into_iter()
                .filter_map(|request_id| {
                    streams
                        .remove(&request_id)
                        .map(|stream| (request_id, stream))
                })
                .collect()
        };
        let stopped_any = !active_streams.is_empty();

        for (request_id, stream) in active_streams {
            log::debug!(
                "stop duplicate stream: {} / {} (pid: {})",
                request_id,
                stream.station_id,
                stream.process_id
            );
            kill_stream_process(stream.process_id);
        }

        stopped_any
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

    /// 停止当前所有活动流，但保持服务器运行。
    pub async fn stop_active_streams(&self) {
        self.state.stop_active_streams().await;
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
                        log::info!("端口 {} 被占用，自动切换到 {}", self.port, port);
                    }
                    listener = Some(l);
                    break;
                }
                Err(e) => {
                    log::warn!("端口 {} 不可用: {}", port, e);
                    port += 1;
                }
            }
        }

        let listener = listener.ok_or_else(|| {
            anyhow::anyhow!(
                "无法找到可用端口 (尝试了 {} 到 {})",
                self.port,
                self.port + max_attempts as u16 - 1
            )
        })?;

        // 更新实际使用的端口
        self.port = port;

        // 同时更新 state 中的端口
        {
            let mut state_port = self.state.port.write().await;
            *state_port = port;
        }

        log::info!("流媒体服务器已启动: http://127.0.0.1:{}", port);

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
            self.stop_active_streams().await;

            // 发送停止信号
            let _ = tx.send(());

            // 等待一小段时间让端口释放
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            self.is_running = false;
            log::info!("流媒体服务器已停止");
        }
    }
}

/// 处理流媒体请求
async fn handle_stream(
    Path(station_id): Path<String>,
    State(state): State<Arc<ServerState>>,
) -> Response {
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

    // WebView 可能会对同一个 audio src 发起两次 GET。
    // 新请求到来时先关闭该电台已有流，确保同一电台最终只保留一个 FFmpeg。
    let replaced_existing_stream = state.stop_streams_for_station(&station_id).await;

    // 获取流地址：自定义电台直接用缓存地址，普通电台刷新
    let stream_url = if station.is_custom {
        log::debug!("custom station stream url");
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
                log::debug!("refreshed stream url");
                url
            }
            Ok(None) => {
                // 使用缓存的地址
                log::warn!("刷新流地址失败，使用缓存地址");
                match station.get_best_stream_url() {
                    Some(url) => url.to_string(),
                    None => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "无可用流地址").into_response();
                    }
                }
            }
            Err(e) => {
                log::error!("刷新流地址失败: {}", e);
                match station.get_best_stream_url() {
                    Some(url) => url.to_string(),
                    None => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "无可用流地址").into_response();
                    }
                }
            }
        }
    };

    log::debug!("stream url: {}...", &stream_url[..stream_url.len().min(80)]);

    // 启动 FFmpeg 进程
    let ffmpeg_path = &state.ffmpeg_path;

    let mut child = match spawn_ffmpeg(ffmpeg_path, &stream_url) {
        Ok(child) => child,
        Err(e) => {
            log::error!("启动 FFmpeg 失败: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("启动 FFmpeg 失败: {}", e),
            )
                .into_response();
        }
    };

    // 记录活动进程。使用请求级 ID，避免同一电台连续播放时互相覆盖。
    let request_id = next_stream_request_id(&station_id);
    if let Some(process_id) = child.id() {
        state.active_streams.write().await.insert(
            request_id.clone(),
            ActiveStream {
                station_id: station_id.clone(),
                process_id,
            },
        );
    }
    if !replaced_existing_stream {
        log::info!("正在播放: {} ({})", station.name, station.province);
    }

    // 获取输出流
    let stdout = child.stdout.take().expect("无法获取 stdout");

    // 创建流式响应
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Vec<u8>, std::io::Error>>(32);

    // 在后台读取 FFmpeg 输出
    let station_id_clone = station_id.clone();
    let request_id_clone = request_id.clone();
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
                    log::error!("读取 FFmpeg 输出失败: {}", e);
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
            .remove(&request_id_clone);
        log::debug!("stream closed: {} / {}", request_id_clone, station_id_clone);
    });

    // 构建响应
    let stream = ReceiverStream::new(rx);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .header(header::TRANSFER_ENCODING, "chunked")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header("icy-name", urlencoding::encode(&station.name).to_string())
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
    let list: Vec<_> = stations
        .values()
        .map(|s| {
            let mut s = s.clone();
            // 添加本地流地址
            s.mp3_play_url_high = Some(format!("http://127.0.0.1:{}/stream/{}", port, s.id));
            s
        })
        .collect();

    axum::Json(list)
}
