# Janus

> 越过边界，开口即世界。

Janus 是太阳系软件系列中的 Windows 桌面端语言学习应用。初期聚焦英语，并通过 Language Pack 架构为后续扩展其他外语保留能力。

## 当前阶段

**M10：三档评分 + 熟词本与防扎堆复习已完成。**

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

M7 新增并验收：

- 桌面复习提醒：接入 Windows 桌面通知插件，按设置的小时/分钟在本地每日触发复习提醒（应用运行时生效），并提供「发送测试通知」即时验证；
- 语音设备选择：设置页可列出/检测系统麦克风，为朗读与 AI 对话指定输入设备（不选择时使用系统默认）；
- 通知状态与降级提示：展示通知授权状态，已保存设备不可用时明确提示，设备枚举失败静默降级；
- 回归：Rust 42 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

M8 新增并验收：

- 版本与更新：设置页新增「版本与更新」面板，展示应用版本与本地数据目录，可一键检查 GitHub Releases 最新版本（网络超时与失败均有中文降级提示），有新版时直接跳转下载页；
- 版本比较：Rust `update.rs` 纯函数解析/比较版本号（支持 v 前缀与预发布后缀），网络请求与判断逻辑分离，单元测试覆盖新版本/旧版本/等值版本与异常载荷；
- NSIS 安装包：`tauri.conf.json` 切换 bundle 目标为 `nsis`（当前用户安装、简体中文、隐藏语言选择器），`pnpm tauri build` 产出 Windows 安装程序与免安装产物；
- 回归：Rust 48 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test`、`cargo check --examples`、`pnpm tauri build` 通过。

M9 新增并验收：

- 词书体系：内置 6 本词书（小学 / 初中 / 高中 / 大学 / 雅思 / 专业），共 164 词条、159 个唯一种子词，`seed_words.rs` 幂等播种，M1 遗留单词自动映射进对应词书；
- 多词书选择：设置页可勾选任意词书组合（默认 高中 + 大学），学习范围随之生效，也可全局关闭范围限制；
- 选词模式：支持「随机选词」（从已勾选词书中按每日配额随机抽取新词）与「自定义选词」（在范围内逐词勾选想学的单词）；
- 每日计划：独立设置每日「新词数」与「复习数」（默认 10 / 20），每日队列先到期复习、后随机新词，按配额截断；
- 自适应记忆算法：以「词书难度 × 单词内在难度」计算有效难度，复习间隔 `effective_interval = round(base_interval / difficulty)`，难度越高复习越频繁；评分仍按 SM-2 风格四档（重来/模糊/认识/熟练）推进阶段；
- 学习范围统计：概览、复习队列、掌握度、今日进度均按已勾选词书范围统计；
- 学习配置持久化：词书勾选、选词模式、每日配额仅存本地（`app_settings`），重启保留；
- 回归：Rust 60 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

M10 新增并验收：

- 三档评分与熟词本：单词复习改为「重来 / 模糊 / 认识」三档（重来 10 分钟后当次重学，模糊隔几个单词再练，认识按计划推进），另设「熟悉」直接加入熟词本（记录掌握时间、`due_at` 置为永不，不再进入任何复习队列），生词本页新增「熟词」Tab，可一键移回复习队列；复习间隔封顶 100 天，达到封顶间隔后仍答「认识」会自动进入熟词本，不再无限循环复习；
- 记忆状态自适应重学：重来彻底重置节奏（reps 归零、lapses+1、ease-0.20）；模糊结合历史记忆状态——曾遗忘 / ease 偏低 / 难度高时立刻重学，否则按当前间隔 × 0.8 做短间隔复习，避免一刀切；
- 防扎堆复习（参考墨墨背单词）：到期复习不足配额时，从未来 7 天内将到期的复习中提前补足（削峰填谷）；到期复习超配额时今日新词自动减半（至少保留 20% 底线）；复习与新词交错排列、首卡必为复习，避免某天集体到期而当天没有新学；
- 间隔随机扰动：复习间隔大于 5 天时按 ±20% 加入确定性随机扰动（种子 = 单词 ID × 当天，同一张卡同一天稳定、跨天变化），首次「认识」按难度给较长基准（约 10 天 ~ 几十天，难度越低间隔越长）并加 ±10% 扰动，后续复习以当前间隔 × ease 推进，进一步打散复习落点、避免同日扎堆；
- 单词难度细化：内在难度由 3 档扩展为 9 档（0.6 / 0.8 / 0.9 / 1.0 / 1.1 / 1.2 / 1.25 / 1.4 / 1.5），92 个种子词按学段重标档位，简单词降档、学术/专业词升档；
- 数据迁移：SQLite 迁移 v7 新增 `mastered_at` 列，旧数据自动升级保留；
- 回归：Rust 70 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。
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
    lib/api.ts         # typed IPC 与统一错误模型（M2–M10）
    lib/recorder.ts    # 麦克风录音：16kHz mono PCM16 WAV，仅内存；设备枚举与选择（M4/M7）
    pages/             # Words/Sentences/Listening/Speaking/Conversation/Notebook/Analytics/Settings 学习页（M2–M10）
    styles/learning.css# 学习闭环样式（仅语义 Token，M2–M10）
  scripts/             # download-whisper-model.ps1 离线 ASR 模型下载（M4）
  src-tauri/           # Tauri/Rust 壳层、窗口命令与权限
    src/asr.rs         # 离线 whisper ASR 转写（M4）
    src/llm.rs         # OpenAI 兼容 AI 网关：超时/重试/中文错误映射（M5）
    src/reminder.rs    # 每日复习提醒调度：本地时间计算与通知触发（M7）
    src/update.rs       # 版本比较与 GitHub Releases 更新检查（M8）
    src/seed_words.rs   # 6 本内置词书种子数据：小学/初中/高中/大学/雅思/专业（M9）
    src/db.rs          # SQLite 迁移、SRS/自适应记忆、听力/听写/朗读/对话、事件、计划/提醒/语音设备、词书/学习配置/熟词本（M2–M10）
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

- 单词学习（6 本内置词书、随机/自定义选词、每日新词+复习配额、难度自适应 SRS、复习队列、评分、生词本）、句子（主动回忆、对/错事件）、听力（播放/变速/逐句循环/听写）、朗读（录音、离线 ASR、逐词反馈）与 AI 对话（场景对话、异步纠错、语音输入）已形成完整学习闭环；
- AI 对话依赖用户自备 OpenAI 兼容服务（Base URL / API Key / 模型，仅存本地数据库），TTS、流式链路与发音评分（音素/韵律）为 M11 规划；
- 听力音频为 Windows 离线 TTS 生成的种子语料，后续里程碑将支持用户导入真实音频；
- 不包含静态伪造评分或模拟 AI 数据；AI 密钥只存本地数据库，不进前端构建；
- 版本与更新：更新检查基于 GitHub Releases（`xlcblues/janus`），需要联网；无新版、网络失败或平台不支持均有明确中文降级提示；
- 发布产物：NSIS 安装程序已产出（当前用户安装、简体中文），尚未代码签名，干净机器安装/卸载/升级 E2E 与商店/官网发布留待 M11；
- 学习数据本地保存、可导出、可清除（清除前自动备份）。

## 下一步：M11

M10 聚焦实时链路与发布收尾：流式 ASR/LLM 与 TTS、语音打断、发音评分（音素/韵律）、真实音频导入与云同步接口，以及 Windows 代码签名、干净机器安装/升级/回滚专项验证与发布检查清单。

## 设计边界

- 与 Mars、Ceres、Neptune、Titan、Miranda、Europa 共享 Windows 桌面壳层与系列视觉语法；
- 不复制既有项目图形，只继承单色几何、节点、纸墨底色与克制层级；
- 所有产品颜色必须来自语义 Token；
- AI 语音、发音评分与纠错必须提供失败降级和可解释反馈。

## 验证状态

截至 2026-11-24，`pnpm typecheck`、`pnpm test`、`pnpm build`、`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo check --examples --manifest-path src-tauri/Cargo.toml` 与 `cargo test --manifest-path src-tauri/Cargo.toml` 均通过（Rust 70 项 + 前端 13 项）。

