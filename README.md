# Janus

> 越过边界，开口即世界。

Janus 是太阳系软件系列中的 Windows 桌面端语言学习应用。初期聚焦英语，并通过 Language Pack 架构为后续扩展其他外语保留能力。

## 当前阶段

**M18：大学词库扩充至 5500 词。**

M18 新增并验收：

- 大学词库大规模扩充：大学词书由 26 词扩充至 5526 词，新增 5500 词，按主题分 40 组（学术论文与研究 / 学术表达与论证 / 心理学与认知 / 哲学与思想基础 / 经济学 / 金融与投资 / 政治学与国际关系 / 社会学与人类学 / 文学与文化理论 / 语言学 / 数学与统计 / 物理学与天文 / 化学 / 生物学 / 医学与健康 / 计算机科学 / 数据科学与人工智能 / 工程与材料 / 环境科学 / 法学 / 商业与管理 / 会计与财务 / 信息技术与网络安全 / 媒体传播与新闻 / 教育 / 艺术与美学 / 全球议题与公共政策 / 研究方法与学术写作 / 学术表达动词 / 学术抽象名词 / 副词与学术连接语 / 地理与地质 / 历史与文明 / 哲学与逻辑学 / 心理学补充 / 职业与职场 / 交通与旅行 / 食品与营养 / 体育与健康 / 建筑与设计），全部含音标、中文释义与中英双语例句；
- 难度档位贴合大学学段：新增词以内在难度 1.25（hw）高阶词为主（55%）、1.1（nw）次之（29%）、1.0（w）基础词为辅（15%），少量 0.9（sw）简单词点缀，与既有档位体系一致；
- 幂等播种无需迁移：`seed_words.rs` 新增词通过 `INSERT OR IGNORE` + word 唯一约束自动入库，`word_count` 自动重算，旧数据与既有复习进度不受影响；与高中及其他词书共享的词沿用既有跨词书共享机制，同词同进度；
- 词条总量：6 本词书共 11108 词条、8362 个唯一种子词（含 M1 遗留词 fluent 共 8363 个可学单词），全库去重校验通过；默认选书（高中+大学）可学词表 7616 词，大学词书 5526 词；
- 回归：`wordbooks_and_catalog_available`、`export_and_clear_learning_data` 等动态计数断言自动适配新总量，Rust 81 项单元/集成测试 + 前端 18 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

**M15：小学词库扩充至 500 词。**

M15 新增并验收：

- 小学词库二次扩充：小学词书由 239 词扩充至 500 词，新增 261 词，按主题分组（动物 / 食物 / 身体 / 家与家居 / 学校 / 家庭与职业 / 时间与数字 / 颜色 / 天气与自然 / 衣物 / 动作动词 / 形容词 / 地点与交通），全部含音标、中文释义与 A1 级英中双语例句；
- 难度档位贴合小学学段：新增词以内在难度 0.6（ew）/ 0.8（lw）/ 0.9（sw）为主，少量 1.0（w），与既有档位体系一致；
- 幂等播种无需迁移：`seed_words.rs` 新增词通过 `INSERT OR IGNORE` + word 唯一约束自动入库，`word_count` 自动重算，旧数据与既有复习进度不受影响；个别与初中词书共享的常用词（如 weather、library、hospital 等）沿用既有跨词书共享机制，同词同进度；
- 词条总量：6 本词书共 2108 词条、2024 个唯一种子词（含 M1 遗留词 fluent 共 2025 个可学单词），全库去重校验通过；
- 回归：`migration_seeds_expected_content` 等动态计数断言自动适配新总量，Rust 81 项单元/集成测试 + 前端 18 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

**M16：初中词库扩充至 1500 词。**

M16 新增并验收：

