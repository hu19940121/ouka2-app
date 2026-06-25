# Directory Structure

> 前端代码按 Tauri 桌面应用的真实边界组织：入口、共享类型、单一业务 store、界面组件和静态资源。

## 当前目录

```text
src/
├── main.ts                 # Vue createApp 入口，只注册 Pinia 并挂载 App
├── App.vue                 # 主应用壳、全局主题覆盖、主要页面编排
├── stores/
│   └── radio.ts            # 电台、服务器、安装队列、日志和 Tauri IPC
├── types/
│   └── index.ts            # 前端共享接口，对齐 Rust 序列化结果
├── components/
│   ├── AudioPlayer.vue
│   ├── CrawlProgress.vue
│   ├── InstallQueuePanel.vue
│   ├── InstallStationsDialog.vue
│   ├── LogPanel.vue
│   ├── StationCard.vue
│   └── StatusBar.vue
└── assets/
    ├── app-logo-ui.png
    ├── app-logo.png
    └── vue.svg
```

仓库其他相关目录：

- `src-tauri/`：Tauri 2 后端。前端新增 IPC 调用前必须先确认 `src-tauri/src/lib.rs` 的 `tauri::generate_handler!` 和对应 `src-tauri/src/commands/*.rs`。
- `scripts/`：构建辅助脚本，例如版本同步和 FFmpeg 下载。
- `docImg/`、`design/`：README 与设计说明图片，不属于运行时代码。

## 放置规则

- 入口保持轻量：`src/main.ts` 只做 `createApp`、`createPinia`、`mount` 这类全局启动工作。不要把业务初始化塞进入口。
- 业务状态和 Tauri 命令优先放在 `src/stores/radio.ts`。当前项目只有一个业务域 store，新功能如果仍围绕电台、安装、服务器或日志，继续扩展该 store。
- UI 组件放在 `src/components/`，文件名使用 PascalCase，例如 `InstallStationsDialog.vue`。组件内部用 `<script setup lang="ts">`。
- 跨组件共享接口放在 `src/types/index.ts`。仅组件私有的渲染辅助类型可留在组件内，例如 `InstallStationsDialog.vue` 的 `StationTransferOption`。
- 图片等前端打包资源放在 `src/assets/`，公共静态文件才放 `public/`。

## 功能边界

- `src/App.vue` 当前承担页面壳、主表格、顶栏、弹窗开关和播放状态编排。新增大型区域时，优先抽到 `src/components/`，让 `App.vue` 只保留状态接线和页面布局。
- `src/components/AudioPlayer.vue` 管理 HTML audio 元素生命周期，播放相关 DOM 状态不要提升到 Pinia，除非多个非播放器组件需要共享。
- `src/components/InstallQueuePanel.vue` 和 `src/components/InstallStationsDialog.vue` 是安装队列的两个视图，真实选择状态仍由 `src/stores/radio.ts` 保存。
- `src/components/LogPanel.vue` 只负责展示、过滤、复制日志；日志加载、清空、实时订阅属于 store。

## 命名约定

- Vue 组件文件：PascalCase。
- store：`useRadioStore`，定义在 `src/stores/radio.ts`。
- 事件名：模板中使用 kebab-case，例如 `@toggle-collapse`、`@clear`；`defineEmits` 中保留同名字符串。
- Tauri 命令名：前端 `invoke()` 使用 Rust 注册命令的 snake_case 字符串，例如 `load_saved_stations`、`install_sii_to_ets2_with_selection`。
- 前端字段名必须跟实际序列化结果一致。`Station`、`ServerStatus`、`CrawlProgress` 当前使用 snake_case 字段；`InstallSelectionState` 因 Rust `#[serde(rename_all = "camelCase")]` 使用 `stationIds`、`hasSavedSelection`。

## 不要这样做

- 不要新增 `pages/`、`features/`、`services/` 等目录，除非先有多个真实模块需要拆分；当前项目规模还没形成这些边界。
- 不要在组件里散落 Tauri `invoke()` 调用。现有模式是组件调用 store action，store 负责 IPC、状态更新和错误归一。
- 不要把 `src-tauri/` 类型直接复制成另一套命名风格。先看 Rust serde 配置，再在 `src/types/index.ts` 对齐实际 JSON 字段。
