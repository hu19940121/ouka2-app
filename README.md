# 欧卡2中国电台 - 桌面应用

将中国云听电台转换为欧卡2可用的 MP3 流格式。

## 功能特性

- 🎵 自动爬取 900+ 中国电台
- 🔄 实时将 m3u8 流转换为 MP3 格式
- 📥 一键生成 ETS2 配置文件
- 🖥️ 内置音频播放器预览
- 🔍 按地区搜索和筛选

## 开发

### 前置要求

- Node.js 18+
- Rust 1.70+
- FFmpeg (开发时需要系统安装)

### 安装依赖

```bash
npm install
```

### 运行开发版本

```bash
npm run tauri dev
```

## 打包

### 一键打包（自动下载 FFmpeg）

```bash
npm run tauri:build
```

此命令会：
1. 自动检测 FFmpeg 是否存在
2. 如果不存在，自动下载并解压 FFmpeg
3. 构建前端和 Rust 后端
4. 生成安装包

### 打包输出

打包完成后，安装文件位于：
- **MSI 安装包**: `src-tauri/target/release/bundle/msi/`
- **NSIS 安装包**: `src-tauri/target/release/bundle/nsis/`

## 使用说明

1. **启动应用** - 双击运行
2. **获取电台** - 点击"刷新数据"获取最新电台列表
3. **启动服务器** - 点击"启动服务器"，会在本地 3000 端口启动流媒体服务
4. **试听电台** - 点击电台卡片上的播放按钮
5. **安装到欧卡2** - 点击"安装到欧卡2"按钮，自动生成配置文件

## 技术栈

- **前端**: Vue 3 + TypeScript + Pinia
- **后端**: Rust + Tauri 2
- **流媒体**: axum HTTP 服务器 + FFmpeg 转码

## 目录结构

```
ouka2-app/
├── src/                    # Vue 3 前端
│   ├── components/         # Vue 组件
│   ├── stores/             # Pinia 状态管理
│   └── types/              # TypeScript 类型
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── radio/          # 电台核心模块
│   │   ├── commands/       # Tauri 命令
│   │   └── utils/          # 工具函数
│   └── binaries/           # 内嵌的 FFmpeg
└── scripts/                # 构建脚本
    └── download-ffmpeg.js  # FFmpeg 自动下载
```

## 许可证

MIT
