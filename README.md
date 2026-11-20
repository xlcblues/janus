# Janus

> 越过边界，开口即世界。

Janus 是太阳系软件系列中的 Windows 桌面端语言学习应用。初期聚焦英语，并通过 Language Pack 架构为后续扩展其他外语保留能力。

## 当前阶段

**M3：听力与媒体模块已完成。**

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
    lib/api.ts         # typed IPC 与统一错误模型（M2/M3）
    pages/             # Words/Sentences/Listening/Notebook/Analytics 学习页（M2/M3）
    styles/learning.css# 学习闭环样式（仅语义 Token）
  src-tauri/           # Tauri/Rust 壳层、窗口命令与权限
    src/db.rs          # SQLite 迁移、SRS、听力/听写、事件、导出与备份（M2/M3）
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

- 单词（SRS 复习队列、评分、生词本）、句子（主动回忆、对/错事件）与听力（播放/变速/逐句循环/听写）已形成本地学习闭环；
- 朗读与 AI 对话尚未接入：无录音、ASR、TTS、发音评分与云端服务；
- 听力音频为 Windows 离线 TTS 生成的种子语料，后续里程碑将支持用户导入真实音频；
- 不包含静态伪造评分、模拟 AI 对话或任何密钥配置；
- 学习数据本地保存、可导出、可清除（清除前自动备份）。

## 下一步：M4

M4 聚焦朗读与口语：麦克风权限流程、VAD、录音回放、流式/分段 ASR 与基础发音反馈。

## 设计边界

- 与 Mars、Ceres、Neptune、Titan、Miranda、Europa 共享 Windows 桌面壳层与系列视觉语法；
- 不复制既有项目图形，只继承单色几何、节点、纸墨底色与克制层级；
- 所有产品颜色必须来自语义 Token；
- AI 语音、发音评分与纠错必须提供失败降级和可解释反馈。

## 验证状态

截至 2026-11-20，`pnpm typecheck`、`pnpm test`、`pnpm build`、`cargo check --manifest-path src-tauri/Cargo.toml` 与 `cargo test --manifest-path src-tauri/Cargo.toml` 均通过（Rust 12 项 + 前端 8 项）。