# Janus

> 越过边界，开口即世界。

Janus 是太阳系软件系列中的 Windows 桌面端语言学习应用。初期聚焦英语，并通过 Language Pack 架构为后续扩展其他外语保留能力。

## 当前阶段

**M6：数据、复习与计划模块已完成。**

M1 已实现并验收：极简 Logo「双面之门（Janus Gate）」、Janus Iris + Trial Brass 主题 Token、Tauri 2 + React 19 + TypeScript + Vite 8 Windows 桌面壳、自定义标题栏与窗口交互、模块导航（`Ctrl+1` 至 `Ctrl+6`）与 `Ctrl+K` 搜索、1360×820 / 1040×700 响应式验收。

M2 新增并验收：

- SQLite 本地数据库（`rusqlite` 0.31 bundled）：迁移、WAL、外键、事务与备份；
- 20 个种子单词 + 20 个种子句子，状态表自动初始化；
- 单词复习队列与 SM-2 风格 SRS 初版：重来/模糊/认识/熟练四档评分，阶段推进（new → learning → reviewing → mastered）；
- 句子学习：隐藏翻译主动回忆、对/错判断与学习事件；
- 生词本：加入 / 移除 / 列表；
- 学习数据：统计卡片、JSON 导出、清除前自动备份并重置；
- typed IPC：统一错误模型，所有命令返回用户可理解的中文错误；
- 浏览器预览降级：不伪造数据，明确提示数据仅在桌面应用可用；
- 回归：Rust 7 项单元/集成测试 + 前端 7 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo check` 通过；
- 重启后状态保留：文件级数据库重开测试验证单词状态与生词本持久化。

M3 新增并验收：

- 20 条真实英文语音片段（Windows 离线 TTS 生成，16kHz/16bit/mono，约 1.65MB）内嵌于安装包并在首次启动写入本地媒体缓存；
- 听力播放器：播放/暂停、进度定位、0.75× / 1.0× / 1.25× 变速、单句循环、字幕开关与双语字幕条；
- 逐句定位：20 条片段按句切分并记录 `start_ms` / `end_ms`，句段可点击跳转；
- 听写判断：后端（Rust/SQLite）规范化文本并判分，前端不自行评分；记录时长、正误与原文回显；
- 听写统计：累计听写、今日正确、总正确率与薄弱片段 Top5；
- 片段删除：删除音频文件与听写记录，并写入墓碑表防止重启后种子片段复活；
- asset 协议：通过 Tauri `assetProtocol` 安全提供本地媒体文件，不暴露绝对路径给前端组件；
- 回归：Rust 12 项单元/集成测试 + 前端 8 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

M4 新增并验收：

- 麦克风录音：`getUserMedia` → AudioContext 捕获，重采样至 16kHz mono PCM16 WAV，录音仅存内存、不落盘，页面卸载自动释放麦克风；
- 离线 ASR：whisper.cpp tiny.en 本地转写（whisper-rs 0.16），无 API 成本；`scripts/download-whisper-model.ps1` 下载约 77MB 模型至 `src-tauri/models/`（不随仓库分发），缺失时 UI 展示下载指引；
- 逐句跟读：复用听力片段库，参考音频逐句播放、录音、回放、重录与切换片段重置；
- 词级对齐反馈：后端 Rust DP 对齐 expected/transcribed，输出正确/漏读/替换/多读词级标记、准确率与置信度，前端不自行评分；
- 低置信度策略：ASR 不确定时明确提示“模型不确定”，不直接判错；
- 朗读统计：累计/今日朗读、平均准确率、低置信次数与薄弱句段；
- 数据记录：SQLite 迁移 v3 新增 `speaking_attempts` 表，仅存转写文本、指标与置信度，原始录音永不持久化；
- 回归：Rust 19 项单元/集成测试 + 前端 10 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test`、`cargo check --examples` 通过。

M5 新增并验收：

