//! FFmpeg 工具模块

use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

/// FFmpeg 管理器
pub struct FFmpegManager;

impl FFmpegManager {
    /// 检测系统中的 FFmpeg
    ///
    /// 按以下顺序查找：
    /// 1. 应用资源目录中的 FFmpeg (binaries/ffmpeg 或 binaries/ffmpeg.exe)
    /// 2. 系统 PATH 中的 FFmpeg
    pub fn detect_ffmpeg(app_resource_dir: Option<&PathBuf>) -> Option<PathBuf> {
        // 根据目标系统确定 FFmpeg 二进制文件名
        #[cfg(target_os = "windows")]
        let ffmpeg_binary = "ffmpeg.exe";
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_binary = "ffmpeg";

        // 1. 检查应用资源目录 (Tauri 会将 binaries 目录打包到 resources)
        if let Some(resource_dir) = app_resource_dir {
            // Tauri 2 资源路径结构
            let bundled_paths = [
                resource_dir.join("binaries").join(ffmpeg_binary),
                resource_dir.join(ffmpeg_binary),
            ];

            for bundled_ffmpeg in bundled_paths {
                if bundled_ffmpeg.exists() {
                    log::debug!("bundled ffmpeg: {:?}", bundled_ffmpeg);
                    return Some(bundled_ffmpeg);
                }
            }
        }

        // 2. 检查系统 PATH
        if Self::check_ffmpeg_in_path() {
            log::debug!("system ffmpeg from PATH");
            return Some(PathBuf::from("ffmpeg"));
        }

        log::error!("未找到 FFmpeg");
        None
    }

    /// 检查 FFmpeg 是否在系统 PATH 中
    fn check_ffmpeg_in_path() -> bool {
        #[cfg(target_os = "windows")]
        let result = Command::new("cmd")
            .args(["/C", "ffmpeg", "-version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        #[cfg(not(target_os = "windows"))]
        let result = Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        result
    }

    /// 获取 FFmpeg 版本信息
    pub fn get_version(ffmpeg_path: &PathBuf) -> Option<String> {
        Command::new(ffmpeg_path)
            .arg("-version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .and_then(|s| s.lines().next().map(|l| l.to_string()))
                } else {
                    None
                }
            })
    }
}

/// 检查 FFmpeg 是否可用
#[tauri::command]
pub fn check_ffmpeg(app_handle: tauri::AppHandle) -> Result<String, String> {
    // 获取资源目录，与 lib.rs 初始化时的逻辑保持一致
    let resource_dir = app_handle.path().resource_dir().ok();

    if let Some(path) = FFmpegManager::detect_ffmpeg(resource_dir.as_ref()) {
        if let Some(version) = FFmpegManager::get_version(&path) {
            Ok(version)
        } else {
            Err("FFmpeg 存在但无法获取版本信息".to_string())
        }
    } else {
        Err(
            "FFmpeg 未安装。请下载 FFmpeg 并放到应用目录的 binaries 文件夹中，或安装到系统 PATH"
                .to_string(),
        )
    }
}