- 初中词库大规模扩充：初中词书由 28 词扩充至 1500 词，新增 1472 词，按主题分 18 组（学校与学习 / 家庭与亲属人际 / 职业与社会身份 / 身体健康与医疗 / 食物饮品与烹饪 / 天气季节与自然 / 时间日期数字与量词 / 地点建筑交通与旅行 / 动物与植物 / 衣物与购物 / 动作动词·上 / 动作动词·下 / 形容词与描述 / 情绪感受与性格 / 社会法律规则与抽象 / 科技通信与媒体 / 运动休闲与游戏 / 家居与工具），全部含音标、中文释义与 A1-A2 级英中双语例句；
- 难度档位贴合初中学段：新增词以内在难度 1.0（w）为主、0.9（sw）/ 0.8（lw）简单词为辅、1.1（nw）稍难词少量点缀，与既有档位体系一致；
- 幂等播种无需迁移：`seed_words.rs` 新增词通过 `INSERT OR IGNORE` + word 唯一约束自动入库，`word_count` 自动重算，旧数据与既有复习进度不受影响；与小学及其他词书共享的常用词（如 weather、library、hospital 等）沿用既有跨词书共享机制，同词同进度；
- 词条总量：6 本词书共 2108 词条、2024 个唯一种子词（含 M1 遗留词 fluent 共 2025 个可学单词），全库去重校验通过；
- 回归：`migration_seeds_expected_content` 等动态计数断言自动适配新总量，Rust 81 项单元/集成测试 + 前端 18 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

**M17：高中词库扩充至 3500 词。**

M17 新增并验收：

- 高中词库大规模扩充：高中词书由 28 词扩充至 3528 词，新增 3500 词，按主题分 32 组（学校与学习 / 考试与学习品质 / 家庭与亲情 / 人际与社交 / 职业与身份 / 情绪感受·积极 / 情绪感受·消极 / 品格与道德 / 身体与器官 / 健康与疾病 / 食物与饮品 / 地点与建筑 / 城市与乡村·人口 / 交通与旅行 / 自然与环境（天气气候） / 生态与环境 / 动物 / 时间与时代 / 动作动词（日常与身体） / 动作动词（学习与思考） / 动作动词（交际与社会） / 形容词与描述（一） / 形容词与描述（二） / 社会与法律·工作与经济 / 科技与通信 / 文化艺术·历史文学音乐 / 体育与健康生活 / 家居与物品 / 抽象名词 / 副词与连接词 / 植物与自然地理 / 高频动词与行为补充），全部含音标、中文释义与中英双语例句；
- 难度档位贴合高中学段：新增词以内在难度 1.0（w）为主、1.1（nw）稍难词为辅、0.9（sw）/ 0.8（lw）简单词少量点缀，与既有档位体系一致；
- 幂等播种无需迁移：`seed_words.rs` 新增词通过 `INSERT OR IGNORE` + word 唯一约束自动入库，`word_count` 自动重算，旧数据与既有复习进度不受影响；与小学/初中及其他词书共享的常用词沿用既有跨词书共享机制，同词同进度；
- 词条总量：6 本词书共 5608 词条、4524 个唯一种子词（含 M1 遗留词 fluent 共 4525 个可学单词），全库去重校验通过；
- 回归：`migration_seeds_expected_content` 等动态计数断言自动适配新总量，Rust 81 项单元/集成测试 + 前端 18 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

**M13：删除学习记录已完成。**

M13 新增并验收：

- 删除学习记录：单词学习页单词卡片与生词本/熟词本列表中新增「删除学习记录」；删除前二次确认，删除后清空该词的复习事件、移出生词本/熟词本，并把复习状态重置为未学习（新词，难度恢复为词书基准），单词本身保留在词库中，可再次作为新词学习；与「清除学习数据」不同，该操作只影响单个单词，不触碰其他数据；
- 前端接入：`api.deleteLearningRecords(wordId)` 新增 Tauri 命令 `delete_learning_records`，单词卡片删除按钮在确认后移除当前卡并刷新概览，生词/熟词列表删除后从两侧列表同时移除；
- 回归：新增 `delete_learning_records_resets_word`、`delete_learning_records_unknown_word_errors` 与前端删除确认/取消用例，Rust 81 项单元/集成测试 + 前端 18 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

**M14：取消「学习计划」，统一以每日学习数量为准。**

M14 变更并验收：

- 移除学习计划：设置页删除「学习计划」面板（每日单词/句子/听力/朗读/对话/时长配额与启用开关），Dashboard 不再按计划配额计算完成度与今日路线，消除「学习计划」与「每日学习数量」两套单词配额的冲突；
- 统一以每日学习数量为准：Dashboard 今日单词进度、单词复习卡与今日路线改为以「每日学习数量」为单词目标（复习优先、余量新学），句子/听力/朗读/对话模块只展示实际完成量；`TodayProgress` 移除 `plan`/`plan_percent`，`StudyPlan`、后端 `get_study_plan`/`save_study_plan` 与前端 `getStudyPlan`/`saveStudyPlan` 全部下线；
- 回归：`study_plan_and_reminder_roundtrip` 收敛为 `reminder_settings_roundtrip`，`today_progress_ties_plan_to_events` 移除计划断言，Rust 单元/集成测试 + 前端 18 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

