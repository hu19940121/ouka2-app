# Hook Guidelines

> 本项目是 Vue，不使用 React hook。这里的“hook”指 Vue 组合式 API、生命周期函数和未来可能抽取的 `use*` 组合式函数。

## 当前事实

- 仓库尚无 `src/composables/` 目录，也没有独立 `use*` 文件。
- 组合式逻辑主要内联在 `src/App.vue`、`src/stores/radio.ts` 和各组件中。
- 生命周期使用 `onMounted`、`onUnmounted`，监听使用 `watch`，派生状态使用 `computed`。

参考文件：

- `src/App.vue`：启动时依次初始化日志、FFmpeg、电台、自定义电台、安装列表，并设置服务器状态轮询。
- `src/components/AudioPlayer.vue`：`onMounted()` 自动播放、`onUnmounted()` 清理 audio。
- `src/components/InstallStationsDialog.vue`：用 `watch()` 在弹窗打开时同步本地选择状态。
- `src/components/LogPanel.vue`：监听日志长度变化后通过 `requestAnimationFrame()` 滚动到底部。
- `src/stores/radio.ts`：用 Pinia setup store 管理 Tauri event listener。

## 何时抽取 composable

当前优先内联，只有满足以下条件才新增 `src/composables/useXxx.ts`：

- 同一段组合式逻辑被 2 个以上组件复用。
- 逻辑包含明确的资源生命周期，例如 DOM 事件、计时器、Tauri `listen()`、浏览器 API 状态。
- 抽取后不会隐藏业务流程。比如安装队列的选择、持久化、去重仍应留在 store，而不是拆成组件私有 composable。

如果新增 `src/composables/`：

- 文件名使用 camelCase 的 `useXxx.ts`。
- 函数名与文件名一致，例如 `usePolling.ts` 导出 `usePolling()`。
- 返回值用对象，字段名保持业务语义，避免返回数组让调用方按位置解构。
- 需要清理的资源必须在 composable 内提供清理机制，或在 `onUnmounted()` 自动释放。

## 生命周期和清理

- 监听 Tauri 事件时，保存 `UnlistenFn` 并避免重复监听。`src/stores/radio.ts` 的 `unlistenDiagnosticLogs` 是现有模式。
- 临时监听应在 `finally` 中释放。`crawlStations()` 对 `crawl-progress` 的监听会在爬取完成或失败后 `unlisten()`。
- `setInterval()`、`setTimeout()`、DOM 事件、audio 资源都应有释放路径。现有 `App.vue` 的服务器轮询还没有清理，因为根组件生命周期等同应用生命周期；普通组件不要复制这种写法。
- `watch()` 用于同步外部输入到本地副本时，要限定触发条件。`InstallStationsDialog.vue` 只在 `visible` 为 true 时同步 `selectedStationIds`。

## 数据获取模式

- 前端没有 React Query / SWR 一类缓存库。所有桌面端数据获取通过 Pinia store 调 Tauri `invoke()`。
- 组件调用 store action，不直接调用 `invoke()`。这能把加载态、错误、持久化和 Rust 命令名集中在 `src/stores/radio.ts`。
- Tauri 事件流通过 `listen<T>()` 订阅，payload 类型必须来自 `src/types/index.ts`，例如 `listen<CrawlProgress>('crawl-progress', ...)`。

## 命名和错误处理

- 组合式函数、handler、computed 名称都用业务动作描述：`handleToggleAllFiltered`、`handleConfirmInstall`、`filteredLogs`、`selectedStations`。
- async handler 捕获错误时面向用户显示中文，内部日志可使用 `console.error`。不要把原始异常对象直接渲染到复杂 UI；当前 store 统一用 `String(e)` 存入 `error`。
- 浏览器 API 失败要有降级反馈，例如 `handleCopyStationUrl()` 和 `LogPanel.vue` 的复制逻辑。

## 不要这样做

- 不要新增 React 风格 hook 术语或目录结构。
- 不要为单个组件的一两个 `computed` 抽 composable。
- 不要在 composable 里隐式修改 Pinia store，除非函数名清楚表达副作用。
- 不要在多个组件重复实现 Tauri event listener；跨组件事件数据应进入 store。
