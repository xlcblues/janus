# Janus

> 越过边界，开口即世界。

Janus 是太阳系软件系列中的 Windows 桌面端语言学习应用。初期聚焦英语，并通过 Language Pack 架构为后续扩展其他外语保留能力。

## 当前阶段

**M1：应用壳与桌面工程基线已完成。**

已实现并验收：

- 极简 Logo「双面之门（Janus Gate）」及浅色、深色、单色、横版字标与 Windows 应用图标；
- Janus Iris + Trial Brass 主题 Token、浅/深色主题及主题持久化；
- Tauri 2 + React 19 + TypeScript + Vite 8 Windows 桌面壳；
- 自定义标题栏、原生拖拽与双击最大化、最小化、最大化/还原、关闭；
- 今日学习、单词、句子、听力、朗读、AI 对话、生词本、学习数据与设置模块入口；
- `Ctrl+1` 至 `Ctrl+6` 模块导航、`Ctrl+K` 搜索与快速跳转；
- 1360×820 常规布局与 1040×700 最小窗口响应式验收；
- Logo 设计规范、UI 设计系统、开发流程与技术方案、功能与技术方案。

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
  src-tauri/           # Tauri/Rust 壳层、窗口命令与权限
  dist/                # Vite 生产构建，可重新生成
```

M2 起将按技术方案逐步增加 `domain/`、`features/`、`services/` 与 Rust 侧 `db/` 等目录。

## 核心主题

| 名称 | 色值 | 用途 |
| --- | --- | --- |
| Janus Iris | `#6A55C2` | 主操作、选中、焦点 |
| Iris Light | `#9A8CF5` | 深色主题主操作 |
| Trial Brass | `#B48736` | 关键节点、成就、少量数据 |
| Night Ink | `#12111A` | 深色背景 |
| Warm Paper | `#F4F1EA` | 浅色背景 |

## 当前边界

- 当前版本只实现桌面壳、导航、主题、搜索与响应式布局；
- 单词/句子/听力/朗读尚未接入真实课程数据与交互闭环；
- AI 语音、麦克风录音、ASR、TTS、发音评分与云端服务尚未接入；
- 不包含静态伪造评分、模拟 AI 对话或任何密钥配置；
- 所有学习数据能力将在 M2 起按本地优先原则实现。

## 下一步：M2

M2 聚焦最小本地学习闭环：

1. SQLite 初始化、迁移与备份；
2. typed IPC 与统一错误模型；
3. 单词复习状态、句子学习事件与生词本；
4. 最小可解释反馈与学习数据统计；
5. 重启后状态保留与端到端回归。

## 设计边界

- 与 Mars、Ceres、Neptune、Titan、Miranda、Europa 共享 Windows 桌面壳层与系列视觉语法；
- 不复制既有项目图形，只继承单色几何、节点、纸墨底色与克制层级；
- 所有产品颜色必须来自语义 Token；
- AI 语音、发音评分与纠错必须提供失败降级和可解释反馈。

## 验证状态

截至 2026-11-20，`pnpm typecheck`、`pnpm test`、`pnpm build` 与 `cargo check --manifest-path src-tauri/Cargo.toml` 均通过。