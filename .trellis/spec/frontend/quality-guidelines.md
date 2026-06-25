# Quality Guidelines

> 质量标准以当前工具链可执行检查为准，同时关注桌面应用的跨层行为、布局稳定性和用户可恢复错误。

## 必跑检查

当前 `package.json` 没有 lint 或 test 脚本。提交前至少运行：

```bash
npm run build
```

该命令执行 `vue-tsc --noEmit && vite build`，能捕获 TypeScript、Vue SFC 和 Vite 构建问题。

如果改动 Tauri / Rust 后端，还应运行对应 Rust 检查；本 frontend spec 不规定 Rust 命令，但前端调用的命令名和返回类型必须与 Rust 源码核对。

## 评审重点

- IPC 契约：新增或修改 `invoke()` 时，核对 `src-tauri/src/lib.rs` 是否注册命令、`src-tauri/src/commands/*.rs` 参数名和 serde 字段名是否匹配。
- 状态一致性：安装队列必须去重并剔除不存在电台；自定义电台增删后要同步队列和服务器状态。
- 生命周期：Tauri `listen()`、DOM 事件、audio、定时器必须有释放路径，避免重复订阅或资源泄漏。
- 加载和错误：长操作要有 loading 状态，失败要能通过中文提示或日志定位。
- 布局稳定：表格、队列、日志、播放器等固定区域要处理长文本、窄屏和滚动，不允许按钮或文字互相覆盖。

## 可访问性和桌面体验

- 裸按钮必须写 `type="button"`。
- 图标按钮必须有 `title` 或 tooltip。
- 输入框、筛选器和主操作按钮应使用 Naive UI 组件，保持键盘和焦点行为。
- 远程电台图片要有 `alt`，无图片时用文字兜底，参考 `AudioPlayer.vue`。
- 长耗时任务如爬取电台应保留进度反馈，参考 `CrawlProgress.vue`。

## 测试现状

- 仓库当前没有单元测试、组件测试或端到端测试配置。
- 不要在 PRD 或实现计划中写“运行测试套件”而不添加对应脚本。
- 对关键交互的回归检查目前以手动场景和构建检查为主：刷新数据、启动服务器、播放/停止、管理安装队列、添加/删除自定义电台、打开调试日志。

## 禁止模式

- 在组件里直接调用 Tauri `invoke()`，绕过 `src/stores/radio.ts`。
- 新增未类型化的 IPC payload 或 `invoke()` 返回值。
- 新增大面积不符合当前工具型 UI 的视觉风格，例如营销 hero、强渐变背景、emoji 按钮、过度圆角卡片。
- 用 `any`、非空断言或宽泛类型掩盖跨层字段不匹配。
- 静默吞掉用户可恢复错误。至少更新 `store.error`、toast 或诊断日志。
- 复制旧组件里的遗留视觉模式作为新主流程 UI，尤其是 `StatusBar.vue` 中的 emoji 和渐变按钮。

## 提交前清单

- [ ] `npm run build` 通过，或明确记录无法运行的原因。
- [ ] 新增可见文案为中文，且与桌面工具语气一致。
- [ ] 新增 props / emits / store action 都有明确类型。
- [ ] 新增 Tauri 命令调用已核对 Rust 注册、参数和返回字段。
- [ ] 长列表、长文本、窄屏布局不会撑破容器。
- [ ] 没有留下模板残文、调试输出或无用依赖。
