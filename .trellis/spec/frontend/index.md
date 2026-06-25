# Frontend Development Guidelines

> 本目录记录 `ouka2-app` 前端层的真实编码约定，供后续 Trellis 任务和 AI 代理读取。

## 项目上下文

这是一个 Vue 3 + TypeScript + Pinia + Naive UI 的 Tauri 2 桌面应用。前端不是独立 Web 服务，而是通过 `@tauri-apps/api` 调用 `src-tauri/src/commands/` 中的 Rust 命令，并围绕“电台列表、安装队列、本地流服务器、播放器、诊断日志”组织界面。

主要参考文件：

- `src/main.ts`：Vue 应用和 Pinia 注册入口。
- `src/App.vue`：当前主界面容器，包含全局主题覆盖、表格、弹窗和主要业务交互。
- `src/stores/radio.ts`：前端业务状态与 Tauri IPC 调用中枢。
- `src/types/index.ts`：前端共享数据类型。
- `src/components/*.vue`：可复用界面组件。
- `src-tauri/src/commands/*.rs`、`src-tauri/src/radio/models.rs`：前后端 IPC 契约来源。

## Guidelines Index

| Guide | 何时阅读 |
|-------|----------|
| [Directory Structure](./directory-structure.md) | 新增或移动前端文件、判断功能放在哪里 |
| [Component Guidelines](./component-guidelines.md) | 新增 Vue 组件、调整 props / emits / 样式 |
| [Hook Guidelines](./hook-guidelines.md) | 抽取组合式逻辑、处理生命周期和浏览器 API |
| [State Management](./state-management.md) | 修改 Pinia store、Tauri IPC、安装队列或服务器状态 |
| [Type Safety](./type-safety.md) | 新增共享类型、对齐 Rust 命令返回值、处理 `any` / 断言 |
| [Quality Guidelines](./quality-guidelines.md) | 提交前检查、评审、可访问性和回归风险 |

## Bootstrap 结论

- 当前 Trellis 初始化为单仓库 `frontend` 层，因此规范集中在 `.trellis/spec/frontend/`。
- 仓库实际还包含 `src-tauri/` Rust 后端；前端规范必须引用 IPC 契约，但不在本目录展开 Rust 后端编码规范。
- 项目没有 ESLint、Vitest 或 Playwright 配置；不要在任务中假设存在测试脚本。最低可靠检查是 `npm run build`，它会执行 `vue-tsc --noEmit && vite build`。
- 规范语言使用中文，匹配当前 `AGENTS.md` 要求和源码注释习惯。
