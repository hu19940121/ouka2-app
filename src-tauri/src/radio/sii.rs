//! SII 配置文件生成器
//!
//! 生成欧卡2可用的 live_streams.sii 配置文件

use std::path::{Path, PathBuf};
use crate::radio::models::Station;

/// SII 文件生成器
pub struct SiiGenerator {
    server_host: String,
    server_port: u16,
}

impl SiiGenerator {
    /// 创建新的生成器
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            server_host: host.to_string(),
            server_port: port,
        }
    }

    /// 生成 SII 文件内容
    pub fn generate(&self, stations: &[Station]) -> String {
        let mut content = format!(
            r#"SiiNunit
{{
# 欧卡2中国电台配置文件
# 由 ouka2-desktop 自动生成
# 生成时间: {}
#
# 使用说明:
# 1. 确保本地转发服务器正在运行
# 2. 将此文件复制到:
#    %USERPROFILE%\Documents\Euro Truck Simulator 2\live_streams.sii
# 3. 重启游戏即可在电台列表中看到中国电台

live_stream_def : .live_streams {{
 stream_data: {}
"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            stations.len()
        );

        // 添加每个电台
        for (index, station) in stations.iter().enumerate() {
            let stream_url = format!(
                "http://{}:{}/stream/{}",
                self.server_host, self.server_port, station.id
            );
            let name = self.to_english_name(&station.name);
            let genre = self.get_genre(station);

            // SII格式: stream_data[index]: "URL|Name|Genre|Language|Bitrate|Favorite"
            content.push_str(&format!(
                " stream_data[{}]: \"{}|{}|{}|CN|128|0\"\n",
                index, stream_url, name, genre
            ));
        }

        content.push_str("}\n}\n");
        content
    }

    /// 保存到文件
    pub fn save_to_file(&self, content: &str, path: &Path) -> anyhow::Result<()> {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        log::info!("✅ 配置文件已生成: {:?}", path);
        Ok(())
    }

    /// 自动安装到欧卡2目录
    pub fn install_to_ets2(&self, content: &str) -> anyhow::Result<PathBuf> {
        let ets2_paths = Self::detect_ets2_paths();

        if ets2_paths.is_empty() {
            anyhow::bail!("未找到欧卡2文档目录");
        }

        // 使用第一个找到的路径
        let target_path = ets2_paths[0].join("live_streams.sii");
        self.save_to_file(content, &target_path)?;

        Ok(target_path)
    }

    /// 检测欧卡2文档目录
    pub fn detect_ets2_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 标准文档目录
        if let Some(docs_dir) = dirs::document_dir() {
            let ets2_dir = docs_dir.join("Euro Truck Simulator 2");
            if ets2_dir.exists() {
                paths.push(ets2_dir);
            }
        }

        // 也检查 OneDrive 文档目录
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let onedrive_ets2 = PathBuf::from(&user_profile)
                .join("OneDrive")
                .join("Documents")
                .join("Euro Truck Simulator 2");
            if onedrive_ets2.exists() && !paths.contains(&onedrive_ets2) {
                paths.push(onedrive_ets2);
            }
        }

        paths
    }

    /// 将中文电台名称转换为英文（欧卡2只支持ASCII字符）
    fn to_english_name(&self, chinese_name: &str) -> String {
        // 常见电台名称映射
        let name_map = [
            ("中国之声", "China Voice"),
            ("经济之声", "Economy Voice"),
            ("音乐之声", "Music Voice"),
            ("都市之声", "City Voice"),
            ("中华之声", "Zhonghua Voice"),
            ("神州之声", "Shenzhou Voice"),
            ("华夏之声", "Huaxia Voice"),
            ("香港之声", "Hong Kong Voice"),
            ("民族之声", "Minzu Voice"),
            ("文艺之声", "Arts Voice"),
            ("老年之声", "Seniors Voice"),
            ("娱乐广播", "Entertainment Radio"),
            ("高速广播", "Highway Radio"),
            ("交通广播", "Traffic Radio"),
            ("新闻广播", "News Radio"),
            ("音乐广播", "Music Radio"),
            ("经济广播", "Economy Radio"),
            ("生活广播", "Life Radio"),
            ("文艺广播", "Arts Radio"),
            ("旅游广播", "Travel Radio"),
            ("农村广播", "Rural Radio"),
            ("体育广播", "Sports Radio"),
            ("私家车广播", "Car Radio"),
            ("故事广播", "Story Radio"),
        ];

        // 尝试匹配已知名称
        for (cn, en) in name_map.iter() {
            if chinese_name.contains(cn) {
                // 提取省份/城市前缀
                let prefix = chinese_name.replace(cn, "").trim().to_string();
                if !prefix.is_empty() {
                    // 清理前缀中的多余字符
                    let clean_prefix = prefix
                        .replace("广播电台", "")
                        .replace("电台", "")
                        .replace("人民广播", "")
                        .trim()
                        .to_string();
                    if !clean_prefix.is_empty() {
                        return format!("{} {}", clean_prefix, en);
                    }
                }
                return en.to_string();
            }
        }

        // 如果没有匹配，尝试基本清理并返回
        let cleaned = chinese_name
            .replace("广播电台", "")
            .replace("电台", "")
            .replace("人民广播", "")
            .replace("频率", "")
            .replace("频道", "")
            .trim()
            .to_string();

        if cleaned.is_empty() {
            "Radio CN".to_string()
        } else {
            // 检查是否全是ASCII字符
            if cleaned.is_ascii() {
                cleaned
            } else {
                // 包含中文，返回通用名称加序号
                format!("CN Radio {}", chinese_name.len() % 100)
            }
        }
    }

    /// 获取电台流派
    fn get_genre(&self, station: &Station) -> &'static str {
        let name = station.name.to_lowercase();

        if name.contains("新闻") || name.contains("之声") {
            "news"
        } else if name.contains("音乐") || name.contains("music") {
            "music"
        } else if name.contains("交通") || name.contains("高速") {
            "traffic"
        } else if name.contains("经济") || name.contains("财经") {
            "economy"
        } else if name.contains("文艺") || name.contains("故事") {
            "culture"
        } else if name.contains("体育") {
            "sports"
        } else if name.contains("娱乐") || name.contains("都市") {
            "entertainment"
        } else {
            "general"
        }
    }
}

impl Default for SiiGenerator {
    fn default() -> Self {
        Self::new("127.0.0.1", 3000)
    }
}
