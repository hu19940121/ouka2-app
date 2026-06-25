# Type Safety

> 项目启用 TypeScript strict 模式，前端类型必须对齐 Tauri/Rust 的实际序列化契约。

## 编译约束

`tsconfig.json` 当前启用：

- `strict: true`
- `noUnusedLocals: true`
- `noUnusedParameters: true`
- `noFallthroughCasesInSwitch: true`
- `isolatedModules: true`
- `noEmit: true`

提交前最低检查是 `npm run build`，它会先运行 `vue-tsc --noEmit`。

## 类型组织

- 跨组件共享类型放在 `src/types/index.ts`。
- 组件私有辅助类型留在组件内。例：`InstallStationsDialog.vue` 的 `StationTransferOption extends TransferOption` 只服务该组件渲染。
- Naive UI 泛型要写清楚。例：`App.vue` 使用 `computed<DataTableColumns<Station>>()`，避免表格 render 回调里的 `station` 退化为隐式 any。
- DOM ref 使用明确元素类型。例：`AudioPlayer.vue` 的 `ref<HTMLAudioElement | null>`、`LogPanel.vue` 的 `ref<HTMLElement | null>`。

## IPC 字段对齐

前端接口必须跟 Rust serde 输出一致：

- `Station` 对齐 `src-tauri/src/radio/models.rs`，字段使用 snake_case：`play_url_low`、`mp3_play_url_low`、`mp3_play_url_high`、`is_custom`。
- `ServerStatus` 对齐 Rust 结构，字段使用 snake_case：`active_streams`、`total_stations`。
- `CrawlProgress` 对齐 Rust 结构，字段使用 snake_case：`stations_found`。
- `DiagnosticLogEntry` 对齐 `src-tauri/src/diagnostics.rs`，Rust 侧使用 `#[serde(rename_all = "camelCase")]`，所以前端使用 `stationId`、`stationName`。
- `InstallSelectionState` 对齐 `src-tauri/src/commands/config.rs` 的 `#[serde(rename_all = "camelCase")]`，所以前端使用 `stationIds`、`hasSavedSelection`。

新增 Rust 命令返回结构时，先检查是否有 `#[serde(rename_all = "camelCase")]`。不要统一改成 camelCase 或 snake_case。

## `invoke` 和 `listen` 类型

- 所有 `invoke()` 调用写泛型返回类型：

```ts
await invoke<DiagnosticLogEntry[]>('get_diagnostic_logs')
await invoke<InstallSelectionState>('load_install_selection')
await invoke<string[]>('get_ets2_paths')
```

- 事件监听也写 payload 类型：

```ts
await listen<CrawlProgress>('crawl-progress', (event) => {
  crawlProgress.value = event.payload
})
```

- Rust `Result<T, String>` 在前端 catch 到的是未知异常；当前项目使用 `String(e)` 转成用户可展示文本。不要假设错误一定有 `.message`。

## 可空和可选字段

- 可缺省字段按接口建模为 optional，例如 `Station.play_url_low?`、`Station.is_custom?`。使用时用布尔转换或兜底文案。
- 当前播放电台使用 `Station | null`，不要用空对象代表“没有播放”。
- DOM ref 一律包含 `null`，使用前判空。
- 日志字段 `stationId`、`stationName`、`detail` 是可选字段，渲染时要条件判断。

## 类型断言和 `any`

- 避免新增 `any`。现有例外：
  - `src/vite-env.d.ts` 的 Vue shim 使用 `DefineComponent<{}, {}, any>`，这是 Vite/Vue 模板声明惯例。
  - `AudioPlayer.vue` 的 `catch (e: any)` 可改进为 `unknown` + 类型收窄；新增代码不要复制这个写法。
- 类型断言只用于第三方库回调缺少项目字段时，并尽量限制范围。例：`InstallStationsDialog.vue` 的 `const stationOption = option as StationTransferOption`。
- 不要用非空断言 `!` 绕过 DOM 或 IPC 的空值检查。

## 版本和全局声明

- `__APP_VERSION__` 由 `vite.config.ts` 从 `package.json` 注入，声明在 `src/vite-env.d.ts`。使用时直接读常量，不要在前端重复读取 package 文件。
- 新增 Vite define 常量时，必须同步更新 `src/vite-env.d.ts`。