- 场景对话：5 个种子场景（咖啡馆点单 / 机场问路 / 酒店入住 / 求职面试 / 周末计划），SQLite 迁移 v4 新增会话四表，支持开始、继续历史会话、结束总结与统计；
- AI 网关：Rust `ureq` OpenAI 兼容 Chat Completions（连接 10s / 读写 45s 超时、单次重试、中文错误映射）；Base URL / API Key / 模型仅存本地数据库，不进入前端构建；
- 会话内纠错：独立 LLM 异步 pass 输出原句 / 更自然表达 / 中文说明并卡片展示，失败静默降级，不阻塞对话；
- 语音输入：复用 M4 离线 Whisper 转写，录音仅内存、转写进入输入框，可纯文字输入降级；
- 未配置降级：未填写服务配置时展示设置引导，测试连接可即时验证；
- 回归：Rust 31 项单元/集成测试 + 前端 11 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test`、`cargo check` 通过。

M6 新增并验收：

- 周趋势：最近 7 天按日聚合复习/听写/朗读/对话事件与学习时长（分钟），Dashboard 与「学习数据」页双图表；
- 掌握度：单词阶段分布（new → learning → reviewing → mastered）与整体掌握度环形图；
- 错题本：聚合「反复记错 ≥ 2 次」的单词、听写/朗读低分片段与对话表达纠错，仅收录可追溯证据；
- 今日进度：单词/句子/听力/朗读/对话完成量与连续学习天数，联动学习计划计算完成度；
- 学习计划：每日配额（单词/句子/听力/朗读/对话/时长）可在「设置」页调整，Dashboard 今日路线同步展示；
- 复习提醒：每日提醒时间偏好（启用/小时/分钟）仅存本地，桌面通知将在后续里程碑接入；
- 数据边界：所有设置与进度仅存本地 SQLite（迁移 v5 新增 app_settings 表），不向任何服务器上传；
- 回归：Rust 36 项单元/集成测试 + 前端 11 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

## 快速开始

环境要求：

- Windows 10/11；
- Node.js 20+；
- pnpm 11+；
- Rust stable；
- Microsoft Edge WebView2 Runtime。

```powershell
pnpm install
pnpm tauri dev
```

如在浏览器中单独调试前端：

```powershell
pnpm dev
```

## 常用命令

| 命令 | 用途 |
| --- | --- |
| `pnpm dev` | 启动 Vite 前端开发服务 |
| `pnpm tauri dev` | 启动 Tauri Windows 桌面应用 |
| `pnpm typecheck` | 运行 TypeScript 类型检查 |
| `pnpm test` | 运行 Vitest 测试 |
| `pnpm build` | 类型检查并生成前端生产构建 |
| `pnpm tauri build` | 生成 Windows 安装包与可执行产物 |
| `powershell -ExecutionPolicy Bypass -File scripts\download-whisper-model.ps1` | 下载离线 ASR 模型（whisper tiny.en，约 77MB）至 `src-tauri/models/` |

Windows 打包产物位于 `src-tauri/target/release/bundle/`。

## 文档入口

- [Logo 设计方案](design/logo-design.md)
- [UI 风格与主题规范](design/ui-design-system.md)
- [开发流程与技术方案](docs/开发流程与技术方案.md)
- [功能与技术方案](docs/功能与技术方案.md)
- [品牌资产说明](assets/brand/README.md)

## 视觉资产

- [品牌板](assets/brand/brand-board.png)
- [浅色 UI 预览](design/ui-style-preview-light.png)
- [深色 UI 预览](design/ui-style-preview-dark.png)
- [最小窗口预览](design/ui-style-preview-narrow.png)
- [主题 Token](design/tokens.css)
- [可交互 UI 预览](design/ui-style-preview.html)

## 目录结构

```text
Janus/
  assets/brand/        # Logo、字标、Windows 图标与品牌板
  design/              # 设计规范、Token 与高保真预览
  docs/                # 开发流程、功能与技术方案
  src/                 # React UI、组件、页面与样式
    lib/api.ts         # typed IPC 与统一错误模型（M2–M6）
    lib/recorder.ts    # 麦克风录音：16kHz mono PCM16 WAV，仅内存（M4）
    pages/             # Words/Sentences/Listening/Speaking/Conversation/Notebook/Analytics/Settings 学习页（M2–M6）
    styles/learning.css# 学习闭环样式（仅语义 Token，M2–M6）
  scripts/             # download-whisper-model.ps1 离线 ASR 模型下载（M4）
  src-tauri/           # Tauri/Rust 壳层、窗口命令与权限
    src/asr.rs         # 离线 whisper ASR 转写（M4）
    src/llm.rs         # OpenAI 兼容 AI 网关：超时/重试/中文错误映射（M5）
    src/db.rs          # SQLite 迁移、SRS、听力/听写/朗读/对话、事件、计划/提醒与备份（M2–M6）
  dist/                # Vite 生产构建，可重新生成
```

后续里程碑将按技术方案逐步增加 `domain/`、`features/`、`services/` 与 Rust 侧 `commands/` 等目录。

## 核心主题

| 名称 | 色值 | 用途 |
| --- | --- | --- |
| Janus Iris | `#6A55C2` | 主操作、选中、焦点 |
| Iris Light | `#9A8CF5` | 深色主题主操作 |
| Trial Brass | `#B48736` | 关键节点、成就、少量数据 |
| Night Ink | `#12111A` | 深色背景 |
| Warm Paper | `#F4F1EA` | 浅色背景 |

## 当前边界

- 单词（SRS 复习队列、评分、生词本）、句子（主动回忆、对/错事件）、听力（播放/变速/逐句循环/听写）、朗读（录音、离线 ASR、逐词反馈）与 AI 对话（场景对话、异步纠错、语音输入）已形成完整学习闭环；
- AI 对话依赖用户自备 OpenAI 兼容服务（Base URL / API Key / 模型，仅存本地数据库），TTS、流式链路与发音评分（音素/韵律）为 M7+ 规划；
- 听力音频为 Windows 离线 TTS 生成的种子语料，后续里程碑将支持用户导入真实音频；
- 不包含静态伪造评分或模拟 AI 数据；AI 密钥只存本地数据库，不进前端构建；
- 学习数据本地保存、可导出、可清除（清除前自动备份）。

## 下一步：M7

M7 聚焦 AI 对话的实时链路与提醒：流式 ASR/LLM 与 TTS、语音打断、发音评分（音素/韵律）、桌面复习通知、真实音频导入与云同步接口。

## 设计边界

- 与 Mars、Ceres、Neptune、Titan、Miranda、Europa 共享 Windows 桌面壳层与系列视觉语法；
- 不复制既有项目图形，只继承单色几何、节点、纸墨底色与克制层级；
- 所有产品颜色必须来自语义 Token；
- AI 语音、发音评分与纠错必须提供失败降级和可解释反馈。

## 验证状态

截至 2026-11-20，`pnpm typecheck`、`pnpm test`、`pnpm build`、`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo check --examples --manifest-path src-tauri/Cargo.toml` 与 `cargo test --manifest-path src-tauri/Cargo.toml` 均通过（Rust 31 项 + 前端 11 项）。