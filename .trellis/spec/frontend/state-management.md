# State Management

> 状态管理以 Pinia setup store 为核心，组件保留局部 UI 状态，Rust/Tauri 负责持久化和本地服务能力。

## Store 边界

`src/stores/radio.ts` 是当前唯一业务 store，覆盖：

- 电台数据：`stations`、`customStations`、`allStations`、`filteredStations`。
- 服务器状态：`serverStatus`、`getStreamUrl()`、`startServer()`、`stopServer()`、`refreshServerStatus()`。
- 爬取状态：`isCrawling`、`crawlProgress`、`crawlStations()`。
- 安装队列：`selectedStationIds`、`selectedStations`、`saveInstallSelection()`、`syncInstallSelection()`。
- 诊断日志：`diagnosticLogs`、`initDiagnosticLogs()`、`clearDiagnosticLogs()`。
- 自定义电台：`loadCustomStations()`、`addCustomStation()`、`removeCustomStation()`、`updateCustomStation()`。

新增业务如果仍属于这些概念，先扩展 `radio.ts`，不要急着创建第二个 store。只有出现新的长期业务域，并且状态不依赖电台 / 服务器 / 安装队列时，才考虑拆分。

## 状态分类

- 全局业务状态：放 Pinia。例：电台列表、安装队列、服务器运行状态、诊断日志。
- 派生状态：用 `computed`，不要手动同步多个 ref。例：`filteredStations`、`selectedStations`、`diagnosticErrorCount`。
- 组件局部状态：放组件 `ref`。例：`App.vue` 的弹窗开关、播放器折叠、toast；`AudioPlayer.vue` 的音量和播放进度；`InstallStationsDialog.vue` 的临时筛选。
- 持久化状态：通过 Tauri 命令落盘。例：安装队列保存到 Rust 侧 `install_selection.json`，自定义电台保存到 `custom_stations.json`。
- 浏览器本地设置：少量纯 UI 偏好可用 `localStorage`，当前只有 `ouka2-debug-mode`。

## Tauri IPC 模式

- `invoke<T>()` 必须写返回类型，返回类型来自 `src/types/index.ts` 或明确 primitive：

```ts
stations.value = await invoke<Station[]>('load_saved_stations')
serverStatus.value = await invoke<ServerStatus>('get_server_status')
return await invoke<string>('install_sii_to_ets2_with_selection', { stationIds })
```

- Rust 命令名以 `src-tauri/src/lib.rs` 的 `tauri::generate_handler!` 为准。不要凭函数名猜测是否已注册。
- 命令参数传给 Tauri 时使用 Rust 命令参数的 camelCase 形式。例：Rust `station_ids: Vec<String>` 对应前端 `{ stationIds: sanitizedIds }`。
- 事件流使用 `listen<T>()`，payload 类型对齐 Rust emit。现有事件：
  - `crawl-progress`，payload `CrawlProgress`，由 `src-tauri/src/commands/crawler.rs` 发出。
  - `diagnostic-log`，payload `DiagnosticLogEntry`，由诊断日志系统发出。

## 安装队列规则

- 所有进入安装队列的 id 必须经过 `sanitizeStationIds()`，保证只保留当前存在电台并去重。
- 自定义电台新增后自动加入安装队列；删除自定义电台后必须同步清理安装队列。
- `loadInstallSelection()` 需要处理首次使用：没有保存文件时保存空列表，并设置 `hasSavedInstallSelection`。
- 批量选择要保留顺序，参考 `saveInstallSelection()` 和 Rust 侧 `filter_stations_by_ids()`。

## 错误和加载态

- `isLoading` 只用于加载已保存电台；`isCrawling` 只用于爬取流程；安装过程的 `isInstalling` 当前是 `App.vue` 局部状态。
- store 中可恢复错误写入 `error: string | null`，UI 再通过 toast 或状态提示展示。
- `refreshServerStatus()` 失败时将 `serverStatus.running` 置为 false，避免 UI 继续显示服务器在线。
- 对日志这类持续增长数据要限制长度。`diagnosticLogs` 当前保留最近 1000 条。

## 不要这样做

- 不要在组件中绕过 store 调用 `invoke()`，否则状态、错误和持久化会分散。
- 不要在多个地方分别实现电台去重或安装队列清洗。复用 `sanitizeStationIds()` 所在流程。
- 不要把 `Station` 对象长期存作安装选择状态；持久化和跨层传递使用 station id。
- 不要用 watch 手动维护可以由 `computed` 表达的派生状态。
