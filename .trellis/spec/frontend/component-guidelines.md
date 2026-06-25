# Component Guidelines

> 组件以 Vue 3 `<script setup lang="ts">`、显式 props / emits、局部 scoped CSS 为主；Naive UI 负责复杂控件，lucide-vue-next 负责图标。

## 基本结构

新组件遵循现有顺序：

1. `<script setup lang="ts">`
2. imports
3. `defineProps` / `defineEmits`
4. 局部 `ref`、`computed`、watch 或生命周期
5. 事件处理函数
6. `<template>`
7. `<style scoped>`

参考：

- `src/components/AudioPlayer.vue`：播放器 DOM ref、计算状态、生命周期清理和事件上抛。
- `src/components/InstallQueuePanel.vue`：纯展示面板，通过 props 接收状态、通过 emits 交给父级处理。
- `src/components/InstallStationsDialog.vue`：组件私有筛选状态、Naive UI `NTransfer` 自定义渲染。
- `src/components/LogPanel.vue`：本地过滤、复制日志、滚动到底部。

## Props 和 Emits

- props 使用类型字面量直接声明，保持字段含义清楚：

```ts
const props = defineProps<{
  status: ServerStatus
  selectedStations: Station[]
  isInstalling: boolean
}>()
```

- emits 使用具名 tuple，并把 payload 类型写出来：

```ts
const emit = defineEmits<{
  confirm: [stationIds: string[]]
  remove: [stationId: string]
  close: []
}>()
```

- 组件不要直接修改 props。需要可编辑副本时，像 `InstallStationsDialog.vue` 一样使用 `localSelectedIds`，在确认时 `emit('confirm', [...localSelectedIds.value])`。
- 事件名称保持业务语义，不要传 DOM 事件给父级让父级猜。例如使用 `install`、`manage`、`clear`，而不是 `clickButton`。

## 状态和组合式 API

- 只在组件内使用真正局部的 UI 状态，例如弹窗内搜索条件、音量、复制反馈、折叠状态。
- 派生值使用 `computed`，不要在多个 handler 中重复维护。例如 `AudioPlayer.vue` 的 `canNavigate`、`progressValue`，`LogPanel.vue` 的 `filteredLogs`。
- 涉及浏览器或 DOM 资源时要清理。`AudioPlayer.vue` 在 `onUnmounted()` 中重置 audio 元素；新增定时器、事件监听也应有对应释放路径。

## UI 组件和图标

- 优先使用 Naive UI 承担输入、选择、表格、标签、按钮、滚动条、空状态和 Transfer，例如 `NDataTable`、`NSelect`、`NTransfer`、`NEmpty`。
- 图标使用 `lucide-vue-next`。按钮如果只是动作入口，应配图标和 `title` 或 Naive UI `NTooltip`，参考 `App.vue` 表格操作列和 `LogPanel.vue` 顶部按钮。
- 主界面中文文案已本地化，新增可见文案继续使用中文，且保持桌面工具风格：短、具体、面向操作。

## 样式模式

- 组件样式默认 `<style scoped>`；`App.vue` 当前包含全局壳层样式和 Naive UI 主题覆盖。
- 视觉基调是浅色、紧凑、偏工具型：绿色主色 `#2f9e55`、中性边框 `#e3e6eb` / `#e4e8ef`、圆角多为 `6px` 或 `8px`。不要引入大面积渐变、营销式 hero 或装饰性背景。
- 固定格式区域用稳定尺寸和溢出处理：表格列、播放器、安装队列、日志面板都要避免文字撑破布局。参考 `App.vue` 的 `NDataTable :scroll-x="620"`、`LogPanel.vue` 的 `width: min(620px, 92vw)`。
- 对长文本使用 `min-width: 0`、`overflow: hidden`、`text-overflow: ellipsis`、`white-space: nowrap` 或 `overflow-wrap: anywhere`，已有示例见 `StationCard.vue` 和 `App.vue`。

## 可访问性和交互

- 所有裸 `button` 必须写 `type="button"`，避免未来放入表单时触发提交。现有 `App.vue`、`StationCard.vue`、`AudioPlayer.vue` 已采用该模式。
- 图标按钮需要 `title` 或 tooltip，尤其是播放、复制、删除、关闭等无文本按钮。
- 弹窗遮罩可支持 `@click.self` 关闭，参考 `InstallStationsDialog.vue`。
- 列表渲染必须有稳定 key，优先使用业务 id；日志这种没有唯一 id 的记录才组合 `time-index-message`。

## 常见风险

- `StatusBar.vue` 和 `StationCard.vue` 保留了旧版视觉和 emoji / 渐变按钮痕迹；新增主流程 UI 时优先参考 `App.vue`、`InstallQueuePanel.vue`、`InstallStationsDialog.vue` 的当前风格。
- 不要在组件内直接读写安装队列持久化。选择、去重、保存由 `src/stores/radio.ts` 的 `saveInstallSelection`、`syncInstallSelection`、`setSelectedStationIds` 负责。
- 不要为了一个组件私有状态新增全局 store 字段。只有跨组件共享或需要 Tauri 持久化的状态才提升。