M12 新增并验收：

- 学习数量统一：设置页将「每日新词」+「每日复习」合并为「每日学习总量」；每天先计算当日到期复习数，总量减去复习数即为今日新学数（复习优先，到期超量时新学为 0），未来 7 天将到期的复习在配额允许时提前少量分散（削峰填谷，提前量不超过剩余额度一半，给新词留足空间）；旧配置自动迁移（`daily_new + daily_review → daily_total`）并写回；
- 复习间隔预测提示：单词学习页「重来 / 模糊 / 认识」按钮下方直接显示后端预测的复习间隔（重来 = 10 分钟后；模糊 = 隔几个单词或 N 天后；认识 = N 天后 / 完成进入熟词本 / 隔几个单词），恢复期会提示「还需再认识一次」，预测值由后端 `preview` 字段下发，前端不伪造；
- 重来后二次确认：单词「重来」后需连续两次「认识」才算真记住——第一次认识仅确认（1 天后复习，`known_streak=1`），第二次认识才按平方序列推进（2 → 6 → 15 → 31 → 56 → 92 → 100 封顶），中途「重来 / 模糊」会打断确认计数；再次「重来」后从确认重新开始；
- 数据迁移：SQLite 迁移 v8 新增 `known_streak` 列，旧数据自动升级保留；
- 回归：新增 `relapsed_known_twice_schedules_plan`、`again_then_again_keeps_known_streak`、`preview_reflects_recovery_confirm`、`daily_queue_reviews_crowd_out_new_words`、`learning_config_legacy_migration` 等测试，Rust 79 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

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

- 词书体系：内置 6 本词书（小学 / 初中 / 高中 / 大学 / 雅思 / 专业），共 2108 词条、2024 个唯一种子词（含 M1 遗留词 fluent 共 2025 个可学单词），`seed_words.rs` 幂等播种，M1 遗留单词自动映射进对应词书；
- 多词书选择：设置页可勾选任意词书组合（默认 高中 + 大学），学习范围随之生效，也可全局关闭范围限制；
- 选词模式：支持「随机选词」（从已勾选词书中按每日配额随机抽取新词）与「自定义选词」（在范围内逐词勾选想学的单词）；
- 每日计划：独立设置每日「新词数」与「复习数」（默认 10 / 20），每日队列先到期复习、后随机新词，按配额截断；
- 自适应记忆算法：以「词书难度 × 单词内在难度」计算有效难度，复习间隔 `effective_interval = round(base_interval / difficulty)`，难度越高复习越频繁；评分仍按 SM-2 风格四档（重来/模糊/认识/熟练）推进阶段；
- 学习范围统计：概览、复习队列、掌握度、今日进度均按已勾选词书范围统计；
- 学习配置持久化：词书勾选、选词模式、每日配额仅存本地（`app_settings`），重启保留；
- 回归：Rust 60 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

M10 新增并验收：

- 三档评分与熟词本：单词复习改为「重来 / 模糊 / 认识」三档（重来 10 分钟后当次重学，模糊隔几个单词再练，认识按计划推进），另设「熟悉」直接加入熟词本（记录掌握时间、`due_at` 置为永不，不再进入任何复习队列），生词本页新增「熟词」Tab，可一键移回复习队列；复习间隔封顶 100 天，达到封顶间隔后仍答「认识」会自动进入熟词本，不再无限循环复习；
- 记忆状态自适应重学：重来彻底重置节奏（reps 归零、lapses+1、ease-0.20），重来后需连续两次「认识」才算真记住（第一次认识仅确认、1 天后复习，第二次认识才按平方序列推进）；按平方递增序列自动规划复习间隔（1 → 2 → 6 → 15 → 31 → 56 → 92 → 100 封顶，第 n 次 = 1 + Σk²），每次重来再从确认重新开始；模糊结合历史记忆状态——曾遗忘 / ease 偏低 / 难度高时隔几个单词立刻重学，否则按当前间隔 × 0.8 做短间隔复习，避免一刀切；
- 防扎堆复习（参考墨墨背单词）：每日学习总量按「复习优先」分配——先取当日到期复习数，剩余配额用于新词（到期超量时新学为 0）；到期复习不足时从未来 7 天内将到期的复习中提前少量补足（削峰填谷，提前量不超过剩余额度一半）；复习与新词交错排列、首卡必为复习，避免某天集体到期而当天没有新学；
- 间隔随机扰动：复习间隔大于 5 天时按 ±20% 加入确定性随机扰动（种子 = 单词 ID × 当天，同一张卡同一天稳定、跨天变化），首次「认识」按难度给较长基准（约 10 天 ~ 几十天，难度越低间隔越长）并加 ±10% 扰动，后续复习以当前间隔 × ease 推进，进一步打散复习落点、避免同日扎堆；
- 单词难度细化：内在难度由 3 档扩展为 9 档（0.6 / 0.8 / 0.9 / 1.0 / 1.1 / 1.2 / 1.25 / 1.4 / 1.5），92 个种子词按学段重标档位，简单词降档、学术/专业词升档；
- 数据迁移：SQLite 迁移 v7 新增 `mastered_at` 列，旧数据自动升级保留；
- 回归：Rust 74 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。

M11 新增并验收：

- 小学词库大规模扩充：小学词书由 28 词扩充至 239 词，按主题分 13 组（数字与时间 / 颜色 / 动物 / 食物与饮料 / 学校与学习 / 家庭与人 / 身体 / 天气与自然 / 衣物 / 家与家居 / 动作 / 形容词 / 其他常用），全部含音标、中文释义与 A1 级英中双语例句；
- 难度档位贴合小学学段：新增词以内在难度 0.6（ew）/ 0.8（lw）/ 0.9（sw）为主，少量 1.0（w），避免过度拔高，保证初学友好；
- 幂等播种无需迁移：`seed_words.rs` 新增词通过 `INSERT OR IGNORE` + word 唯一约束自动入库，`word_count` 自动重算，旧数据与既有复习进度不受影响；
- 词条总量：6 本词书共 2108 词条、2024 个唯一种子词（含 M1 遗留词 fluent 共 2025 个可学单词），全库去重校验通过；
- 回归：新增 `primary_book_has_substantial_words`（小学 ≥ 150 词）与 `migration_seeds_expected_content` 动态计数断言，Rust 74 项单元/集成测试 + 前端 13 项测试全绿，`pnpm typecheck`、`pnpm build`、`cargo test` 通过。
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

- 单词学习（6 本内置词书、随机/自定义选词、每日学习数量（复习优先、余量新学）、难度自适应 SRS、复习队列、评分、生词本）、句子（主动回忆、对/错事件）、听力（播放/变速/逐句循环/听写）、朗读（录音、离线 ASR、逐词反馈）与 AI 对话（场景对话、异步纠错、语音输入）已形成完整学习闭环；
- AI 对话依赖用户自备 OpenAI 兼容服务（Base URL / API Key / 模型，仅存本地数据库），TTS、流式链路与发音评分（音素/韵律）为 M11 规划；
- 听力音频为 Windows 离线 TTS 生成的种子语料，后续里程碑将支持用户导入真实音频；
- 不包含静态伪造评分或模拟 AI 数据；AI 密钥只存本地数据库，不进前端构建；
- 版本与更新：更新检查基于 GitHub Releases（`xlcblues/janus`），需要联网；无新版、网络失败或平台不支持均有明确中文降级提示；
- 发布产物：NSIS 安装程序已产出（当前用户安装、简体中文），尚未代码签名，干净机器安装/卸载/升级 E2E 与商店/官网发布留待 M11；
- 学习数据本地保存、可导出、可清除（清除前自动备份）、可删除单个单词的学习记录（重置为新词）。

## 下一步：M19

M19 聚焦实时链路与发布收尾：流式 ASR/LLM 与 TTS、语音打断、发音评分（音素/韵律）、真实音频导入与云同步接口，以及 Windows 代码签名、干净机器安装/升级/回滚专项验证与发布检查清单。

## 设计边界

- 与 Mars、Ceres、Neptune、Titan、Miranda、Europa 共享 Windows 桌面壳层与系列视觉语法；
- 不复制既有项目图形，只继承单色几何、节点、纸墨底色与克制层级；
- 所有产品颜色必须来自语义 Token；
- AI 语音、发音评分与纠错必须提供失败降级和可解释反馈。

## 验证状态

截至 2026-11-27，`pnpm typecheck`、`pnpm test`、`pnpm build`、`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo check --examples --manifest-path src-tauri/Cargo.toml` 与 `cargo test --manifest-path src-tauri/Cargo.toml` 均通过（Rust 81 项 + 前端 18 项）。

