use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::seed_audio::{wav_duration_ms, SEED_CLIPS};
use crate::seed_words::BOOKS as SEED_BOOKS;

pub const SCHEMA_VERSION: i64 = 7;

const MIGRATION_1: &str = r#"
CREATE TABLE IF NOT EXISTS words (
    id INTEGER PRIMARY KEY,
    word TEXT NOT NULL UNIQUE,
    phonetic TEXT NOT NULL,
    meaning TEXT NOT NULL,
    example_en TEXT NOT NULL,
    example_zh TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'a2',
    lang TEXT NOT NULL DEFAULT 'en'
);

CREATE TABLE IF NOT EXISTS word_states (
    word_id INTEGER PRIMARY KEY REFERENCES words(id) ON DELETE CASCADE,
    stage TEXT NOT NULL DEFAULT 'new',
    interval_days INTEGER NOT NULL DEFAULT 0,
    ease REAL NOT NULL DEFAULT 2.5,
    repetitions INTEGER NOT NULL DEFAULT 0,
    lapses INTEGER NOT NULL DEFAULT 0,
    due_at INTEGER NOT NULL,
    last_reviewed_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sentences (
    id INTEGER PRIMARY KEY,
    en TEXT NOT NULL UNIQUE,
    zh TEXT NOT NULL,
    lang TEXT NOT NULL DEFAULT 'en'
);

CREATE TABLE IF NOT EXISTS sentence_states (
    sentence_id INTEGER PRIMARY KEY REFERENCES sentences(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'new',
    attempts INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    last_result INTEGER,
    last_attempted_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS vocabulary (
    word_id INTEGER PRIMARY KEY REFERENCES words(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL,
    note TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS study_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    ref_id INTEGER NOT NULL,
    result INTEGER NOT NULL,
    rating INTEGER,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_word_states_due ON word_states(due_at, stage);
CREATE INDEX IF NOT EXISTS idx_study_events_created ON study_events(created_at);
CREATE INDEX IF NOT EXISTS idx_study_events_kind ON study_events(kind);

INSERT INTO words (id, word, phonetic, meaning, example_en, example_zh, level) VALUES
    (1, 'threshold', '/ˈθreʃhəʊld/', '门槛；临界点', 'We are approaching the threshold of a new era.', '我们正接近一个新时代的门槛。', 'b1'),
    (2, 'deliberate', '/dɪˈlɪbərət/', '深思熟虑的；刻意的', 'She made a deliberate effort to speak slowly.', '她刻意放慢语速。', 'b1'),
    (3, 'nuance', '/ˈnjuːɑːns/', '细微差别', 'The writer captures every nuance of the dialogue.', '作者捕捉到了对话中的每个细微差别。', 'b2'),
    (4, 'serendipity', '/ˌserənˈdɪpəti/', '意外发现美好事物的能力', 'Finding that old letter was pure serendipity.', '找到那封旧信完全是意外之喜。', 'b2'),
    (5, 'resilient', '/rɪˈzɪliənt/', '有韧性的；能恢复的', 'Children are often more resilient than adults expect.', '孩子往往比大人想象中更有韧性。', 'b2'),
    (6, 'articulate', '/ɑːˈtɪkjuleɪt/', '清晰表达（的）', 'He is articulate when explaining complex ideas.', '解释复杂想法时他表达得很清楚。', 'b2'),
    (7, 'feasible', '/ˈfiːzəbl/', '可行的', 'A full redesign is not feasible this quarter.', '本季度全面改版不可行。', 'b1'),
    (8, 'sustain', '/səˈsteɪn/', '维持；支撑', 'Small habits can sustain long-term progress.', '小习惯可以维持长期进步。', 'b1'),
    (9, 'acknowledge', '/əkˈnɒlɪdʒ/', '承认；致谢', 'She acknowledged the effort behind the report.', '她认可了这份报告背后的付出。', 'b1'),
    (10, 'ambiguous', '/æmˈbɪɡjuəs/', '模棱两可的；含糊的', 'The ending is deliberately ambiguous.', '这个结尾故意留白。', 'b2'),
    (11, 'coherent', '/kəʊˈhɪərənt/', '连贯的；有条理的', 'The essay needs a more coherent structure.', '这篇文章需要更连贯的结构。', 'b2'),
    (12, 'concise', '/kənˈsaɪs/', '简洁的', 'Keep your summary concise and clear.', '让你的总结简洁清晰。', 'b1'),
    (13, 'empathy', '/ˈempəθi/', '共情；同理心', 'Good listeners show empathy without judgment.', '好的倾听者不带评判地共情。', 'b2'),
    (14, 'fluent', '/ˈfluːənt/', '流利的', 'She became fluent in English within two years.', '她在两年内英语变得流利。', 'a2'),
    (15, 'generate', '/ˈdʒenəreɪt/', '产生；生成', 'This exercise will generate new ideas.', '这个练习会产生新想法。', 'b1'),
    (16, 'hypothesis', '/haɪˈpɒθəsɪs/', '假说；假设', 'The data supports our initial hypothesis.', '数据支持我们最初的假设。', 'b2'),
    (17, 'inevitable', '/ɪnˈevɪtəbl/', '不可避免的', 'Change is inevitable in any growing city.', '任何成长中的城市都不可避免变化。', 'b1'),
    (18, 'negotiate', '/nɪˈɡəʊʃieɪt/', '谈判；协商', 'We need to negotiate a better deadline.', '我们需要协商一个更好的截止时间。', 'b2'),
    (19, 'perspective', '/pəˈspektɪv/', '视角；观点', 'Travel gives you a new perspective on life.', '旅行会给你看待生活的新视角。', 'b1'),
    (20, 'subtle', '/ˈsʌtl/', '微妙的；不易察觉的', 'There is a subtle difference between the two words.', '这两个词之间有微妙的差别。', 'b2');

INSERT INTO sentences (id, en, zh) VALUES
    (1, 'It depends on how you look at it.', '取决于你从哪个角度看。'),
    (2, 'Could you say that again, please?', '可以请你再说一遍吗？'),
    (3, 'I am running out of time.', '我的时间快不够了。'),
    (4, 'Let us take a closer look.', '我们再仔细看一下。'),
    (5, 'That makes perfect sense.', '这完全说得通。'),
    (6, 'I will keep that in mind.', '我会记住这一点。'),
    (7, 'It is worth giving it a try.', '值得试一试。'),
    (8, 'We are on the same page.', '我们看法一致。'),
    (9, 'Could you be more specific?', '你能说得更具体一点吗？'),
    (10, 'I have a lot on my plate right now.', '我现在手头有很多事。'),
    (11, 'That is a great question.', '这是个好问题。'),
    (12, 'Let me walk you through it.', '我来带你过一遍。'),
    (13, 'It is not as simple as it sounds.', '事情没有听起来那么简单。'),
    (14, 'I would rather get it right than hurry.', '我宁可做对，也不愿赶时间。'),
    (15, 'The sooner we start, the better.', '我们越早开始越好。'),
    (16, 'Could you give me a hand?', '你能帮我一下吗？'),
    (17, 'I will get back to you tomorrow.', '我明天再答复你。'),
    (18, 'Practice makes progress, not perfection.', '练习带来进步，而不是完美。'),
    (19, 'Every small step counts.', '每一步都很重要。'),
    (20, 'You will get there with time.', '假以时日，你会做到的。');

INSERT OR IGNORE INTO word_states (word_id, stage, interval_days, ease, repetitions, lapses, due_at, created_at)
SELECT id, 'new', 0, 2.5, 0, 0, CAST(strftime('%s', 'now') AS INTEGER), CAST(strftime('%s', 'now') AS INTEGER) FROM words;

INSERT OR IGNORE INTO sentence_states (sentence_id, status, attempts, correct_count, last_result, last_attempted_at, created_at)
SELECT id, 'new', 0, 0, NULL, NULL, CAST(strftime('%s', 'now') AS INTEGER) FROM sentences;
"#;

const MIGRATION_2: &str = r#"
CREATE TABLE IF NOT EXISTS listening_clips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    lang TEXT NOT NULL DEFAULT 'en',
    source TEXT NOT NULL DEFAULT 'import',
    file_name TEXT NOT NULL DEFAULT '',
    duration_ms INTEGER NOT NULL DEFAULT 0,
    segment_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS listening_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    clip_id INTEGER NOT NULL REFERENCES listening_clips(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    text_en TEXT NOT NULL,
    text_zh TEXT NOT NULL DEFAULT '',
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS dictation_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    clip_id INTEGER NOT NULL REFERENCES listening_clips(id) ON DELETE CASCADE,
    segment_id INTEGER NOT NULL REFERENCES listening_segments(id) ON DELETE CASCADE,
    expected TEXT NOT NULL,
    submitted TEXT NOT NULL,
    correct INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS deleted_seed_clips (
    seed_index INTEGER PRIMARY KEY,
    deleted_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_listening_segments_clip ON listening_segments(clip_id, seq);
CREATE INDEX IF NOT EXISTS idx_dictation_attempts_clip ON dictation_attempts(clip_id, created_at);
"#;

const MIGRATION_3: &str = r#"
CREATE TABLE IF NOT EXISTS speaking_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    clip_id INTEGER NOT NULL REFERENCES listening_clips(id) ON DELETE CASCADE,
    segment_id INTEGER NOT NULL REFERENCES listening_segments(id) ON DELETE CASCADE,
    expected TEXT NOT NULL,
    transcribed TEXT NOT NULL,
    word_matches INTEGER NOT NULL,
    word_total INTEGER NOT NULL,
    accuracy_percent INTEGER NOT NULL,
    confidence REAL NOT NULL DEFAULT 0,
    low_confidence INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_speaking_attempts_segment ON speaking_attempts(segment_id, created_at);
CREATE INDEX IF NOT EXISTS idx_speaking_attempts_clip ON speaking_attempts(clip_id, created_at);
"#;

const MIGRATION_4: &str = r#"
CREATE TABLE IF NOT EXISTS conversation_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS conversation_scenes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    goal TEXT NOT NULL,
    role TEXT NOT NULL,
    user_goal TEXT NOT NULL,
    target_words TEXT NOT NULL DEFAULT '',
    level TEXT NOT NULL DEFAULT 'a2',
    greeting TEXT NOT NULL,
    lang TEXT NOT NULL DEFAULT 'en',
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS conversation_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scene_id INTEGER NOT NULL REFERENCES conversation_scenes(id),
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    turn_count INTEGER NOT NULL DEFAULT 0,
    summary TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS conversation_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES conversation_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    correction TEXT,
    latency_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_conversation_messages_session ON conversation_messages(session_id, id);
CREATE INDEX IF NOT EXISTS idx_conversation_sessions_created ON conversation_sessions(created_at);

INSERT OR IGNORE INTO conversation_scenes (id, title, goal, role, user_goal, target_words, level, greeting, lang) VALUES
    (1, '咖啡馆点单', '练习在咖啡馆点单、询问推荐并完成结账。', '你是咖啡馆店员 Emma，友好耐心，说话简短自然。', '点一杯饮料、询问推荐并完成结账。', 'menu, order, recommend, latte, bill', 'a2', 'Hi there! Welcome to our cafe. What would you like to order today?', 'en'),
    (2, '机场问路', '练习在机场问路、确认登机口与时间。', '你是机场工作人员 David，语气温和，指引清晰。', '找到登机口并确认登机时间。', 'gate, boarding, terminal, flight, depart', 'a2', 'Good morning! How can I help you at the airport today?', 'en'),
    (3, '酒店入住', '练习办理酒店入住、提出需求并咨询设施。', '你是前台接待员 Mia，礼貌高效。', '办理入住，并询问早餐时间与 Wi-Fi。', 'check-in, reservation, breakfast, wifi, key', 'a2', 'Welcome to the hotel! Do you have a reservation with us?', 'en'),
    (4, '求职面试', '练习面试中自我介绍并回答问题。', '你是面试官 Mr. Lee，专业但不严肃。', '自我介绍并回答 2–3 个面试问题。', 'experience, strength, teamwork, career, goal', 'b1', 'Thanks for coming in. Could you start by telling me a little about yourself?', 'en'),
    (5, '周末计划', '练习谈论周末安排与回应邀约。', '你是朋友 Sam，轻松随意，会追问细节。', '聊一聊周末计划并回应邀约。', 'plan, weekend, invite, join, free', 'a2', 'Hey! What are you planning to do this weekend?', 'en');
"#;

const MIGRATION_5: &str = r#"
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

const MIGRATION_6: &str = r#"
CREATE TABLE IF NOT EXISTS wordbooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    level TEXT NOT NULL DEFAULT 'a2',
    difficulty REAL NOT NULL DEFAULT 1.0,
    word_count INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    lang TEXT NOT NULL DEFAULT 'en'
);

CREATE TABLE IF NOT EXISTS wordbook_words (
    wordbook_id INTEGER NOT NULL REFERENCES wordbooks(id) ON DELETE CASCADE,
    word_id INTEGER NOT NULL REFERENCES words(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (wordbook_id, word_id)
);

CREATE INDEX IF NOT EXISTS idx_wordbook_words_word ON wordbook_words(word_id);
CREATE INDEX IF NOT EXISTS idx_wordbook_words_book ON wordbook_words(wordbook_id, word_id);

ALTER TABLE words ADD COLUMN difficulty REAL NOT NULL DEFAULT 1.0;
ALTER TABLE word_states ADD COLUMN difficulty REAL NOT NULL DEFAULT 1.0;
ALTER TABLE word_states ADD COLUMN introduced_at INTEGER;
"#;

const MIGRATION_7: &str = r#"
ALTER TABLE word_states ADD COLUMN mastered_at INTEGER;
"#;

const MIGRATION_8: &str = r#"
ALTER TABLE word_states ADD COLUMN known_streak INTEGER NOT NULL DEFAULT 0;
"#;

#[derive(Debug)]
pub enum AppError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Sqlite(error) => write!(formatter, "database error: {error}"),
            AppError::Io(error) => write!(formatter, "io error: {error}"),
            AppError::Json(error) => write!(formatter, "json error: {error}"),
            AppError::NotFound(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::Sqlite(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Json(error)
    }
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            AppError::NotFound(message) => message.clone(),
            AppError::Sqlite(rusqlite::Error::QueryReturnedNoRows) => {
                "没有找到对应的学习项，请刷新后重试。".to_string()
            }
            _ => "本地数据操作失败，请稍后重试。".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewPreview {
    /// 重来（评分 1）后的预测：始终为 0（10 分钟后当次重学）。
    pub again_days: i64,
    /// 模糊（评分 2）后的预测：0 = 隔几个单词再见；>0 = 几天后复习。
    pub fuzzy_days: i64,
    /// 认识（评分 3）后的预测：>0 = 几天后复习；-1 = 直接进入熟词本；0 = 隔几个单词再见。
    pub known_days: i64,
    /// 认识是否处于「恢复期第 1 次」：还需再认识一次才算真正记住。
    pub known_needs_again: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordCard {
    pub id: i64,
    pub word: String,
    pub phonetic: String,
    pub meaning: String,
    pub example_en: String,
    pub example_zh: String,
    pub stage: String,
    pub interval_days: i64,
    pub due_at: i64,
    pub difficulty: f64,
    pub in_vocabulary: bool,
    pub mastered_at: Option<i64>,
    /// 恢复期连续「认识」计数（0..=1，达到 2 后归零）。
    pub known_streak: i64,
    /// 当前状态下各档评分的复习间隔预测，用于学习页按钮提示。
    pub preview: ReviewPreview,
}

#[derive(Debug, Clone, Serialize)]
pub struct MasteredWord {
    pub word_id: i64,
    pub word: String,
    pub phonetic: String,
    pub meaning: String,
    pub difficulty: f64,
    pub mastered_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Wordbook {
    pub id: i64,
    pub code: String,
    pub title: String,
    pub description: String,
    pub level: String,
    pub difficulty: f64,
    pub word_count: i64,
    pub sort_order: i64,
    pub lang: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyQueue {
    pub cards: Vec<WordCard>,
    pub new_count: i64,
    pub review_count: i64,
    pub total_done_today: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WordbookCatalogEntry {
    pub word_id: i64,
    pub word: String,
    pub phonetic: String,
    pub meaning: String,
    pub difficulty: f64,
    pub level: String,
    pub book_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Overview {
    pub total_words: i64,
    pub due_words: i64,
    pub learned_words: i64,
    pub mastered_words: i64,
    pub total_reviews: i64,
    pub today_reviews: i64,
    pub review_queue: Vec<WordCard>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StudyStats {
    pub total_reviews: i64,
    pub correct_reviews: i64,
    pub accuracy_percent: i64,
    pub total_duration_min: i64,
    pub today_reviews: i64,
    pub today_duration_min: i64,
    pub due_words: i64,
    pub mastered_words: i64,
    pub vocabulary_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SentenceCard {
    pub id: i64,
    pub en: String,
    pub zh: String,
    pub status: String,
    pub attempts: i64,
    pub correct_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VocabularyEntry {
    pub word_id: i64,
    pub word: String,
    pub phonetic: String,
    pub meaning: String,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventRow {
    pub id: i64,
    pub kind: String,
    pub ref_id: i64,
    pub result: i64,
    pub rating: Option<i64>,
    pub duration_ms: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportPayload {
    pub exported_at: i64,
    pub schema_version: i64,
    pub words: Vec<WordCard>,
    pub sentences: Vec<SentenceCard>,
    pub events: Vec<EventRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportResult {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClearResult {
    pub backup_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListeningClip {
    pub id: i64,
    pub title: String,
    pub lang: String,
    pub source: String,
    pub file_name: String,
    pub duration_ms: i64,
    pub segment_count: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListeningSegment {
    pub id: i64,
    pub clip_id: i64,
    pub seq: i64,
    pub text_en: String,
    pub text_zh: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListeningDetail {
    pub clip: ListeningClip,
    pub segments: Vec<ListeningSegment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioCatalogEntry {
    pub id: i64,
    pub title: String,
    pub text_zh: String,
    pub source: String,
    pub file_name: String,
    pub file_path: String,
    pub duration_ms: i64,
    pub segment_count: i64,
    pub dictation_attempts: i64,
    pub dictation_correct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DictationResult {
    pub id: i64,
    pub clip_id: i64,
    pub segment_id: i64,
    pub expected: String,
    pub submitted: String,
    pub correct: bool,
    pub duration_ms: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeakClip {
    pub clip_id: i64,
    pub title: String,
    pub attempts: i64,
    pub correct: i64,
    pub accuracy_percent: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DictationStats {
    pub total_attempts: i64,
    pub correct_attempts: i64,
    pub accuracy_percent: i64,
    pub today_attempts: i64,
    pub today_correct: i64,
    pub weakest_clips: Vec<WeakClip>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteResult {
    pub file_name: String,
}


#[derive(Debug, Clone, Serialize)]
pub struct WordFeedback {
    pub word: String,
    /// "correct" | "missing" | "changed" | "extra"
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeakingResult {
    pub id: i64,
    pub clip_id: i64,
    pub segment_id: i64,
    pub expected: String,
    pub transcribed: String,
    pub word_matches: i64,
    pub word_total: i64,
    pub accuracy_percent: i64,
    pub confidence: f64,
    pub low_confidence: bool,
    pub duration_ms: i64,
    pub created_at: i64,
    pub words: Vec<WordFeedback>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeakingWeakSegment {
    pub segment_id: i64,
    pub clip_id: i64,
    pub title: String,
    pub attempts: i64,
    pub accuracy_percent: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpeakingStats {
    pub total_attempts: i64,
    pub today_attempts: i64,
    pub accuracy_percent: i64,
    pub low_confidence_attempts: i64,
    pub total_duration_ms: i64,
    pub weakest_segments: Vec<SpeakingWeakSegment>,
}

#[derive(Debug, Clone)]
struct SrsState {
    stage: String,
    interval_days: i64,
    ease: f64,
    repetitions: i64,
    lapses: i64,
    due_at: i64,
    difficulty: f64,
    introduced_at: Option<i64>,
    mastered_at: Option<i64>,
    known_streak: i64,
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn configure(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;",
    )
}

fn migrate(conn: &mut Connection) -> SqliteResult<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_1)?;
        tx.execute_batch("PRAGMA user_version = 1;")?;
        tx.commit()?;
    }
    if version < 2 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_2)?;
        tx.execute_batch("PRAGMA user_version = 2;")?;
        tx.commit()?;
    }
    if version < 3 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_3)?;
        tx.execute_batch("PRAGMA user_version = 3;")?;
        tx.commit()?;
    }
    if version < 4 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_4)?;
        tx.execute_batch("PRAGMA user_version = 4;")?;
        tx.commit()?;
    }
    if version < 5 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_5)?;
        tx.execute_batch("PRAGMA user_version = 5;")?;
        tx.commit()?;
    }
    if version < 6 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_6)?;
        tx.execute_batch("PRAGMA user_version = 6;")?;
        tx.commit()?;
    }
    if version < 7 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_7)?;
        tx.execute_batch("PRAGMA user_version = 7;")?;
        tx.commit()?;
    }
    if version < 8 {
        let tx = conn.transaction()?;
        tx.execute_batch(MIGRATION_8)?;
        tx.execute_batch("PRAGMA user_version = 8;")?;
        tx.commit()?;
    }
    Ok(())
}

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&mut conn)?;
    ensure_wordbook_seed(&mut conn)?;
    Ok(conn)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn open_in_memory() -> Result<Connection, AppError> {
    let mut conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&mut conn)?;
    ensure_wordbook_seed(&mut conn)?;
    Ok(conn)
}

/// 幂等写入 M9 词书与种子词库：词书、单词、词书-单词关联与单词状态。
/// 已有单词只补充关联，不覆盖用户数据；新单词同步创建初始 word_states。
pub fn ensure_wordbook_seed(conn: &mut Connection) -> Result<(), AppError> {
    let now = now_secs();
    let tx = conn.transaction()?;
    for book in SEED_BOOKS {
        let existing_book: Option<i64> = tx
            .query_row(
                "SELECT id FROM wordbooks WHERE code = ?1",
                params![book.code],
                |row| row.get(0),
            )
            .optional()?;
        let book_id: i64 = match existing_book {
            Some(id) => id,
            None => {
                tx.execute(
                    "INSERT INTO wordbooks (code, title, description, level, difficulty, word_count, sort_order, lang)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, 'en')",
                    params![
                        book.code,
                        book.title,
                        book.description,
                        book.level,
                        book.difficulty,
                        book.sort_order
                    ],
                )?;
                tx.last_insert_rowid()
            }
        };
        for (index, seed) in book.words.iter().enumerate() {
            let existing_word: Option<i64> = tx
                .query_row(
                    "SELECT id FROM words WHERE word = ?1",
                    params![seed.word],
                    |row| row.get(0),
                )
                .optional()?;
            let word_id: i64 = match existing_word {
                Some(id) => id,
                None => {
                    tx.execute(
                        "INSERT INTO words (word, phonetic, meaning, example_en, example_zh, level, lang, difficulty)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'en', ?7)",
                        params![
                            seed.word,
                            seed.phonetic,
                            seed.meaning,
                            seed.example_en,
                            seed.example_zh,
                            book.level,
                            seed.difficulty
                        ],
                    )?;
                    let id = tx.last_insert_rowid();
                    let effective = (book.difficulty * seed.difficulty).clamp(0.5, 2.0);
                    tx.execute(
                        "INSERT INTO word_states (word_id, stage, interval_days, ease, repetitions, lapses,
                                                  due_at, created_at, difficulty, introduced_at)
                         VALUES (?1, 'new', 0, 2.5, 0, 0, ?2, ?2, ?3, NULL)",
                        params![id, now, effective],
                    )?;
                    id
                }
            };
            tx.execute(
                "INSERT OR IGNORE INTO wordbook_words (wordbook_id, word_id, seq) VALUES (?1, ?2, ?3)",
                params![book_id, word_id, index as i64 + 1],
            )?;
        }
        tx.execute(
            "UPDATE wordbooks SET word_count = (SELECT COUNT(*) FROM wordbook_words WHERE wordbook_id = ?1)
             WHERE id = ?1",
            params![book_id],
        )?;
    }

    // 迁移 1 遗留单词按难度映射进对应词书，保证旧数据进入学习队列且可被统计。
    let mut orphan_stmt = tx.prepare(
        "SELECT w.id, w.level FROM words w
         WHERE NOT EXISTS (SELECT 1 FROM wordbook_words ww WHERE ww.word_id = w.id)",
    )?;
    let orphans: Vec<(i64, String)> = orphan_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<SqliteResult<Vec<_>>>()?;
    drop(orphan_stmt);
    let level_book: &[(&str, &str)] = &[
        ("a1", "primary"),
        ("a2", "primary"),
        ("b1", "senior"),
        ("b2", "college"),
        ("c1", "ielts"),
        ("c2", "professional"),
    ];
    for (word_id, level) in orphans {
        let target = level_book
            .iter()
            .find(|(lvl, _)| *lvl == level)
            .map(|(_, code)| *code);
        if let Some(code) = target {
            let book_id: Option<i64> = tx
                .query_row("SELECT id FROM wordbooks WHERE code = ?1", [code], |row| row.get(0))
                .optional()?;
            if let Some(book_id) = book_id {
                let seq: i64 = tx.query_row(
                    "SELECT COALESCE(MAX(seq), 0) + 1 FROM wordbook_words WHERE wordbook_id = ?1",
                    [book_id],
                    |row| row.get(0),
                )?;
                tx.execute(
                    "INSERT OR IGNORE INTO wordbook_words (wordbook_id, word_id, seq) VALUES (?1, ?2, ?3)",
                    params![book_id, word_id, seq],
                )?;
            }
        }
    }
    // 迁移 1 遗留单词的难度快照按所属词书补齐：只在难度仍为默认 1.0 时更新，
    // 不覆盖新词已按「词书 × 内在难度」计算的难度；重复打开幂等。
    tx.execute(
        "UPDATE word_states
         SET difficulty = (
             SELECT b.difficulty FROM wordbook_words ww
             JOIN wordbooks b ON b.id = ww.wordbook_id
             WHERE ww.word_id = word_states.word_id
             ORDER BY b.sort_order ASC
             LIMIT 1
         )
         WHERE difficulty = 1.0
           AND word_id IN (SELECT word_id FROM wordbook_words)",
        [],
    )?;

    let book_ids: Vec<i64> = tx
        .prepare("SELECT id FROM wordbooks")?
        .query_map([], |row| row.get(0))?
        .collect::<SqliteResult<Vec<_>>>()?;
    for book_id in book_ids {
        tx.execute(
            "UPDATE wordbooks SET word_count = (SELECT COUNT(*) FROM wordbook_words WHERE wordbook_id = ?1)
             WHERE id = ?1",
            params![book_id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

fn word_card_from_row(row: &rusqlite::Row<'_>) -> SqliteResult<WordCard> {
    let id: i64 = row.get(0)?;
    let difficulty: f64 = row.get(9)?;
    let known_streak: i64 = row.get(12)?;
    let current = SrsState {
        stage: row.get(6)?,
        interval_days: row.get(7)?,
        ease: row.get(13)?,
        repetitions: row.get(14)?,
        lapses: row.get(15)?,
        due_at: row.get(8)?,
        difficulty,
        introduced_at: None,
        mastered_at: row.get(10)?,
        known_streak,
    };
    let preview = srs_preview_for(&current, id);
    Ok(WordCard {
        id,
        word: row.get(1)?,
        phonetic: row.get(2)?,
        meaning: row.get(3)?,
        example_en: row.get(4)?,
        example_zh: row.get(5)?,
        stage: current.stage.clone(),
        interval_days: current.interval_days,
        due_at: current.due_at,
        difficulty: current.difficulty,
        mastered_at: current.mastered_at,
        in_vocabulary: row.get::<_, i64>(11)? != 0,
        known_streak,
        preview,
    })
}

const WORD_CARD_SELECT: &str = "
SELECT w.id, w.word, w.phonetic, w.meaning, w.example_en, w.example_zh,
       s.stage, s.interval_days, s.due_at, s.difficulty, s.mastered_at,
       EXISTS(SELECT 1 FROM vocabulary v WHERE v.word_id = w.id) AS in_vocabulary,
       s.known_streak, s.ease, s.repetitions, s.lapses
FROM words w
JOIN word_states s ON s.word_id = w.id
";

pub fn get_review_queue(conn: &Connection, limit: Option<i64>) -> Result<Vec<WordCard>, AppError> {
    let limit = limit.unwrap_or(10).clamp(1, 100);
    let now = now_secs();
    let (scope, params) = scope_sql(conn)?;
    let mut stmt = conn.prepare(&format!(
        "{WORD_CARD_SELECT} WHERE s.due_at <= ? AND s.stage <> 'mastered' {scope} ORDER BY s.due_at ASC, w.id ASC LIMIT ?"
    ))?;
    let mut all: Vec<&dyn rusqlite::ToSql> = vec![&now];
    all.extend(bind_scope(&params));
    all.push(&limit);
    let cards = stmt
        .query_map(all.as_slice(), word_card_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(cards)
}

/// 当前学习范围（已选词书或自定义选词）对应的 SQL 片段与参数。
/// 片段形如 `AND w.id IN (...)`，参数按出现顺序绑定。
fn scope_sql(conn: &Connection) -> Result<(String, Vec<String>), AppError> {
    let config = get_learning_config(conn)?;
    if !config.enabled {
        return Ok((String::new(), Vec::new()));
    }
    if config.selection_mode == "custom" && !config.custom_word_ids.is_empty() {
        let placeholders = vec!["?"; config.custom_word_ids.len()].join(",");
        let sql = format!("AND w.id IN ({placeholders})");
        let params = config.custom_word_ids.iter().map(|id| id.to_string()).collect();
        return Ok((sql, params));
    }
    if config.book_codes.is_empty() {
        return Ok((String::new(), Vec::new()));
    }
    let placeholders = vec!["?"; config.book_codes.len()].join(",");
    let sql = format!(
        "AND w.id IN (SELECT ww.word_id FROM wordbook_words ww JOIN wordbooks b ON b.id = ww.wordbook_id WHERE b.code IN ({placeholders}))"
    );
    Ok((sql, config.book_codes.clone()))
}

fn bind_scope<'a>(params: &'a [String]) -> Vec<&'a dyn rusqlite::ToSql> {
    params
        .iter()
        .map(|value| value as &dyn rusqlite::ToSql)
        .collect()
}

/// 按当前学习范围统计单词状态数量。predicate 以 ?N 形式占位参数，
/// extras 先绑定，随后追加范围参数；返回范围内单词数。
fn scoped_count(
    conn: &Connection,
    predicate: &str,
    extras: &[&dyn rusqlite::ToSql],
) -> Result<i64, AppError> {
    let (scope, scope_params) = scope_sql(conn)?;
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    params.extend(extras.iter().copied());
    params.extend(bind_scope(&scope_params));
    let sql = format!(
        "SELECT COUNT(*) FROM word_states s JOIN words w ON w.id = s.word_id
         WHERE {predicate} {scope}"
    );
    Ok(conn.query_row(&sql, params.as_slice(), |row| row.get(0))?)
}

pub fn get_wordbooks(conn: &Connection) -> Result<Vec<Wordbook>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, code, title, description, level, difficulty, word_count, sort_order, lang
         FROM wordbooks ORDER BY sort_order ASC, id ASC",
    )?;
    let books = stmt
        .query_map([], |row| {
            Ok(Wordbook {
                id: row.get(0)?,
                code: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                level: row.get(4)?,
                difficulty: row.get(5)?,
                word_count: row.get(6)?,
                sort_order: row.get(7)?,
                lang: row.get(8)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(books)
}

/// 每日学习队列：到期复习优先、提前分散未来复习（削峰填谷），
/// 到期复习较多时动态减少今日新词，并按「复习-新词」交错排列（参考墨墨背单词）。
pub fn get_daily_queue(conn: &Connection, limit: Option<i64>) -> Result<DailyQueue, AppError> {
    let config = get_learning_config(conn)?;
    let now = now_secs();
    let today_start = now - (now % 86400);
    // 每日学习总量：先算当日到期复习数，总量减去复习数即为今日新学数（复习优先）。
    let total_limit = config.daily_total.max(0).min(300) as usize;
    let queue_limit = limit.unwrap_or(total_limit as i64).clamp(1, 500) as usize;
    let (scope, params) = scope_sql(conn)?;
    let due_count: i64 =
        scoped_count(conn, "s.due_at <= ?1 AND s.stage IN ('learning','reviewing')", &[&now])?;

    // 1) 复习配额 = 今日到期复习数，封顶每日总量（到期超量时当天先消化，余量顺延）。
    let review_limit = (due_count as usize).min(total_limit);

    // 2) 到期复习：先取已到期（含今天）的复习卡片。
    let due_cards: Vec<WordCard> = if review_limit > 0 {
        let sql = format!(
            "{WORD_CARD_SELECT} WHERE s.due_at <= ? AND s.stage IN ('learning','reviewing') {scope}
             ORDER BY s.due_at ASC, w.id ASC LIMIT ?"
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut all: Vec<&dyn rusqlite::ToSql> = vec![&now];
        all.extend(bind_scope(&params));
        let review_limit_i64 = review_limit as i64;
        all.push(&review_limit_i64);
        let rows = stmt.query_map(all.as_slice(), word_card_from_row)?
            .collect::<SqliteResult<Vec<_>>>()?;
        rows
    } else {
        Vec::new()
    };

    // 3) 削峰填谷：到期复习不足时，把未来 7 天内将到期的复习提前少量分散到今天
    //    （提前量不超过剩余额度的一半），避免某天集体到期导致复习扎堆，同时给新词留足空间。
    let mut review_cards = due_cards;
    if review_cards.len() < total_limit {
        let remaining = total_limit - review_cards.len();
        let preload_cap = remaining / 2;
        if preload_cap > 0 {
            let window_end = now + 7 * 86400;
            let sql = format!(
                "{WORD_CARD_SELECT} WHERE s.due_at > ? AND s.due_at <= ? AND s.stage IN ('learning','reviewing') {scope}
                 ORDER BY s.due_at ASC, w.id ASC LIMIT ?"
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut all: Vec<&dyn rusqlite::ToSql> = vec![&now, &window_end];
            all.extend(bind_scope(&params));
            let preload_cap_i64 = preload_cap as i64;
            all.push(&preload_cap_i64);
            let rows = stmt.query_map(all.as_slice(), word_card_from_row)?
                .collect::<SqliteResult<Vec<_>>>()?;
            review_cards.extend(rows);
        }
    }

    // 4) 新学数量 = 每日总量 - 实际复习数（复习超过总量时新学为 0）。
    let new_quota = total_limit.saturating_sub(review_cards.len());

    let new_cards: Vec<WordCard> = if new_quota > 0 {
        let sql = format!(
            "{WORD_CARD_SELECT} WHERE s.stage = 'new' {scope} ORDER BY RANDOM() LIMIT ?"
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut all: Vec<&dyn rusqlite::ToSql> = Vec::new();
        all.extend(bind_scope(&params));
        let new_quota_i64 = new_quota as i64;
        all.push(&new_quota_i64);
        let rows = stmt.query_map(all.as_slice(), word_card_from_row)?
            .collect::<SqliteResult<Vec<_>>>()?;
        rows
    } else {
        Vec::new()
    };

    // 5) 交错队列：复习与新词交替排列，首卡必为复习，避免新词堆在前面。
    let mut cards: Vec<WordCard> = Vec::with_capacity(queue_limit);
    let mut review_idx = 0usize;
    let mut new_idx = 0usize;
    let mut review_turn = true;
    while cards.len() < queue_limit {
        if review_turn && review_idx < review_cards.len() {
            cards.push(review_cards[review_idx].clone());
            review_idx += 1;
        } else if new_idx < new_cards.len() {
            cards.push(new_cards[new_idx].clone());
            new_idx += 1;
        } else if review_idx < review_cards.len() {
            cards.push(review_cards[review_idx].clone());
            review_idx += 1;
        } else {
            break;
        }
        review_turn = !review_turn;
    }
    let review_count = review_idx.min(queue_limit);
    let new_count = new_idx.min(queue_limit.saturating_sub(review_count));

    let total_done_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind = 'word_review' AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;

    Ok(DailyQueue {
        cards,
        new_count: new_count as i64,
        review_count: review_count as i64,
        total_done_today,
    })
}


/// 已选词书内全部单词（含难度与所属词书），用于自定义选词。
/// 无论选词模式如何，都按已勾选词书返回，方便用户在自定义模式下勾选单词。
pub fn get_wordbook_catalog(conn: &Connection) -> Result<Vec<WordbookCatalogEntry>, AppError> {
    let config = get_learning_config(conn)?;
    let (scope, params) = if config.book_codes.is_empty() {
        (String::new(), Vec::new())
    } else {
        let placeholders = vec!["?"; config.book_codes.len()].join(",");
        let sql = format!(
            "AND w.id IN (SELECT ww.word_id FROM wordbook_words ww JOIN wordbooks b ON b.id = ww.wordbook_id WHERE b.code IN ({placeholders}))"
        );
        (sql, config.book_codes.clone())
    };
    let sql = format!(
        "SELECT w.id, w.word, w.phonetic, w.meaning, w.difficulty, w.level
         FROM words w WHERE 1 = 1 {scope} ORDER BY w.id ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows: Vec<(i64, String, String, String, f64, String)> = stmt
        .query_map(bind_scope(&params).as_slice(), |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    let mut entries = Vec::with_capacity(rows.len());
    for (word_id, word, phonetic, meaning, difficulty, level) in rows {
        let mut book_stmt = conn.prepare(
            "SELECT b.code FROM wordbooks b JOIN wordbook_words ww ON ww.wordbook_id = b.id
             WHERE ww.word_id = ?1 ORDER BY b.sort_order ASC",
        )?;
        let book_codes = book_stmt
            .query_map([word_id], |row| row.get::<_, String>(0))?
            .collect::<SqliteResult<Vec<_>>>()?;
        entries.push(WordbookCatalogEntry {
            word_id,
            word,
            phonetic,
            meaning,
            difficulty,
            level,
            book_codes,
        });
    }
    Ok(entries)
}

pub fn get_word_catalog(conn: &Connection) -> Result<Vec<WordCard>, AppError> {
    let mut stmt = conn.prepare(&format!("{WORD_CARD_SELECT} ORDER BY w.id ASC"))?;
    let cards = stmt
        .query_map([], word_card_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(cards)
}

pub fn get_word_card(conn: &Connection, word_id: i64) -> Result<Option<WordCard>, AppError> {
    let mut stmt = conn.prepare(&format!("{WORD_CARD_SELECT} WHERE w.id = ?1"))?;
    let card = stmt
        .query_row(params![word_id], word_card_from_row)
        .optional()?;
    Ok(card)
}

pub fn get_overview(conn: &Connection) -> Result<Overview, AppError> {
    let now = now_secs();
    let today_start = now - (now % 86400);
    let total_words: i64 = scoped_count(conn, "1 = 1", &[])?;
    let due_words: i64 = scoped_count(conn, "s.due_at <= ?1 AND s.stage <> 'mastered'", &[&now])?;
    let learned_words: i64 = scoped_count(conn, "s.stage <> 'new'", &[])?;
    let mastered_words: i64 = scoped_count(conn, "s.stage = 'mastered'", &[])?;
    let total_reviews: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind IN ('word_review', 'sentence_event')",
        [],
        |row| row.get(0),
    )?;
    let today_reviews: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind IN ('word_review', 'sentence_event') AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let review_queue = get_review_queue(conn, Some(4))?;
    Ok(Overview {
        total_words,
        due_words,
        learned_words,
        mastered_words,
        total_reviews,
        today_reviews,
        review_queue,
    })
}

/// 按单词难度缩短复习间隔：有效间隔 = 基础间隔 ÷ 难度系数，
/// 难度越高（>1.0）巩固越频繁，至少 1 天。
fn effective_interval(base: i64, difficulty: f64) -> i64 {
    if difficulty <= 0.0 {
        return base;
    }
    ((base as f64) / difficulty).round().max(1.0) as i64
}

/// 重来后重新开始的认识间隔基准序列（平方递增）：
/// 第 n 次「认识」→ 1 + Σk²（1 → 2 → 6 → 15 → 31 → 56 → 92 → 141…），
/// 由调用方封顶 100 天并叠加随机扰动。
fn restart_interval_squared(reps: i64) -> i64 {
    if reps <= 1 {
        return 1;
    }
    let n = reps;
    1 + (n - 1) * n * (2 * n - 1) / 6
}

/// SplitMix64 混合函数：由 (word_id, 当天) 派生确定性伪随机数，
/// 让不同单词、不同天得到不同扰动，避免复习内容扎堆。
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// 给复习间隔加随机扰动（防扎堆）：
/// - first_known（首次「认识」）：基准间隔 ±10%；
/// - 常规：仅当间隔 > 5 天时按 ±20% 扰动，否则原样返回。
/// 种子 = (word_id, 当天)，同一张卡同一天内结果稳定、跨天变化。
fn jitter_interval(base: i64, word_id: i64, now: i64, first_known: bool) -> i64 {
    let span = if first_known {
        ((base as f64) * 0.10).round().max(1.0) as i64
    } else if base > 5 {
        ((base as f64) * 0.20).round().max(1.0) as i64
    } else {
        0
    };
    if span == 0 {
        return base;
    }
    let lo = (base - span).max(1);
    let hi = base + span;
    let range = (hi - lo + 1) as u64;
    let seed = (word_id as u64) ^ ((now / 86400) as u64).wrapping_shl(32);
    lo + (splitmix64(seed) % range) as i64
}


/// 当前记忆状态下，各档评分对应的复习间隔预测（用于学习页按钮下方提示）。
/// - `again_days` 恒为 0：重来后 10 分钟当次重学；
/// - `fuzzy_days` = 0 表示模糊后隔几个单词立刻重学，>0 为几天后复习；
/// - `known_days` = -1 表示认识后直接进入熟词本，>0 为几天后复习，0 为隔几个单词再见；
/// - `known_needs_again`：点击「认识」后是否处于恢复期第 1 次（还需再认识一次才算真记住）。
fn srs_preview_for(current: &SrsState, word_id: i64) -> ReviewPreview {
    let again = next_srs(current, 1, word_id);
    let fuzzy = next_srs(current, 2, word_id);
    let known = next_srs(current, 3, word_id);
    ReviewPreview {
        again_days: again.interval_days,
        fuzzy_days: fuzzy.interval_days,
        known_days: if known.stage == "mastered" {
            -1
        } else {
            known.interval_days
        },
        known_needs_again: current.lapses > 0
            && current.known_streak == 0
            && current.repetitions <= 1,
    }
}

fn next_srs(current: &SrsState, rating: i64, word_id: i64) -> SrsState {
    let now = now_secs();
    let mut next = SrsState {
        stage: current.stage.clone(),
        interval_days: current.interval_days,
        ease: current.ease,
        repetitions: current.repetitions,
        lapses: current.lapses,
        due_at: current.due_at,
        difficulty: current.difficulty,
        introduced_at: current.introduced_at,
        mastered_at: current.mastered_at,
        known_streak: current.known_streak,
    };
    let difficulty = if current.difficulty > 0.0 {
        current.difficulty
    } else {
        1.0
    };

    let mut mastered_now = false;
    match rating {
        1 => {
            // 重来：彻底忘记，从头再来，10 分钟后当次重学。
            next.known_streak = 0;
            next.repetitions = 0;
            next.lapses += 1;
            next.ease = (current.ease - 0.20).max(1.30);
            next.interval_days = 0;
            next.due_at = now + 600;
            next.mastered_at = None;
        }
        2 => {
            // 模糊：结合历史记忆情况决定立刻重学或短间隔复习；会打断「认识确认」连续计数。
            next.known_streak = 0;
            next.repetitions += 1;
            next.ease = (current.ease - 0.15).max(1.30);
            let shaky = current.lapses > 0 || current.ease <= 2.0 || current.difficulty >= 1.3;
            if shaky {
                // 记忆不稳定：隔几个单词立刻重学。
                next.interval_days = 0;
                next.due_at = now + 900;
            } else {
                let base = if next.repetitions == 1 {
                    // 首次「模糊」：隔几个单词（1 天内复习）。
                    1
                } else {
                    // 模糊：以当前间隔 × 0.8 做短间隔复习（当前间隔已含难度）。
                    ((current.interval_days as f64) * 0.8).round().max(1.0) as i64
                };
                // 短间隔同样受「>5 天加随机」与 100 天封顶约束。
                next.interval_days = jitter_interval(base, word_id, now, false).min(100);
                next.due_at = now + next.interval_days * 86400;
            }
            next.mastered_at = None;
        }
        3 => {
            // 认识：按计划推进复习间隔，封顶 100 天。
            next.repetitions += 1;
            if current.interval_days >= 100 {
                // 已达封顶间隔仍认识：直接进入熟词本，不再安排下一次复习。
                next.known_streak = 0;
                next.stage = "mastered".to_string();
                next.mastered_at = Some(now);
                next.interval_days = 0;
                next.due_at = i64::MAX;
                mastered_now = true;
            } else if current.lapses > 0 && current.known_streak == 0 && current.repetitions <= 1 {
                // 重来后第一次「认识」：只算“确认”，1 天后复习，还需再认识一次才算真记住。
                next.known_streak = 1;
                next.repetitions = 1;
                next.interval_days = 1;
                next.due_at = now + 86400;
                next.mastered_at = None;
            } else {
                next.known_streak = 0;
                let base = if current.known_streak == 1 {
                    // 恢复期第二次「认识」：确认完成，按平方序列推进（第 2 次 = 2 天）。
                    restart_interval_squared(next.repetitions)
                } else if current.lapses > 0 {
                    // 重来后已确认完成的后续认识：继续按平方序列自动规划（1 → 2 → 6 → 15 → 31 → 56 → 92 → 100 封顶）。
                    restart_interval_squared(next.repetitions)
                } else if next.repetitions == 1 {
                    // 真正首次「认识」：全新单词第一次答对，按难度给较长基准（难度越低间隔越长）。
                    ((14.0 / difficulty).round() as i64).max(7)
                } else {
                    // 全新词后续：以当前间隔 × ease 推进（当前间隔已含难度，不再重复除以难度）。
                    ((current.interval_days as f64) * current.ease).round().max(1.0) as i64
                };
                let interval = if next.repetitions == 1 && current.lapses == 0 {
                    // 真正首次「认识」：难度基准 + ±10% 随机划分。
                    jitter_interval(base, word_id, now, true)
                } else {
                    // 常规推进：间隔 > 5 天时 ±20% 随机（确认期 1 天固定、恢复期 2 天不加扰动）。
                    jitter_interval(base, word_id, now, false)
                };
                next.interval_days = interval.max(1).min(100);
                next.due_at = now + next.interval_days * 86400;
                next.mastered_at = None;
            }
        }
        4 => {
            // 熟悉：加入熟词本，永不安排复习。
            next.known_streak = 0;
            next.repetitions += 1;
            next.stage = "mastered".to_string();
            next.mastered_at = Some(now);
            next.interval_days = 0;
            next.due_at = i64::MAX;
            mastered_now = true;
            // 熟悉不调整 ease，保持既有节奏稳定。
        }
        _ => {}
    }

    if !mastered_now {
        // 阶段推进：「熟悉」与封顶间隔后仍「认识」已进入熟词本，其余按节奏推进。
        if next.interval_days > 3 && next.repetitions >= 3 {
            next.stage = "reviewing".to_string();
        } else if next.repetitions > 0 {
            next.stage = "learning".to_string();
        } else {
            next.stage = "new".to_string();
        }
    }

    next
}


pub fn answer_word(
    conn: &mut Connection,
    word_id: i64,
    rating: i64,
    duration_ms: i64,
) -> Result<WordCard, AppError> {
    if !(1..=4).contains(&rating) {
        return Err(AppError::NotFound("复习评分必须是 1–4。".to_string()));
    }
    let now = now_secs();
    let tx = conn.transaction()?;
    let current: Option<SrsState> = tx
        .query_row(
            "SELECT stage, interval_days, ease, repetitions, lapses, due_at, difficulty, introduced_at, mastered_at, known_streak
             FROM word_states WHERE word_id = ?1",
            [word_id],
            |row| {
                Ok(SrsState {
                    stage: row.get(0)?,
                    interval_days: row.get(1)?,
                    ease: row.get(2)?,
                    repetitions: row.get(3)?,
                    lapses: row.get(4)?,
                    due_at: row.get(5)?,
                    difficulty: row.get(6)?,
                    introduced_at: row.get(7)?,
                    mastered_at: row.get(8)?,
                    known_streak: row.get(9)?,
                })
            },
        )
        .optional()?;
    let current = current.ok_or_else(|| AppError::NotFound("没有找到这个单词的复习状态。".to_string()))?;

    let next = next_srs(&current, rating, word_id);
    tx.execute(
        "UPDATE word_states
         SET stage = ?1, interval_days = ?2, ease = ?3, repetitions = ?4,
             lapses = ?5, due_at = ?6, last_reviewed_at = ?7,
             introduced_at = COALESCE(introduced_at, ?9),
             mastered_at = ?10,
             known_streak = ?11
         WHERE word_id = ?8",
        params![
            next.stage,
            next.interval_days,
            next.ease,
            next.repetitions,
            next.lapses,
            next.due_at,
            now,
            word_id,
            now,
            next.mastered_at,
            next.known_streak
        ],
    )?;
    let result = if rating >= 3 { 1 } else { 0 };
    tx.execute(
        "INSERT INTO study_events (kind, ref_id, result, rating, duration_ms, created_at)
         VALUES ('word_review', ?1, ?2, ?3, ?4, ?5)",
        params![word_id, result, rating, duration_ms, now],
    )?;
    tx.commit()?;

    get_word_card(conn, word_id)?
        .ok_or_else(|| AppError::NotFound("复习记录已保存，但读取结果失败。".to_string()))
}

fn sentence_card_from_row(row: &rusqlite::Row<'_>) -> SqliteResult<SentenceCard> {
    Ok(SentenceCard {
        id: row.get(0)?,
        en: row.get(1)?,
        zh: row.get(2)?,
        status: row.get(3)?,
        attempts: row.get(4)?,
        correct_count: row.get(5)?,
    })
}

const SENTENCE_CARD_SELECT: &str = "
SELECT s.id, s.en, s.zh, st.status, st.attempts, st.correct_count
FROM sentences s
JOIN sentence_states st ON st.sentence_id = s.id
";

pub fn get_sentences(conn: &Connection, limit: Option<i64>) -> Result<Vec<SentenceCard>, AppError> {
    let limit = limit.unwrap_or(50).clamp(1, 500);
    let mut stmt = conn.prepare(&format!(
        "{SENTENCE_CARD_SELECT} ORDER BY s.id ASC LIMIT ?1"
    ))?;
    let cards = stmt
        .query_map(params![limit], sentence_card_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(cards)
}

pub fn get_sentence_card(
    conn: &Connection,
    sentence_id: i64,
) -> Result<Option<SentenceCard>, AppError> {
    let mut stmt = conn.prepare(&format!("{SENTENCE_CARD_SELECT} WHERE s.id = ?1"))?;
    let card = stmt
        .query_row(params![sentence_id], sentence_card_from_row)
        .optional()?;
    Ok(card)
}

pub fn complete_sentence(
    conn: &mut Connection,
    sentence_id: i64,
    correct: bool,
    duration_ms: i64,
) -> Result<SentenceCard, AppError> {
    let now = now_secs();
    let result = if correct { 1 } else { 0 };
    let tx = conn.transaction()?;
    let exists: Option<i64> = tx
        .query_row("SELECT 1 FROM sentences WHERE id = ?1", [sentence_id], |row| {
            row.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound("没有找到这个句子。".to_string()));
    }
    tx.execute(
        "UPDATE sentence_states
         SET attempts = attempts + 1,
             correct_count = correct_count + ?1,
             last_result = ?1,
             last_attempted_at = ?2,
             status = CASE
                WHEN correct_count + ?1 >= 3 THEN 'mastered'
                WHEN attempts + 1 > 0 THEN 'learning'
                ELSE 'new'
             END
         WHERE sentence_id = ?3",
        params![result, now, sentence_id],
    )?;
    tx.execute(
        "INSERT INTO study_events (kind, ref_id, result, rating, duration_ms, created_at)
         VALUES ('sentence_event', ?1, ?2, NULL, ?3, ?4)",
        params![sentence_id, result, duration_ms, now],
    )?;
    tx.commit()?;

    get_sentence_card(conn, sentence_id)?
        .ok_or_else(|| AppError::NotFound("句子记录已保存，但读取结果失败。".to_string()))
}

pub fn get_vocabulary(conn: &Connection) -> Result<Vec<VocabularyEntry>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT v.word_id, w.word, w.phonetic, w.meaning, v.added_at
         FROM vocabulary v
         JOIN words w ON w.id = v.word_id
         ORDER BY v.added_at DESC, w.word ASC",
    )?;
    let entries = stmt
        .query_map([], |row| {
            Ok(VocabularyEntry {
                word_id: row.get(0)?,
                word: row.get(1)?,
                phonetic: row.get(2)?,
                meaning: row.get(3)?,
                added_at: row.get(4)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(entries)
}

pub fn add_to_vocabulary(conn: &Connection, word_id: i64) -> Result<(), AppError> {
    let exists: Option<i64> = conn
        .query_row("SELECT 1 FROM words WHERE id = ?1", [word_id], |row| {
            row.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound("没有找到这个单词。".to_string()));
    }
    conn.execute(
        "INSERT OR IGNORE INTO vocabulary (word_id, added_at) VALUES (?1, ?2)",
        params![word_id, now_secs()],
    )?;
    Ok(())
}

pub fn remove_from_vocabulary(conn: &Connection, word_id: i64) -> Result<(), AppError> {
    conn.execute("DELETE FROM vocabulary WHERE word_id = ?1", [word_id])?;
    Ok(())
}

/// 熟词本：所有标记为「熟悉」的单词（永不进入复习队列）。
pub fn get_mastered_words(conn: &Connection) -> Result<Vec<MasteredWord>, AppError> {
    let (scope, params) = scope_sql(conn)?;
    let sql = format!(
        "SELECT s.word_id, w.word, w.phonetic, w.meaning, s.difficulty, s.mastered_at
         FROM word_states s
         JOIN words w ON w.id = s.word_id
         WHERE s.stage = 'mastered' {scope}
         ORDER BY s.mastered_at DESC, w.word ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(bind_scope(&params).as_slice(), |row| {
            Ok(MasteredWord {
                word_id: row.get(0)?,
                word: row.get(1)?,
                phonetic: row.get(2)?,
                meaning: row.get(3)?,
                difficulty: row.get(4)?,
                mastered_at: row.get(5)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(rows)
}

/// 把熟词本中的单词移回复习：按当前时间重新排队（立即可复习）。
pub fn unmaster_word(conn: &Connection, word_id: i64) -> Result<(), AppError> {
    let now = now_secs();
    let affected = conn.execute(
        "UPDATE word_states
         SET stage = 'learning', interval_days = 0, due_at = ?1, mastered_at = NULL
         WHERE word_id = ?2 AND stage = 'mastered'",
        params![now, word_id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound("这个单词不在熟词本中。".to_string()));
    }
    Ok(())
}

/// 删除指定单词的学习记录：清空其复习事件、移出生词本/熟词本，
/// 并把复习状态重置为未学习（新词）。单词本身保留在词库中。
pub fn delete_learning_records(conn: &mut Connection, word_id: i64) -> Result<(), AppError> {
    let now = now_secs();
    let tx = conn.transaction()?;
    let exists: Option<i64> = tx
        .query_row("SELECT 1 FROM words WHERE id = ?1", [word_id], |row| {
            row.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound("没有找到这个单词。".to_string()));
    }
    // 难度恢复为词书基准（与 clear_learning_data 一致），保证“未学习”的初始状态。
    let difficulty: f64 = tx.query_row(
        "SELECT COALESCE(
            (SELECT MIN(MAX(b.difficulty * w.difficulty, 0.5), 2.0)
             FROM wordbook_words ww JOIN wordbooks b ON b.id = ww.wordbook_id
             JOIN words w ON w.id = ww.word_id
             WHERE ww.word_id = ?1 ORDER BY b.sort_order ASC LIMIT 1),
            w.difficulty)
         FROM words w WHERE w.id = ?1",
        [word_id],
        |row| row.get(0),
    )?;
    tx.execute(
        "DELETE FROM study_events WHERE kind = 'word_review' AND ref_id = ?1",
        [word_id],
    )?;
    tx.execute("DELETE FROM vocabulary WHERE word_id = ?1", [word_id])?;
    tx.execute(
        "UPDATE word_states
         SET stage = 'new', interval_days = 0, ease = 2.5, repetitions = 0,
             lapses = 0, due_at = ?1, last_reviewed_at = NULL,
             mastered_at = NULL, known_streak = 0, introduced_at = NULL,
             difficulty = ?2
         WHERE word_id = ?3",
        params![now, difficulty, word_id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn get_stats(conn: &Connection) -> Result<StudyStats, AppError> {
    let now = now_secs();
    let today_start = now - (now % 86400);
    let total_reviews: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind IN ('word_review', 'sentence_event')",
        [],
        |row| row.get(0),
    )?;
    let correct_reviews: i64 = conn.query_row(
        "SELECT COALESCE(SUM(result), 0) FROM study_events WHERE kind IN ('word_review', 'sentence_event')",
        [],
        |row| row.get(0),
    )?;
    let total_duration_ms: i64 = conn.query_row(
        "SELECT COALESCE(SUM(duration_ms), 0) FROM study_events WHERE kind IN ('word_review', 'sentence_event')",
        [],
        |row| row.get(0),
    )?;
    let today_reviews: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind IN ('word_review', 'sentence_event') AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let today_duration_ms: i64 = conn.query_row(
        "SELECT COALESCE(SUM(duration_ms), 0) FROM study_events WHERE kind IN ('word_review', 'sentence_event') AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let due_words: i64 = scoped_count(conn, "s.due_at <= ?1 AND s.stage <> 'mastered'", &[&now])?;
    let mastered_words: i64 = scoped_count(conn, "s.stage = 'mastered'", &[])?;
    let vocabulary_count: i64 = conn.query_row("SELECT COUNT(*) FROM vocabulary", [], |row| {
        row.get(0)
    })?;

    Ok(StudyStats {
        total_reviews,
        correct_reviews,
        accuracy_percent: if total_reviews > 0 {
            (correct_reviews * 100 / total_reviews).max(0)
        } else {
            0
        },
        total_duration_min: total_duration_ms / 60000,
        today_reviews,
        today_duration_min: today_duration_ms / 60000,
        due_words,
        mastered_words,
        vocabulary_count,
    })
}

pub fn export_data(conn: &Connection, data_dir: &Path) -> Result<ExportResult, AppError> {
    let exports_dir = data_dir.join("exports");
    std::fs::create_dir_all(&exports_dir)?;
    let path = exports_dir.join(format!("janus-export-{}.json", now_secs()));
    let words = get_word_catalog(conn)?;
    let sentences = get_sentences(conn, None)?;
    let mut stmt = conn.prepare(
        "SELECT id, kind, ref_id, result, rating, duration_ms, created_at
         FROM study_events ORDER BY id ASC",
    )?;
    let events = stmt
        .query_map([], |row| {
            Ok(EventRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                ref_id: row.get(2)?,
                result: row.get(3)?,
                rating: row.get(4)?,
                duration_ms: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    let payload = ExportPayload {
        exported_at: now_secs(),
        schema_version: SCHEMA_VERSION,
        words,
        sentences,
        events,
    };
    let json = serde_json::to_string_pretty(&payload)?;
    std::fs::write(&path, json)?;
    Ok(ExportResult {
        path: path.to_string_lossy().to_string(),
    })
}

pub fn clear_learning_data(conn: &mut Connection, data_dir: &Path) -> Result<ClearResult, AppError> {
    let backups_dir = data_dir.join("backups");
    std::fs::create_dir_all(&backups_dir)?;
    let backup_path = backups_dir.join(format!("janus-backup-{}.db", now_secs()));

    {
        let mut dest = Connection::open(&backup_path)?;
        let backup = rusqlite::backup::Backup::new(conn, &mut dest)?;
        backup.run_to_completion(5, Duration::from_millis(50), None)?;
    }

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM study_events", [])?;
    tx.execute("DELETE FROM dictation_attempts", [])?;
    tx.execute("DELETE FROM vocabulary", [])?;
    tx.execute("DELETE FROM word_states", [])?;
    tx.execute("DELETE FROM sentence_states", [])?;
    tx.execute("DELETE FROM conversation_sessions", [])?;
    let now = now_secs();
    tx.execute(
        "INSERT INTO word_states (word_id, stage, interval_days, ease, repetitions, lapses, due_at, created_at, difficulty, introduced_at)
         SELECT w.id, 'new', 0, 2.5, 0, 0, ?1, ?1,
                COALESCE(
                    (SELECT MIN(MAX(b.difficulty * w.difficulty, 0.5), 2.0)
                     FROM wordbook_words ww JOIN wordbooks b ON b.id = ww.wordbook_id
                     WHERE ww.word_id = w.id ORDER BY b.sort_order ASC LIMIT 1),
                    w.difficulty
                ),
                NULL FROM words w",
        [now],
    )?;
    tx.execute(
        "INSERT INTO sentence_states (sentence_id, status, attempts, correct_count, last_result, last_attempted_at, created_at)
         SELECT id, 'new', 0, 0, NULL, NULL, ?1 FROM sentences",
        [now],
    )?;
    tx.commit()?;

    Ok(ClearResult {
        backup_path: backup_path.to_string_lossy().to_string(),
    })
}


fn normalize_dictation_text(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_whitespace() {
                ' '
            } else if c.is_alphanumeric() || c == '\'' || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Insert the 20 bundled seed clips (and their single segments) into the
/// listening tables. Idempotent: uses stable ids 1..=20 and skips clips the
/// user has deleted (tombstoned in deleted_seed_clips).
pub fn ensure_seed_listening_clips(conn: &mut Connection) -> Result<(), AppError> {
    let now = now_secs();
    let tx = conn.transaction()?;
    for (index, clip) in SEED_CLIPS.iter().enumerate() {
        let seed_index = (index + 1) as i64;
        let tombstoned: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM deleted_seed_clips WHERE seed_index = ?1",
                [seed_index],
                |row| row.get(0),
            )
            .optional()?;
        if tombstoned.is_some() {
            continue;
        }
        let file_name = format!("seed-{seed_index:02}.wav");
        let duration_ms = wav_duration_ms(clip.wav);
        tx.execute(
            "INSERT OR IGNORE INTO listening_clips
             (id, title, lang, source, file_name, duration_ms, segment_count, created_at)
             VALUES (?1, ?2, 'en', 'seed', ?3, ?4, 1, ?5)",
            params![seed_index, clip.title, file_name, duration_ms, now],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO listening_segments
             (id, clip_id, seq, text_en, text_zh, start_ms, end_ms)
             VALUES (?1, ?1, 1, ?2, ?3, 0, ?4)",
            params![seed_index, clip.title, clip.text_zh, duration_ms],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn get_audio_catalog(
    conn: &Connection,
    media_dir: &Path,
) -> Result<Vec<AudioCatalogEntry>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.source, c.file_name, c.duration_ms, c.segment_count,
                COALESCE((SELECT seg.text_zh FROM listening_segments seg WHERE seg.clip_id = c.id LIMIT 1), ''),
                (SELECT COUNT(*) FROM dictation_attempts d WHERE d.clip_id = c.id),
                (SELECT COUNT(*) FROM dictation_attempts d WHERE d.clip_id = c.id AND d.correct = 1)
         FROM listening_clips c
         ORDER BY c.id ASC",
    )?;
    let entries = stmt
        .query_map([], |row| {
            let file_name: String = row.get(3)?;
            let file_path = media_dir.join(&file_name);
            Ok(AudioCatalogEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                source: row.get(2)?,
                file_name,
                file_path: file_path.to_string_lossy().to_string(),
                duration_ms: row.get(4)?,
                segment_count: row.get(5)?,
                text_zh: row.get(6)?,
                dictation_attempts: row.get(7)?,
                dictation_correct: row.get(8)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(entries)
}

pub fn get_listening_detail(conn: &Connection, clip_id: i64) -> Result<ListeningDetail, AppError> {
    let clip = conn
        .query_row(
            "SELECT id, title, lang, source, file_name, duration_ms, segment_count, created_at
             FROM listening_clips WHERE id = ?1",
            [clip_id],
            |row| {
                Ok(ListeningClip {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    lang: row.get(2)?,
                    source: row.get(3)?,
                    file_name: row.get(4)?,
                    duration_ms: row.get(5)?,
                    segment_count: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("没有找到这个听力片段。".to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT id, clip_id, seq, text_en, text_zh, start_ms, end_ms
         FROM listening_segments WHERE clip_id = ?1 ORDER BY seq ASC",
    )?;
    let segments = stmt
        .query_map(params![clip_id], |row| {
            Ok(ListeningSegment {
                id: row.get(0)?,
                clip_id: row.get(1)?,
                seq: row.get(2)?,
                text_en: row.get(3)?,
                text_zh: row.get(4)?,
                start_ms: row.get(5)?,
                end_ms: row.get(6)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(ListeningDetail { clip, segments })
}

/// Record one dictation attempt. Correctness is judged by the backend against
/// the stored segment text (the client never declares its own score).
pub fn record_dictation(
    conn: &mut Connection,
    clip_id: i64,
    segment_id: i64,
    submitted: String,
    duration_ms: i64,
) -> Result<DictationResult, AppError> {
    let expected = conn
        .query_row(
            "SELECT text_en FROM listening_segments WHERE id = ?1 AND clip_id = ?2",
            params![segment_id, clip_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("没有找到这个听写片段。".to_string()))?;
    let correct = normalize_dictation_text(&submitted) == normalize_dictation_text(&expected);
    let now = now_secs();
    conn.execute(
        "INSERT INTO dictation_attempts (clip_id, segment_id, expected, submitted, correct, duration_ms, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![clip_id, segment_id, expected, submitted, correct, duration_ms, now],
    )?;
    Ok(DictationResult {
        id: conn.last_insert_rowid(),
        clip_id,
        segment_id,
        expected,
        submitted,
        correct,
        duration_ms,
        created_at: now,
    })
}

pub fn get_dictation_stats(conn: &Connection) -> Result<DictationStats, AppError> {
    let now = now_secs();
    let today_start = now - (now % 86400);
    let total_attempts: i64 =
        conn.query_row("SELECT COUNT(*) FROM dictation_attempts", [], |row| row.get(0))?;
    let correct_attempts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dictation_attempts WHERE correct = 1",
        [],
        |row| row.get(0),
    )?;
    let today_attempts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dictation_attempts WHERE created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let today_correct: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dictation_attempts WHERE correct = 1 AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, COUNT(d.id) AS attempts, COALESCE(SUM(d.correct), 0) AS correct
         FROM listening_clips c
         LEFT JOIN dictation_attempts d ON d.clip_id = c.id
         GROUP BY c.id
         HAVING attempts > 0
         ORDER BY (CAST(correct AS REAL) / attempts) ASC, attempts DESC
         LIMIT 5",
    )?;
    let weakest_clips = stmt
        .query_map([], |row| {
            let attempts: i64 = row.get(2)?;
            let correct: i64 = row.get(3)?;
            Ok(WeakClip {
                clip_id: row.get(0)?,
                title: row.get(1)?,
                attempts,
                correct,
                accuracy_percent: if attempts > 0 {
                    correct * 100 / attempts
                } else {
                    0
                },
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(DictationStats {
        total_attempts,
        correct_attempts,
        accuracy_percent: if total_attempts > 0 {
            correct_attempts * 100 / total_attempts
        } else {
            0
        },
        today_attempts,
        today_correct,
        weakest_clips,
    })
}

/// Delete a listening clip (cascades segments and dictation attempts). Seed
/// clips are tombstoned so the next startup does not resurrect them.
pub fn delete_seed_audio(conn: &mut Connection, clip_id: i64) -> Result<DeleteResult, AppError> {
    let clip = conn
        .query_row(
            "SELECT source, file_name FROM listening_clips WHERE id = ?1",
            [clip_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("没有找到这个听力片段。".to_string()))?;
    let (source, file_name) = clip;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM listening_clips WHERE id = ?1", [clip_id])?;
    if source == "seed" {
        let seed_index = seed_index_from_file_name(&file_name);
        if let Some(index) = seed_index {
            tx.execute(
                "INSERT OR IGNORE INTO deleted_seed_clips (seed_index, deleted_at) VALUES (?1, ?2)",
                params![index, now_secs()],
            )?;
        }
    }
    tx.commit()?;
    Ok(DeleteResult { file_name })
}

fn seed_index_from_file_name(file_name: &str) -> Option<i64> {
    let prefix = "seed-";
    let suffix = ".wav";
    if file_name.starts_with(prefix) && file_name.ends_with(suffix) {
        file_name[prefix.len()..file_name.len() - suffix.len()]
            .parse::<i64>()
            .ok()
    } else {
        None
    }
}

/// Normalize spoken text for comparison: lowercase, keep alphanumeric
/// characters, collapse whitespace and drop punctuation so "It depends."
/// compares equal to "it depends".
fn normalize_speaking_text(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                normalized.push(lower);
            }
        } else if ch.is_whitespace() {
            if !normalized.ends_with(' ') {
                normalized.push(' ');
            }
        }
    }
    normalized.trim_end().to_string()
}

fn tokenize_speaking(text: &str) -> Vec<String> {
    normalize_speaking_text(text)
        .split_whitespace()
        .map(|token| token.to_string())
        .collect()
}

/// Word-level alignment between expected and transcribed tokens.
///
/// Uses dynamic programming over tokens (LCS with substitutions); backtracking
/// produces per-word feedback for the UI:
/// - "correct": expected word matched verbatim;
/// - "missing": expected word was not spoken;
/// - "changed": expected word was spoken differently;
/// - "extra": an unexpected word was spoken.
fn align_words(expected: &[String], transcribed: &[String]) -> Vec<WordFeedback> {
    let n = expected.len();
    let m = transcribed.len();
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in 1..=n {
        for j in 1..=m {
            let hit = usize::from(expected[i - 1] == transcribed[j - 1]);
            dp[i][j] = dp[i - 1][j]
                .max(dp[i][j - 1])
                .max(dp[i - 1][j - 1] + hit);
        }
    }

    enum Kind {
        Correct,
        Missing,
        Changed,
        Extra,
    }

    let mut events = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        let hit = expected[i - 1] == transcribed[j - 1];
        if hit && dp[i][j] == dp[i - 1][j - 1] + 1 {
            events.push((Kind::Correct, expected[i - 1].clone()));
            i -= 1;
            j -= 1;
        } else if dp[i][j] == dp[i - 1][j - 1] {
            // Tie-breaking: a substitution reads more clearly to learners
            // than an "extra + missing" pair when both achieve the same score.
            events.push((Kind::Changed, expected[i - 1].clone()));
            i -= 1;
            j -= 1;
        } else if dp[i][j] == dp[i - 1][j] {
            events.push((Kind::Missing, expected[i - 1].clone()));
            i -= 1;
        } else if dp[i][j] == dp[i][j - 1] {
            events.push((Kind::Extra, transcribed[j - 1].clone()));
            j -= 1;
        } else {
            events.push((Kind::Changed, expected[i - 1].clone()));
            i -= 1;
            j -= 1;
        }
    }
    while i > 0 {
        events.push((Kind::Missing, expected[i - 1].clone()));
        i -= 1;
    }
    while j > 0 {
        events.push((Kind::Extra, transcribed[j - 1].clone()));
        j -= 1;
    }
    events.reverse();

    events
        .into_iter()
        .map(|(kind, word)| WordFeedback {
            status: match kind {
                Kind::Correct => "correct",
                Kind::Missing => "missing",
                Kind::Changed => "changed",
                Kind::Extra => "extra",
            }
            .to_string(),
            word,
        })
        .collect()
}

/// Record one speaking attempt. The backend aligns the ASR transcription
/// against the stored segment text, computes word-level feedback and stores
/// only text + metrics (raw recordings are never persisted).
pub fn record_speaking_attempt(
    conn: &mut Connection,
    clip_id: i64,
    segment_id: i64,
    transcribed: String,
    confidence: f64,
    low_confidence: bool,
    duration_ms: i64,
) -> Result<SpeakingResult, AppError> {
    let expected = conn
        .query_row(
            "SELECT text_en FROM listening_segments WHERE id = ?1 AND clip_id = ?2",
            params![segment_id, clip_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound("没有找到这个朗读片段。".to_string()))?;
    let expected_tokens = tokenize_speaking(&expected);
    let transcribed_tokens = tokenize_speaking(&transcribed);
    let words = align_words(&expected_tokens, &transcribed_tokens);
    let word_total = expected_tokens.len() as i64;
    let word_matches = words
        .iter()
        .filter(|word| word.status == "correct")
        .count() as i64;
    let accuracy_percent = if word_total > 0 {
        word_matches * 100 / word_total
    } else {
        0
    };
    let now = now_secs();
    conn.execute(
        "INSERT INTO speaking_attempts (clip_id, segment_id, expected, transcribed, word_matches, word_total, accuracy_percent, confidence, low_confidence, duration_ms, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            clip_id,
            segment_id,
            expected,
            transcribed,
            word_matches,
            word_total,
            accuracy_percent,
            confidence,
            low_confidence as i64,
            duration_ms,
            now
        ],
    )?;
    Ok(SpeakingResult {
        id: conn.last_insert_rowid(),
        clip_id,
        segment_id,
        expected,
        transcribed,
        word_matches,
        word_total,
        accuracy_percent,
        confidence,
        low_confidence,
        duration_ms,
        created_at: now,
        words,
    })
}

pub fn get_speaking_stats(conn: &Connection) -> Result<SpeakingStats, AppError> {
    let now = now_secs();
    let today_start = now - (now % 86400);
    let total_attempts: i64 =
        conn.query_row("SELECT COUNT(*) FROM speaking_attempts", [], |row| row.get(0))?;
    let today_attempts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM speaking_attempts WHERE created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let total_matches: i64 = conn.query_row(
        "SELECT COALESCE(SUM(word_matches), 0) FROM speaking_attempts",
        [],
        |row| row.get(0),
    )?;
    let total_words: i64 = conn.query_row(
        "SELECT COALESCE(SUM(word_total), 0) FROM speaking_attempts",
        [],
        |row| row.get(0),
    )?;
    let low_confidence_attempts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM speaking_attempts WHERE low_confidence = 1",
        [],
        |row| row.get(0),
    )?;
    let total_duration_ms: i64 = conn.query_row(
        "SELECT COALESCE(SUM(duration_ms), 0) FROM speaking_attempts",
        [],
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT s.id, s.clip_id, c.title, COUNT(a.id) AS attempts, COALESCE(SUM(a.word_matches), 0) AS matches, COALESCE(SUM(a.word_total), 0) AS words
         FROM listening_segments s
         JOIN listening_clips c ON c.id = s.clip_id
         LEFT JOIN speaking_attempts a ON a.segment_id = s.id
         GROUP BY s.id
         HAVING attempts > 0
         ORDER BY (CASE WHEN words > 0 THEN CAST(matches AS REAL) / words ELSE 1 END) ASC, attempts DESC
         LIMIT 5",
    )?;
    let weakest_segments = stmt
        .query_map([], |row| {
            let attempts: i64 = row.get(3)?;
            let matches: i64 = row.get(4)?;
            let words: i64 = row.get(5)?;
            Ok(SpeakingWeakSegment {
                segment_id: row.get(0)?,
                clip_id: row.get(1)?,
                title: row.get(2)?,
                attempts,
                accuracy_percent: if words > 0 {
                    matches * 100 / words
                } else {
                    0
                },
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(SpeakingStats {
        total_attempts,
        today_attempts,
        accuracy_percent: if total_words > 0 {
            total_matches * 100 / total_words
        } else {
            0
        },
        low_confidence_attempts,
        total_duration_ms,
        weakest_segments,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationScene {
    pub id: i64,
    pub title: String,
    pub goal: String,
    pub role: String,
    pub user_goal: String,
    pub target_words: String,
    pub level: String,
    pub greeting: String,
    pub lang: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationMessage {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String,
    pub correction: Option<String>,
    pub latency_ms: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationSessionSummary {
    pub id: i64,
    pub scene_id: i64,
    pub scene_title: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub turn_count: i64,
    pub summary: String,
    pub message_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationStartResult {
    pub session: ConversationSessionSummary,
    pub greeting: ConversationMessage,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationTurnResult {
    pub user_message: ConversationMessage,
    pub assistant_message: ConversationMessage,
    pub turn_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationStats {
    pub total_sessions: i64,
    pub total_turns: i64,
    pub today_sessions: i64,
    pub today_turns: i64,
    pub recent_scenes: Vec<SceneUsage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneUsage {
    pub scene_id: i64,
    pub title: String,
    pub sessions: i64,
}

/// One learner-facing correction item, produced by the LLM correction pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub original: String,
    pub suggestion: String,
    pub note: String,
}

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";

pub fn get_conversation_settings(conn: &Connection) -> Result<ConversationSettings, AppError> {
    let mut stmt = conn.prepare("SELECT key, value FROM conversation_settings")?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .collect::<SqliteResult<Vec<_>>>()?;
    let mut settings = ConversationSettings {
        base_url: DEFAULT_BASE_URL.to_string(),
        api_key: String::new(),
        model: DEFAULT_MODEL.to_string(),
    };
    for (key, value) in rows {
        match key.as_str() {
            "base_url" => settings.base_url = value,
            "api_key" => settings.api_key = value,
            "model" => settings.model = value,
            _ => {}
        }
    }
    Ok(settings)
}

pub fn save_conversation_settings(
    conn: &mut Connection,
    settings: &ConversationSettings,
) -> Result<(), AppError> {
    let tx = conn.transaction()?;
    for (key, value) in [
        ("base_url", settings.base_url.trim()),
        ("api_key", settings.api_key.trim()),
        ("model", settings.model.trim()),
    ] {
        tx.execute(
            "INSERT INTO conversation_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn get_conversation_scenes(conn: &Connection) -> Result<Vec<ConversationScene>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, title, goal, role, user_goal, target_words, level, greeting, lang
         FROM conversation_scenes WHERE enabled = 1 ORDER BY id",
    )?;
    let scenes = stmt
        .query_map([], scene_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(scenes)
}

fn scene_from_row(row: &rusqlite::Row<'_>) -> SqliteResult<ConversationScene> {
    Ok(ConversationScene {
        id: row.get(0)?,
        title: row.get(1)?,
        goal: row.get(2)?,
        role: row.get(3)?,
        user_goal: row.get(4)?,
        target_words: row.get(5)?,
        level: row.get(6)?,
        greeting: row.get(7)?,
        lang: row.get(8)?,
    })
}

pub fn get_conversation_scene(conn: &Connection, scene_id: i64) -> Result<ConversationScene, AppError> {
    conn.query_row(
        "SELECT id, title, goal, role, user_goal, target_words, level, greeting, lang
         FROM conversation_scenes WHERE id = ?1",
        [scene_id],
        scene_from_row,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound("对话场景不存在，请刷新后重试。".to_string())
        }
        other => AppError::Sqlite(other),
    })
}

pub fn get_conversation_scene_by_session(
    conn: &Connection,
    session_id: i64,
) -> Result<ConversationScene, AppError> {
    conn.query_row(
        "SELECT sc.id, sc.title, sc.goal, sc.role, sc.user_goal, sc.target_words, sc.level, sc.greeting, sc.lang
         FROM conversation_sessions s
         JOIN conversation_scenes sc ON sc.id = s.scene_id
         WHERE s.id = ?1",
        [session_id],
        scene_from_row,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound("对话会话不存在，请刷新后重试。".to_string())
        }
        other => AppError::Sqlite(other),
    })
}

pub fn start_conversation_session(
    conn: &Connection,
    scene_id: i64,
) -> Result<ConversationStartResult, AppError> {
    let scene = get_conversation_scene(conn, scene_id)?;
    let now = now_secs();
    conn.execute(
        "INSERT INTO conversation_sessions (scene_id, started_at, ended_at, turn_count, summary, created_at)
         VALUES (?1, ?2, NULL, 0, '', ?2)",
        params![scene_id, now],
    )?;
    let session_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO conversation_messages (session_id, role, content, correction, latency_ms, created_at)
         VALUES (?1, 'assistant', ?2, NULL, 0, ?3)",
        params![session_id, scene.greeting, now],
    )?;
    let greeting = ConversationMessage {
        id: conn.last_insert_rowid(),
        session_id,
        role: "assistant".to_string(),
        content: scene.greeting,
        correction: None,
        latency_ms: 0,
        created_at: now,
    };
    let session = get_conversation_session(conn, session_id)?;
    Ok(ConversationStartResult { session, greeting })
}

pub fn get_conversation_session(
    conn: &Connection,
    session_id: i64,
) -> Result<ConversationSessionSummary, AppError> {
    conn.query_row(
        "SELECT s.id, s.scene_id, sc.title, s.started_at, s.ended_at, s.turn_count, s.summary,
                (SELECT COUNT(*) FROM conversation_messages m WHERE m.session_id = s.id) AS message_count
         FROM conversation_sessions s
         JOIN conversation_scenes sc ON sc.id = s.scene_id
         WHERE s.id = ?1",
        [session_id],
        session_summary_from_row,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound("对话会话不存在，请刷新后重试。".to_string())
        }
        other => AppError::Sqlite(other),
    })
}

fn session_summary_from_row(row: &rusqlite::Row<'_>) -> SqliteResult<ConversationSessionSummary> {
    Ok(ConversationSessionSummary {
        id: row.get(0)?,
        scene_id: row.get(1)?,
        scene_title: row.get(2)?,
        started_at: row.get(3)?,
        ended_at: row.get(4)?,
        turn_count: row.get(5)?,
        summary: row.get(6)?,
        message_count: row.get(7)?,
    })
}

pub fn get_conversation_sessions(
    conn: &Connection,
    limit: Option<i64>,
) -> Result<Vec<ConversationSessionSummary>, AppError> {
    let limit = limit.unwrap_or(20).max(1).min(200);
    let mut stmt = conn.prepare(
        "SELECT s.id, s.scene_id, sc.title, s.started_at, s.ended_at, s.turn_count, s.summary,
                (SELECT COUNT(*) FROM conversation_messages m WHERE m.session_id = s.id) AS message_count
         FROM conversation_sessions s
         JOIN conversation_scenes sc ON sc.id = s.scene_id
         ORDER BY s.id DESC LIMIT ?1",
    )?;
    let sessions = stmt
        .query_map([limit], session_summary_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(sessions)
}

pub fn get_conversation_history(
    conn: &Connection,
    session_id: i64,
) -> Result<Vec<ConversationMessage>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, correction, latency_ms, created_at
         FROM conversation_messages WHERE session_id = ?1 ORDER BY id",
    )?;
    let messages = stmt
        .query_map([session_id], message_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(messages)
}

fn message_from_row(row: &rusqlite::Row<'_>) -> SqliteResult<ConversationMessage> {
    Ok(ConversationMessage {
        id: row.get(0)?,
        session_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        correction: row.get(4)?,
        latency_ms: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn get_conversation_message(
    conn: &Connection,
    session_id: i64,
    message_id: i64,
) -> Result<ConversationMessage, AppError> {
    conn.query_row(
        "SELECT id, session_id, role, content, correction, latency_ms, created_at
         FROM conversation_messages WHERE session_id = ?1 AND id = ?2",
        params![session_id, message_id],
        message_from_row,
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound("对话消息不存在，请刷新后重试。".to_string())
        }
        other => AppError::Sqlite(other),
    })
}

pub fn update_conversation_correction(
    conn: &Connection,
    session_id: i64,
    message_id: i64,
    correction_json: &str,
) -> Result<(), AppError> {
    let changed = conn.execute(
        "UPDATE conversation_messages SET correction = ?1
         WHERE session_id = ?2 AND id = ?3",
        params![correction_json, session_id, message_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound("对话消息不存在，请刷新后重试。".to_string()));
    }
    Ok(())
}

pub fn get_conversation_session_turns(
    conn: &Connection,
    session_id: i64,
) -> Result<i64, AppError> {
    conn.query_row(
        "SELECT turn_count FROM conversation_sessions WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    )
    .map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound("对话会话不存在，请刷新后重试。".to_string())
        }
        other => AppError::Sqlite(other),
    })
}

pub fn end_conversation_session(
    conn: &Connection,
    session_id: i64,
    summary: &str,
) -> Result<ConversationSessionSummary, AppError> {
    let now = now_secs();
    conn.execute(
        "UPDATE conversation_sessions SET ended_at = ?1, summary = ?2 WHERE id = ?3",
        params![now, summary, session_id],
    )?;
    get_conversation_session(conn, session_id)
}

/// Persist one full turn (user + assistant) inside a single transaction.
pub fn record_conversation_turn(
    conn: &mut Connection,
    session_id: i64,
    user_content: &str,
    assistant_content: &str,
    latency_ms: i64,
) -> Result<ConversationTurnResult, AppError> {
    let now = now_secs();
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO conversation_messages (session_id, role, content, correction, latency_ms, created_at)
         VALUES (?1, 'user', ?2, NULL, 0, ?3)",
        params![session_id, user_content, now],
    )?;
    let user_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO conversation_messages (session_id, role, content, correction, latency_ms, created_at)
         VALUES (?1, 'assistant', ?2, NULL, ?3, ?4)",
        params![session_id, assistant_content, latency_ms, now],
    )?;
    let assistant_id = tx.last_insert_rowid();
    tx.execute(
        "UPDATE conversation_sessions SET turn_count = turn_count + 1 WHERE id = ?1",
        [session_id],
    )?;
    let turn_count: i64 = tx.query_row(
        "SELECT turn_count FROM conversation_sessions WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    )?;
    tx.commit()?;
    Ok(ConversationTurnResult {
        user_message: ConversationMessage {
            id: user_id,
            session_id,
            role: "user".to_string(),
            content: user_content.to_string(),
            correction: None,
            latency_ms: 0,
            created_at: now,
        },
        assistant_message: ConversationMessage {
            id: assistant_id,
            session_id,
            role: "assistant".to_string(),
            content: assistant_content.to_string(),
            correction: None,
            latency_ms,
            created_at: now,
        },
        turn_count,
    })
}

pub fn get_conversation_stats(conn: &Connection) -> Result<ConversationStats, AppError> {
    let now = now_secs();
    let today_start = now - (now % 86400);
    let total_sessions: i64 =
        conn.query_row("SELECT COUNT(*) FROM conversation_sessions", [], |row| row.get(0))?;
    let total_turns: i64 = conn.query_row(
        "SELECT COALESCE(SUM(turn_count), 0) FROM conversation_sessions",
        [],
        |row| row.get(0),
    )?;
    let today_sessions: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversation_sessions WHERE started_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let today_turns: i64 = conn.query_row(
        "SELECT COALESCE(SUM(turn_count), 0) FROM conversation_sessions WHERE started_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT s.scene_id, sc.title, COUNT(*) AS sessions
         FROM conversation_sessions s
         JOIN conversation_scenes sc ON sc.id = s.scene_id
         GROUP BY s.scene_id
         ORDER BY sessions DESC, s.scene_id ASC
         LIMIT 4",
    )?;
    let recent_scenes = stmt
        .query_map([], |row| {
            Ok(SceneUsage {
                scene_id: row.get(0)?,
                title: row.get(1)?,
                sessions: row.get(2)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(ConversationStats {
        total_sessions,
        total_turns,
        today_sessions,
        today_turns,
        recent_scenes,
    })
}

/// Parse a Correction object embedded in an LLM reply. Returns None when the
/// model reported no correction or the reply could not be parsed (tolerated).
pub fn parse_correction(content: &str) -> Option<Correction> {
    let trimmed = content.trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if start >= end {
        return None;
    }
    let candidate = &trimmed[start..=end];
    let value: serde_json::Value = serde_json::from_str(candidate).ok()?;
    let get = |key: &str| -> String {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let correction = Correction {
        original: get("original"),
        suggestion: get("suggestion"),
        note: get("note"),
    };
    if correction.original.is_empty() && correction.suggestion.is_empty() && correction.note.is_empty() {
        None
    } else {
        Some(correction)
    }
}

/// JSON stored in the correction column; "{}" marks "analyzed, no correction".
pub fn correction_to_json(correction: Option<&Correction>) -> String {
    match correction {
        Some(item) => serde_json::to_string(item).unwrap_or_else(|_| "{}".to_string()),
        None => "{}".to_string(),
    }
}


/* ============================================================
 * M6 · 数据、复习与计划
 * 周趋势 / 掌握度 / 错题聚合 / 提醒与今日进度
 * ============================================================ */

#[derive(Debug, Clone, Serialize)]
pub struct DailyStat {
    pub day_start: i64,
    pub reviews: i64,
    pub dictations: i64,
    pub speaking: i64,
    pub conversation_turns: i64,
    pub duration_min: i64,
    pub accuracy_percent: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageCount {
    pub stage: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MasteryProgress {
    pub word_stages: Vec<StageCount>,
    pub sentence_statuses: Vec<StageCount>,
    pub learned_words: i64,
    pub mastered_words: i64,
    pub vocabulary_count: i64,
    pub mastery_percent: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrectionEntry {
    pub session_id: i64,
    pub scene_title: String,
    pub original: String,
    pub suggestion: String,
    pub note: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeakWord {
    pub word_id: i64,
    pub word: String,
    pub meaning: String,
    pub wrong_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MistakeBook {
    pub weak_dictation: Vec<WeakClip>,
    pub weak_speaking: Vec<SpeakingWeakSegment>,
    pub conversation_corrections: Vec<CorrectionEntry>,
    pub low_score_words: Vec<WeakWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderSettings {
    pub enabled: bool,
    pub hour: i64,
    pub minute: i64,
}

impl Default for ReminderSettings {
    fn default() -> Self {
        ReminderSettings {
            enabled: true,
            hour: 20,
            minute: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub enabled: bool,
    pub book_codes: Vec<String>,
    pub selection_mode: String,
    pub custom_word_ids: Vec<i64>,
    /// 每日学习总量：先算当日到期复习数，总量减去复习数即为今日新学数。
    #[serde(default)]
    pub daily_total: i64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        LearningConfig {
            enabled: true,
            book_codes: vec!["senior".to_string(), "college".to_string()],
            selection_mode: "random".to_string(),
            custom_word_ids: Vec::new(),
            daily_total: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TodayProgress {
    pub word_reviews: i64,
    pub sentences: i64,
    pub dictations: i64,
    pub speaking: i64,
    pub conversation_turns: i64,
    pub duration_min: i64,
    pub due_words: i64,
    pub streak_days: i64,
}

const REMINDER_KEY: &str = "reminder";

fn read_setting<T>(conn: &Connection, key: &str) -> Result<T, AppError>
where
    T: Default + for<'de> Deserialize<'de>,
{
    let raw: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = ?1", [key], |row| row.get(0))
        .optional()?;
    match raw {
        Some(json) => serde_json::from_str(&json).map_err(AppError::from),
        None => Ok(T::default()),
    }
}

fn write_setting<T: Serialize>(conn: &Connection, key: &str, value: &T) -> Result<(), AppError> {
    let json = serde_json::to_string(value)?;
    conn.execute(
        "INSERT INTO app_settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, json],
    )?;
    Ok(())
}

pub fn get_reminder_settings(conn: &Connection) -> Result<ReminderSettings, AppError> {
    read_setting(conn, REMINDER_KEY)
}

pub fn save_reminder_settings(
    conn: &Connection,
    settings: &ReminderSettings,
) -> Result<(), AppError> {
    write_setting(conn, REMINDER_KEY, settings)
}

const LEARNING_CONFIG_KEY: &str = "learning_config";

pub fn get_learning_config(conn: &Connection) -> Result<LearningConfig, AppError> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [LEARNING_CONFIG_KEY],
            |row| row.get(0),
        )
        .optional()?;
    match raw {
        Some(json) => {
            let value: serde_json::Value = serde_json::from_str(&json)?;
            let mut config: LearningConfig = serde_json::from_value(value.clone())?;
            // 兼容旧版配置：旧结构只有 daily_new + daily_review，合并为每日学习总量并写回新结构。
            if value.get("daily_total").is_none() {
                let old_new = value.get("daily_new").and_then(|v| v.as_i64()).unwrap_or(0);
                let old_review = value.get("daily_review").and_then(|v| v.as_i64()).unwrap_or(0);
                config.daily_total = (old_new + old_review).clamp(1, 300);
                let _ = write_setting(conn, LEARNING_CONFIG_KEY, &config);
            }
            Ok(config)
        }
        None => Ok(LearningConfig::default()),
    }
}

pub fn save_learning_config(conn: &Connection, config: &LearningConfig) -> Result<(), AppError> {
    write_setting(conn, LEARNING_CONFIG_KEY, config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneSettings {
    pub device_id: String,
    pub device_label: String,
}

impl Default for MicrophoneSettings {
    fn default() -> Self {
        MicrophoneSettings {
            device_id: String::new(),
            device_label: String::new(),
        }
    }
}

const MICROPHONE_KEY: &str = "microphone";

pub fn get_microphone_settings(conn: &Connection) -> Result<MicrophoneSettings, AppError> {
    read_setting(conn, MICROPHONE_KEY)
}

pub fn save_microphone_settings(
    conn: &Connection,
    settings: &MicrophoneSettings,
) -> Result<(), AppError> {
    write_setting(conn, MICROPHONE_KEY, settings)
}

/// 最近 N 天（默认 7）按日聚合的复习/听写/朗读/对话统计。所有数字都从
/// 本地事件表投影而来，与「今日统计」使用相同的 UTC 日边界保持一致。
pub fn get_weekly_stats(conn: &Connection, days: Option<i64>) -> Result<Vec<DailyStat>, AppError> {
    let days = days.unwrap_or(7).clamp(1, 90) as usize;
    let now = now_secs();
    let today_start = now - (now % 86400);
    let start = today_start - ((days as i64 - 1) * 86400);
    let end = today_start + 86400;
    let bucket = |ts: i64| -> usize {
        let index = ((ts - start) / 86400) as usize;
        index.min(days - 1)
    };

    let mut reviews = vec![0i64; days];
    let mut correct = vec![0i64; days];
    let mut duration_ms = vec![0i64; days];
    let mut dictations = vec![0i64; days];
    let mut speaking = vec![0i64; days];
    let mut conversation_turns = vec![0i64; days];

    let mut study_stmt = conn.prepare(
        "SELECT created_at, result, duration_ms FROM study_events
         WHERE kind IN ('word_review', 'sentence_event') AND created_at >= ?1 AND created_at < ?2",
    )?;
    let study_rows = study_stmt.query_map(params![start, end], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
    })?;
    for row in study_rows {
        let (created_at, result, duration) = row?;
        let b = bucket(created_at);
        reviews[b] += 1;
        correct[b] += result;
        duration_ms[b] += duration;
    }

    let mut dict_stmt = conn.prepare(
        "SELECT created_at, duration_ms FROM dictation_attempts
         WHERE created_at >= ?1 AND created_at < ?2",
    )?;
    let dict_rows = dict_stmt.query_map(params![start, end], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in dict_rows {
        let (created_at, duration) = row?;
        let b = bucket(created_at);
        dictations[b] += 1;
        duration_ms[b] += duration;
    }

    let mut speak_stmt = conn.prepare(
        "SELECT created_at, duration_ms FROM speaking_attempts
         WHERE created_at >= ?1 AND created_at < ?2",
    )?;
    let speak_rows = speak_stmt.query_map(params![start, end], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in speak_rows {
        let (created_at, duration) = row?;
        let b = bucket(created_at);
        speaking[b] += 1;
        duration_ms[b] += duration;
    }

    let mut turn_stmt = conn.prepare(
        "SELECT created_at FROM conversation_messages
         WHERE role = 'assistant' AND created_at >= ?1 AND created_at < ?2",
    )?;
    let turn_rows = turn_stmt.query_map(params![start, end], |row| {
        Ok(row.get::<_, i64>(0)?)
    })?;
    for created_at in turn_rows {
        let created_at = created_at?;
        let b = bucket(created_at);
        conversation_turns[b] += 1;
    }

    let mut result = Vec::with_capacity(days);
    for i in 0..days {
        let accuracy_percent = if reviews[i] > 0 {
            (correct[i] * 100 / reviews[i]).max(0)
        } else {
            0
        };
        result.push(DailyStat {
            day_start: start + (i as i64) * 86400,
            reviews: reviews[i],
            dictations: dictations[i],
            speaking: speaking[i],
            conversation_turns: conversation_turns[i],
            duration_min: duration_ms[i] / 60000,
            accuracy_percent,
        });
    }
    Ok(result)
}

/// 单词各阶段分布、句子掌握分布与总体掌握度（已开始学习 / 全部单词）。
pub fn get_mastery_progress(conn: &Connection) -> Result<MasteryProgress, AppError> {
    let (scope, scope_params) = scope_sql(conn)?;
    let mut word_stmt = conn.prepare(&format!(
        "SELECT s.stage, COUNT(*) FROM word_states s JOIN words w ON w.id = s.word_id
         WHERE 1 = 1 {scope} GROUP BY s.stage ORDER BY
         CASE s.stage WHEN 'new' THEN 0 WHEN 'learning' THEN 1 WHEN 'reviewing' THEN 2 ELSE 3 END"
    ))?;
    let mut word_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    word_params.extend(bind_scope(&scope_params));
    let word_stages = word_stmt
        .query_map(word_params.as_slice(), |row| {
            Ok(StageCount {
                stage: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    let mut sentence_stmt = conn.prepare(
        "SELECT status, COUNT(*) FROM sentence_states GROUP BY status ORDER BY
         CASE status WHEN 'new' THEN 0 WHEN 'learning' THEN 1 ELSE 2 END",
    )?;
    let sentence_statuses = sentence_stmt
        .query_map([], |row| {
            Ok(StageCount {
                stage: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    let total_words: i64 = scoped_count(conn, "1 = 1", &[])?;
    let learned_words: i64 = scoped_count(conn, "s.stage <> 'new'", &[])?;
    let mastered_words: i64 = scoped_count(conn, "s.stage = 'mastered'", &[])?;
    let vocabulary_count: i64 = conn.query_row("SELECT COUNT(*) FROM vocabulary", [], |row| {
        row.get(0)
    })?;

    Ok(MasteryProgress {
        word_stages,
        sentence_statuses,
        learned_words,
        mastered_words,
        vocabulary_count,
        mastery_percent: if total_words > 0 {
            (learned_words * 100 / total_words).min(100)
        } else {
            0
        },
    })
}

/// 错题/薄弱点聚合：低分单词、听写薄弱片段、朗读薄弱句段与对话纠错。
/// 每一项都保留出处，用户可回到对应模块查看证据。
pub fn get_mistake_book(conn: &Connection) -> Result<MistakeBook, AppError> {
    let dictation_stats = get_dictation_stats(conn)?;
    let speaking_stats = get_speaking_stats(conn)?;

    let mut correction_stmt = conn.prepare(
        "SELECT m.session_id, sc.title, m.content, m.correction, m.created_at
         FROM conversation_messages m
         JOIN conversation_sessions s ON s.id = m.session_id
         JOIN conversation_scenes sc ON sc.id = s.scene_id
         WHERE m.role = 'user'
           AND m.correction IS NOT NULL
           AND TRIM(m.correction) <> ''
           AND m.correction <> '{}'
         ORDER BY m.created_at DESC
         LIMIT 10",
    )?;
    let correction_rows = correction_stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;
    let mut conversation_corrections = Vec::new();
    for row in correction_rows {
        let (session_id, scene_title, content, raw, created_at) = row?;
        if let Some(correction) = parse_correction(&raw) {
            conversation_corrections.push(CorrectionEntry {
                session_id,
                scene_title,
                original: content,
                suggestion: correction.suggestion,
                note: correction.note,
                created_at,
            });
        }
    }

    let mut weak_stmt = conn.prepare(
        "SELECT w.id, w.word, w.meaning,
                SUM(CASE WHEN e.result = 0 THEN 1 ELSE 0 END) AS wrong_count,
                COUNT(*) AS total
         FROM study_events e
         JOIN words w ON w.id = e.ref_id
         WHERE e.kind = 'word_review'
         GROUP BY w.id
         HAVING wrong_count >= 2
         ORDER BY wrong_count DESC, total DESC
         LIMIT 10",
    )?;
    let low_score_words = weak_stmt
        .query_map([], |row| {
            Ok(WeakWord {
                word_id: row.get(0)?,
                word: row.get(1)?,
                meaning: row.get(2)?,
                wrong_count: row.get(3)?,
                total_count: row.get(4)?,
            })
        })?
        .collect::<SqliteResult<Vec<_>>>()?;

    Ok(MistakeBook {
        weak_dictation: dictation_stats.weakest_clips,
        weak_speaking: speaking_stats.weakest_segments,
        conversation_corrections,
        low_score_words,
    })
}

/// 今日各模块进度、计划完成度与连续学习天数。数字全部可追溯到事件。
pub fn get_today_progress(conn: &Connection) -> Result<TodayProgress, AppError> {
    let now = now_secs();
    let today_start = now - (now % 86400);
    let word_reviews: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind = 'word_review' AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let sentences: i64 = conn.query_row(
        "SELECT COUNT(*) FROM study_events WHERE kind = 'sentence_event' AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let dictations: i64 = conn.query_row(
        "SELECT COUNT(*) FROM dictation_attempts WHERE created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let speaking: i64 = conn.query_row(
        "SELECT COUNT(*) FROM speaking_attempts WHERE created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let conversation_turns: i64 = conn.query_row(
        "SELECT COUNT(*) FROM conversation_messages WHERE role = 'assistant' AND created_at >= ?1",
        [today_start],
        |row| row.get(0),
    )?;
    let duration_ms: i64 = conn.query_row(
        "SELECT COALESCE(SUM(duration_ms), 0) FROM (
             SELECT duration_ms FROM study_events
             WHERE kind IN ('word_review', 'sentence_event') AND created_at >= ?1
             UNION ALL
             SELECT duration_ms FROM dictation_attempts WHERE created_at >= ?1
             UNION ALL
             SELECT duration_ms FROM speaking_attempts WHERE created_at >= ?1
         )",
        [today_start],
        |row| row.get(0),
    )?;
    let due_words: i64 = scoped_count(conn, "s.due_at <= ?1 AND s.stage <> 'mastered'", &[&now])?;
    let streak_days = compute_streak_days(conn, today_start);

    Ok(TodayProgress {
        word_reviews,
        sentences,
        dictations,
        speaking,
        conversation_turns,
        duration_min: duration_ms / 60000,
        due_words,
        streak_days,
    })
}

/// 从今天往回数，统计连续有学习活动的天数（单词/句子/听写/朗读/对话任一即可）。
fn compute_streak_days(conn: &Connection, today_start: i64) -> i64 {
    let mut streak = 0i64;
    for day_offset in 0..365i64 {
        let day_start = today_start - day_offset * 86400;
        let day_end = day_start + 86400;
        let has_activity: i64 = conn
            .query_row(
                "SELECT
                     (SELECT COUNT(*) FROM study_events WHERE created_at >= ?1 AND created_at < ?2)
                   + (SELECT COUNT(*) FROM dictation_attempts WHERE created_at >= ?1 AND created_at < ?2)
                   + (SELECT COUNT(*) FROM speaking_attempts WHERE created_at >= ?1 AND created_at < ?2)
                   + (SELECT COUNT(*) FROM conversation_messages WHERE role = 'assistant' AND created_at >= ?1 AND created_at < ?2)",
                params![day_start, day_end],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if has_activity > 0 {
            streak += 1;
        } else {
            break;
        }
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        open_in_memory().expect("in-memory database should open")
    }

    #[test]
    fn conversation_settings_roundtrip() {
        let mut conn = test_conn();
        let settings = ConversationSettings {
            base_url: "https://api.deepseek.com/v1".to_string(),
            api_key: "sk-test".to_string(),
            model: "deepseek-chat".to_string(),
        };
        save_conversation_settings(&mut conn, &settings).unwrap();
        let loaded = get_conversation_settings(&conn).unwrap();
        assert_eq!(loaded.base_url, settings.base_url);
        assert_eq!(loaded.api_key, settings.api_key);
        assert_eq!(loaded.model, settings.model);
    }

    #[test]
    fn conversation_settings_have_defaults() {
        let conn = test_conn();
        let settings = get_conversation_settings(&conn).unwrap();
        assert!(!settings.base_url.is_empty());
        assert!(settings.api_key.is_empty());
        assert!(!settings.model.is_empty());
    }

    #[test]
    fn conversation_scenes_are_seeded() {
        let conn = test_conn();
        let scenes = get_conversation_scenes(&conn).unwrap();
        assert!(!scenes.is_empty());
        assert!(scenes.iter().any(|scene| scene.title.contains("咖啡馆")));
    }

    #[test]
    fn conversation_session_flow() {
        let mut conn = test_conn();
        let scenes = get_conversation_scenes(&conn).unwrap();
        let scene_id = scenes[0].id;
        let started = start_conversation_session(&conn, scene_id).unwrap();
        assert_eq!(started.greeting.role, "assistant");
        assert!(!started.greeting.content.is_empty());
        assert_eq!(started.session.turn_count, 0);

        let turn = record_conversation_turn(
            &mut conn,
            started.session.id,
            "I would like a coffee please.",
            "Sure! Would you like it hot or iced?",
            1200,
        )
        .unwrap();
        assert_eq!(turn.turn_count, 1);
        assert_eq!(turn.user_message.role, "user");
        assert_eq!(turn.assistant_message.role, "assistant");

        let history = get_conversation_history(&conn, started.session.id).unwrap();
        assert_eq!(history.len(), 3);

        let ended = end_conversation_session(&conn, started.session.id, "完成 1 轮对话练习。").unwrap();
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.turn_count, 1);

        let sessions = get_conversation_sessions(&conn, None).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].message_count, 3);

        let stats = get_conversation_stats(&conn).unwrap();
        assert_eq!(stats.total_sessions, 1);
        assert_eq!(stats.total_turns, 1);
        assert_eq!(stats.recent_scenes[0].scene_id, scene_id);
    }

    #[test]
    fn correction_parse_and_json_roundtrip() {
        let correction = parse_correction(
            r#"{"original":"I want coffee.","suggestion":"I would like a coffee, please.","note":"更礼貌的表达"}"#,
        )
        .unwrap();
        assert_eq!(correction.original, "I want coffee.");
        assert_eq!(correction.suggestion, "I would like a coffee, please.");
        assert!(parse_correction(r#"{"original":"","suggestion":"","note":""}"#).is_none());
        assert!(parse_correction("完全不是 JSON").is_none());
        assert_eq!(correction_to_json(Some(&correction)).contains("suggestion"), true);
        assert_eq!(correction_to_json(None), "{}");
    }

    #[test]
    fn clearing_learning_data_clears_conversations() {
        let mut conn = test_conn();
        let scenes = get_conversation_scenes(&conn).unwrap();
        let started = start_conversation_session(&conn, scenes[0].id).unwrap();
        record_conversation_turn(&mut conn, started.session.id, "hello", "Hi!", 0).unwrap();
        let dir = std::env::temp_dir().join(format!("janus-clear-conv-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        clear_learning_data(&mut conn, &dir).unwrap();
        assert!(get_conversation_sessions(&conn, None).unwrap().is_empty());
        assert!(get_conversation_history(&conn, started.session.id).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn migration_seeds_expected_content() {
        let conn = test_conn();
        let words: i64 = conn
            .query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))
            .unwrap();
        let sentences: i64 = conn
            .query_row("SELECT COUNT(*) FROM sentences", [], |row| row.get(0))
            .unwrap();
        let expected_unique = {
            let mut set = std::collections::HashSet::new();
            for book in SEED_BOOKS {
                for seed in book.words {
                    set.insert(seed.word);
                }
            }
            set.len() as i64
        };
        // M1 遗留 20 词中仅 fluent 不在种子词表，因此 words 总数 = 词表去重 + 1。
        assert_eq!(words, expected_unique + 1);
        assert_eq!(sentences, 20);
        let due: i64 = conn
            .query_row("SELECT COUNT(*) FROM word_states WHERE due_at <= ?1", [now_secs()], |row| row.get(0))
            .unwrap();
        assert_eq!(due, expected_unique + 1);
    }

    #[test]
    fn answer_word_advances_state() {
        let mut conn = test_conn();
        let card = answer_word(&mut conn, 1, 3, 5000).unwrap();
        // 首次「认识」按难度 + 随机划分（threshold 1.3 → 约 11 天 ±10%）。
        assert!((8..=14).contains(&card.interval_days));
        assert_eq!(card.stage, "learning");
        let reviews: i64 = conn
            .query_row("SELECT COUNT(*) FROM study_events WHERE kind = 'word_review'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(reviews, 1);
    }

    #[test]
    fn answer_word_rejects_invalid_rating() {
        let mut conn = test_conn();
        assert!(answer_word(&mut conn, 1, 9, 0).is_err());
    }

    #[test]
    fn complete_sentence_updates_state() {
        let mut conn = test_conn();
        let card = complete_sentence(&mut conn, 1, true, 3000).unwrap();
        assert_eq!(card.attempts, 1);
        assert_eq!(card.correct_count, 1);
        let card = complete_sentence(&mut conn, 1, true, 2000).unwrap();
        assert_eq!(card.correct_count, 2);
    }

    #[test]
    fn vocabulary_add_and_remove() {
        let conn = test_conn();
        add_to_vocabulary(&conn, 1).unwrap();
        add_to_vocabulary(&conn, 1).unwrap();
        assert_eq!(get_vocabulary(&conn).unwrap().len(), 1);
        remove_from_vocabulary(&conn, 1).unwrap();
        assert_eq!(get_vocabulary(&conn).unwrap().len(), 0);
    }

    #[test]
    fn seed_listening_clips_are_created() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let clips: i64 = conn
            .query_row("SELECT COUNT(*) FROM listening_clips", [], |row| row.get(0))
            .unwrap();
        assert_eq!(clips, 20);
        let segments: i64 = conn
            .query_row("SELECT COUNT(*) FROM listening_segments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(segments, 20);

        // Re-running is idempotent.
        ensure_seed_listening_clips(&mut conn).unwrap();
        let clips_again: i64 = conn
            .query_row("SELECT COUNT(*) FROM listening_clips", [], |row| row.get(0))
            .unwrap();
        assert_eq!(clips_again, 20);

        let dir = std::env::temp_dir().join(format!("janus-media-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let catalog = get_audio_catalog(&conn, &dir).unwrap();
        assert_eq!(catalog.len(), 20);
        assert!(catalog[0].file_path.ends_with("seed-01.wav"));
        assert!(catalog[0].duration_ms > 1000);
        assert_eq!(catalog[0].dictation_attempts, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn listening_detail_has_segments() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let detail = get_listening_detail(&conn, 1).unwrap();
        assert_eq!(detail.clip.title, "It depends on how you look at it.");
        assert_eq!(detail.segments.len(), 1);
        assert_eq!(detail.segments[0].text_en, detail.clip.title);
        assert_eq!(detail.segments[0].end_ms, detail.clip.duration_ms);
    }

    #[test]
    fn record_dictation_judges_correctness() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let ok = record_dictation(
            &mut conn,
            1,
            1,
            "It depends on how you look at it.".to_string(),
            8000,
        )
        .unwrap();
        assert!(ok.correct);
        let typo = record_dictation(
            &mut conn,
            1,
            1,
            "It depends on how you look at this".to_string(),
            7000,
        )
        .unwrap();
        assert!(!typo.correct);

        let stats = get_dictation_stats(&conn).unwrap();
        assert_eq!(stats.total_attempts, 2);
        assert_eq!(stats.correct_attempts, 1);
        assert_eq!(stats.accuracy_percent, 50);
        assert_eq!(stats.today_attempts, 2);
        assert_eq!(stats.today_correct, 1);
        assert_eq!(stats.weakest_clips.len(), 1);
        assert_eq!(stats.weakest_clips[0].clip_id, 1);
        assert_eq!(stats.weakest_clips[0].accuracy_percent, 50);
    }

    #[test]
    fn delete_seed_audio_marks_tombstone() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let result = delete_seed_audio(&mut conn, 1).unwrap();
        assert_eq!(result.file_name, "seed-01.wav");
        assert!(get_listening_detail(&conn, 1).is_err());

        // The deleted seed clip must not be resurrected by a later ensure pass.
        ensure_seed_listening_clips(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM listening_clips", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 19);
    }
    #[test]
    fn speaking_alignment_marks_feedback() {
        let expected = tokenize_speaking("It depends on how you look at it.");
        let transcribed = tokenize_speaking("it depends how you look at this");
        let words = align_words(&expected, &transcribed);
        let statuses: Vec<&str> = words.iter().map(|word| word.status.as_str()).collect();
        assert_eq!(
            statuses,
            vec![
                "correct", "correct", "missing", "correct", "correct", "correct", "correct", "changed"
            ]
        );
        assert_eq!(
            words.iter().filter(|word| word.status == "correct").count(),
            6
        );
    }

    #[test]
    fn speaking_extra_words_are_marked() {
        let expected = tokenize_speaking("Could you say that again");
        let transcribed = tokenize_speaking("could you please say that again please");
        let words = align_words(&expected, &transcribed);
        let extra: Vec<&str> = words
            .iter()
            .filter(|word| word.status == "extra")
            .map(|word| word.word.as_str())
            .collect();
        assert_eq!(extra, vec!["please", "please"]);
        assert!(words
            .iter()
            .all(|word| word.status == "correct" || word.status == "extra"));
    }

    #[test]
    fn speaking_attempt_records_and_stats() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let perfect = record_speaking_attempt(
            &mut conn,
            1,
            1,
            "It depends on how you look at it.".to_string(),
            0.92,
            false,
            4200,
        )
        .unwrap();
        assert_eq!(perfect.word_total, 8);
        assert_eq!(perfect.word_matches, 8);
        assert_eq!(perfect.accuracy_percent, 100);
        assert!(!perfect.low_confidence);
        assert_eq!(perfect.words.len(), 8);
        assert!(perfect.words.iter().all(|word| word.status == "correct"));

        let rough = record_speaking_attempt(
            &mut conn,
            1,
            1,
            "it depends how you look at this".to_string(),
            0.35,
            true,
            3600,
        )
        .unwrap();
        assert_eq!(rough.word_matches, 6);
        assert_eq!(rough.accuracy_percent, 75);
        assert!(rough.low_confidence);

        let stats = get_speaking_stats(&conn).unwrap();
        assert_eq!(stats.total_attempts, 2);
        assert_eq!(stats.today_attempts, 2);
        assert_eq!(stats.accuracy_percent, (8 + 6) * 100 / 16);
        assert_eq!(stats.low_confidence_attempts, 1);
        assert_eq!(stats.weakest_segments.len(), 1);
        assert_eq!(stats.weakest_segments[0].segment_id, 1);
        // The weakest-segment accuracy aggregates both attempts for the segment.
        assert_eq!(stats.weakest_segments[0].accuracy_percent, 87);
    }

    #[test]
    fn speaking_attempt_rejects_missing_segment() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let result = record_speaking_attempt(
            &mut conn,
            1,
            999,
            "hello".to_string(),
            0.5,
            false,
            1000,
        );
        assert!(result.is_err());
    }

    #[test]
    fn reminder_settings_roundtrip() {
        let conn = test_conn();
        let reminder = ReminderSettings {
            enabled: false,
            hour: 21,
            minute: 30,
        };
        save_reminder_settings(&conn, &reminder).unwrap();
        let loaded_reminder = get_reminder_settings(&conn).unwrap();
        assert!(!loaded_reminder.enabled);
        assert_eq!(loaded_reminder.hour, 21);
        assert_eq!(loaded_reminder.minute, 30);
    }
    #[test]
    fn microphone_settings_roundtrip() {
        let conn = test_conn();
        let loaded = get_microphone_settings(&conn).unwrap();
        assert!(loaded.device_id.is_empty());

        let settings = MicrophoneSettings {
            device_id: "default-mic-id-1".to_string(),
            device_label: "内置麦克风 (Realtek)".to_string(),
        };
        save_microphone_settings(&conn, &settings).unwrap();
        let reloaded = get_microphone_settings(&conn).unwrap();
        assert_eq!(reloaded.device_id, "default-mic-id-1");
        assert_eq!(reloaded.device_label, "内置麦克风 (Realtek)");
    }

    #[test]
    fn weekly_stats_aggregate_across_modules() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let before = get_weekly_stats(&conn, Some(7)).unwrap();
        assert_eq!(before.len(), 7);
        assert!(before.iter().all(|day| day.reviews == 0 && day.dictations == 0));

        let now = now_secs();
        let today_start = now - (now % 86400);
        conn.execute(
            "INSERT INTO study_events (kind, ref_id, result, rating, duration_ms, created_at)
             VALUES ('word_review', 1, 1, 3, 50000, ?1),
                    ('word_review', 2, 0, 1, 30000, ?1),
                    ('sentence_event', 1, 1, NULL, 20000, ?1)",
            params![today_start],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO dictation_attempts
                 (clip_id, segment_id, expected, submitted, correct, duration_ms, created_at)
             VALUES (1, 1, 'hello', 'hello', 1, 40000, ?1)",
            params![today_start],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO speaking_attempts
                 (clip_id, segment_id, expected, transcribed, word_matches, word_total,
                  accuracy_percent, confidence, low_confidence, duration_ms, created_at)
             VALUES (1, 1, 'hello', 'hello', 1, 1, 100, 0.9, 0, 30000, ?1)",
            params![today_start],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_sessions (scene_id, started_at, created_at) VALUES (1, ?1, ?1)",
            params![today_start],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_messages (session_id, role, content, created_at)
             VALUES (1, 'assistant', 'Hi!', ?1)",
            params![today_start],
        )
        .unwrap();

        let stats = get_weekly_stats(&conn, Some(7)).unwrap();
        let today = stats.last().unwrap();
        assert_eq!(today.reviews, 3);
        assert_eq!(today.dictations, 1);
        assert_eq!(today.speaking, 1);
        assert_eq!(today.conversation_turns, 1);
        assert_eq!(today.accuracy_percent, 66);
        assert_eq!(today.duration_min, (50000 + 30000 + 20000 + 40000 + 30000) / 60000);
        // Older days remain untouched.
        assert_eq!(stats[0].reviews, 0);
    }

    #[test]
    fn mastery_progress_reflects_states() {
        let mut conn = test_conn();
        answer_word(&mut conn, 1, 3, 5000).unwrap();
        complete_sentence(&mut conn, 1, true, 2000).unwrap();
        add_to_vocabulary(&conn, 2).unwrap();
        let progress = get_mastery_progress(&conn).unwrap();
        assert_eq!(progress.learned_words, 1);
        assert_eq!(progress.vocabulary_count, 1);
        assert_eq!(progress.mastery_percent, 1);
        let learning = progress
            .word_stages
            .iter()
            .find(|item| item.stage == "learning")
            .expect("learning stage should exist");
        assert_eq!(learning.count, 1);
        let new = progress
            .word_stages
            .iter()
            .find(|item| item.stage == "new")
            .expect("new stage should exist");
        assert_eq!(new.count, 53);
        assert!(progress
            .sentence_statuses
            .iter()
            .any(|item| item.stage == "learning"));
    }

    #[test]
    fn mistake_book_aggregates_weaknesses() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        answer_word(&mut conn, 1, 1, 3000).unwrap();
        answer_word(&mut conn, 1, 1, 3000).unwrap();
        answer_word(&mut conn, 2, 3, 3000).unwrap();
        record_dictation(&mut conn, 1, 1, "wrong".to_string(), 2000).unwrap();
        record_speaking_attempt(
            &mut conn,
            1,
            1,
            "it depends how you look at this".to_string(),
            0.3,
            true,
            3000,
        )
        .unwrap();
        let now = now_secs();
        conn.execute(
            "INSERT INTO conversation_sessions (scene_id, started_at, turn_count, created_at)
             VALUES (1, ?1, 2, ?1)",
            params![now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_messages (session_id, role, content, correction, created_at)
             VALUES (1, 'user', 'I am very exciting about it.',
                     '{\"original\":\"I am very exciting about it.\",\"suggestion\":\"I am very excited about it.\",\"note\":\"excited 用于描述感受\"}', ?1)",
            params![now],
        )
        .unwrap();

        let book = get_mistake_book(&conn).unwrap();
        assert!(book
            .low_score_words
            .iter()
            .any(|word| word.word_id == 1 && word.wrong_count == 2 && word.total_count == 2));
        assert_eq!(book.weak_dictation.len(), 1);
        assert_eq!(book.weak_speaking.len(), 1);
        assert_eq!(book.conversation_corrections.len(), 1);
        assert_eq!(book.conversation_corrections[0].suggestion, "I am very excited about it.");
        assert_eq!(book.conversation_corrections[0].scene_title, "咖啡馆点单");
    }

    #[test]
    fn today_progress_ties_plan_to_events() {
        let mut conn = test_conn();
        ensure_seed_listening_clips(&mut conn).unwrap();
        let before = get_today_progress(&conn).unwrap();
        assert_eq!(before.word_reviews, 0);
        assert_eq!(before.due_words, 54);

        let now = now_secs();
        let today_start = now - (now % 86400);
        for i in 0..10 {
            conn.execute(
                "INSERT INTO study_events (kind, ref_id, result, rating, duration_ms, created_at)
                 VALUES ('word_review', 1, 1, 3, 60000, ?1)",
                params![today_start + i],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO dictation_attempts
                 (clip_id, segment_id, expected, submitted, correct, duration_ms, created_at)
             VALUES (1, 1, 'hello', 'hello', 1, 300000, ?1)",
            params![today_start],
        )
        .unwrap();

        let after = get_today_progress(&conn).unwrap();
        assert_eq!(after.word_reviews, 10);
        assert_eq!(after.dictations, 1);
        assert_eq!(after.duration_min, 15);
        assert_eq!(after.streak_days, 1);
    }

    #[test]
    fn learning_config_roundtrip() {
        let mut conn = test_conn();
        let config = LearningConfig {
            enabled: true,
            book_codes: vec!["primary".to_string(), "ielts".to_string()],
            selection_mode: "random".to_string(),
            custom_word_ids: Vec::new(),
            daily_total: 20,
        };
        save_learning_config(&mut conn, &config).unwrap();
        let loaded = get_learning_config(&conn).unwrap();
        assert_eq!(loaded.book_codes, config.book_codes);
        assert_eq!(loaded.selection_mode, "random");
        assert_eq!(loaded.daily_total, 20);
        assert!(loaded.enabled);
    }

    #[test]
    fn learning_config_has_defaults() {
        let conn = test_conn();
        let config = get_learning_config(&conn).unwrap();
        assert!(config.enabled);
        assert!(config.book_codes.contains(&"senior".to_string()));
        assert!(config.book_codes.contains(&"college".to_string()));
        assert_eq!(config.selection_mode, "random");
        assert_eq!(config.daily_total, 30);
    }

    #[test]
    fn wordbooks_and_catalog_available() {
        let conn = test_conn();
        let books = get_wordbooks(&conn).unwrap();
        assert_eq!(books.len(), 6);
        let codes: Vec<&str> = books.iter().map(|book| book.code.as_str()).collect();
        assert!(codes.contains(&"primary"));
        assert!(codes.contains(&"ielts"));
        assert!(codes.contains(&"professional"));

        let catalog = get_wordbook_catalog(&conn).unwrap();
        assert_eq!(catalog.len(), 54);
        for entry in &catalog {
            assert!(!entry.book_codes.is_empty());
        }
        let senior = books.iter().find(|book| book.code == "senior").unwrap();
        let senior_count = catalog
            .iter()
            .filter(|entry| entry.book_codes.contains(&"senior".to_string()))
            .count();
        assert_eq!(senior.word_count as usize, senior_count);
        assert!(senior.word_count > 0);
    }

    #[test]
    fn wordbook_seed_is_idempotent() {
        let mut conn = test_conn();
        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))
            .unwrap();
        let books_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM wordbooks", [], |row| row.get(0))
            .unwrap();
        ensure_wordbook_seed(&mut conn).unwrap();
        let after: i64 = conn
            .query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))
            .unwrap();
        let books_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM wordbooks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(books_before, books_after);
        let books = get_wordbooks(&conn).unwrap();
        assert_eq!(books.len(), 6);
    }

    #[test]
    fn legacy_words_map_into_books() {
        let conn = test_conn();
        let mapped: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM wordbook_words ww
                 JOIN wordbooks b ON b.id = ww.wordbook_id
                 JOIN words w ON w.id = ww.word_id
                 WHERE w.word IN ('threshold', 'fluent')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(mapped, 2);
        // threshold 已存在于 college 种子词表 -> 难度按词书补齐为 1.3；
        // fluent（a2）不在任何种子词表 -> 按遗留映射进 primary（0.7）。
        let difficulty: f64 = conn
            .query_row(
                "SELECT s.difficulty FROM word_states s JOIN words w ON w.id = s.word_id
                 WHERE w.word = 'threshold'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!((difficulty - 1.3).abs() < 1e-6);
    }

    #[test]
    fn scope_filters_review_queue() {
        let conn = test_conn();
        let queue = get_review_queue(&conn, Some(100)).unwrap();
        assert_eq!(queue.len(), 54);
        for card in &queue {
            let codes: Option<String> = conn
                .query_row(
                    "SELECT GROUP_CONCAT(b.code) FROM wordbook_words ww
                     JOIN wordbooks b ON b.id = ww.wordbook_id
                     WHERE ww.word_id = ?1",
                    [card.id],
                    |row| row.get(0),
                )
                .optional()
                .unwrap();
            let codes = codes.unwrap_or_default();
            assert!(
                codes.split(',').any(|code| code == "senior" || code == "college"),
                "词 {} 不在默认范围内",
                card.word
            );
        }

        let config = LearningConfig {
            enabled: false,
            ..LearningConfig::default()
        };
        save_learning_config(&conn, &config).unwrap();
        let global = get_review_queue(&conn, Some(200)).unwrap();
        // get_review_queue 的 limit 被 clamp 到 [1, 100]，200 只会返回 100 张。
        assert_eq!(global.len(), 100);
    }

    #[test]
    fn daily_queue_mixes_review_and_new() {
        let mut conn = test_conn();
        let config = LearningConfig {
            enabled: true,
            book_codes: vec!["senior".to_string(), "college".to_string()],
            selection_mode: "random".to_string(),
            custom_word_ids: Vec::new(),
            daily_total: 15,
        };
        save_learning_config(&mut conn, &config).unwrap();

        // 把 word 1 手动置为到期复习，验证复习在前、新词补足。
        conn.execute(
            "UPDATE word_states SET stage = 'learning', interval_days = 1, due_at = ?1
             WHERE word_id = 1",
            [now_secs()],
        )
        .unwrap();

        let queue = get_daily_queue(&conn, Some(15)).unwrap();
        assert!(!queue.cards.is_empty());
        assert_eq!(queue.cards[0].id, 1);
        assert!(queue.new_count <= 14);
        assert!(queue.review_count >= 1);
        assert!(queue.cards.len() <= 15);
        assert!(queue.total_done_today >= 0);
    }

    #[test]
    fn adaptive_srs_respects_difficulty() {
        let mut conn = test_conn();
        // 高难度词（threshold，college 1.3）「模糊」：记忆不稳定 -> 隔几个单词立刻重学。
        let hard = answer_word(&mut conn, 1, 2, 5000).unwrap();
        assert_eq!(hard.interval_days, 0);
        assert_eq!(hard.stage, "learning");
        assert!(hard.difficulty > 1.0);

        // 低难度词（apple，primary 0.7）「模糊」：记忆稳定 -> 1 天后复习。
        let easy = answer_word(&mut conn, 21, 2, 5000).unwrap();
        assert_eq!(easy.interval_days, 1);
        assert!(easy.difficulty < 1.0);

        // 「认识」首次：按难度定较长基准并加随机扰动（word 2 难度 1.3 → 约 11 天）。
        let known = answer_word(&mut conn, 2, 3, 5000).unwrap();
        assert!((8..=14).contains(&known.interval_days));
        assert_eq!(known.stage, "learning");

        // 「熟悉」：加入熟词本，永不安排复习。
        let mastered = answer_word(&mut conn, 1, 4, 5000).unwrap();
        assert_eq!(mastered.stage, "mastered");
        assert_eq!(mastered.interval_days, 0);
        assert!(mastered.mastered_at.is_some());

        // 答错「重来」：repetitions 归零、lapses 增加、ease 下调。
        let reset = answer_word(&mut conn, 21, 1, 3000).unwrap();
        assert_eq!(reset.stage, "new");
        assert_eq!(reset.interval_days, 0);
        let state: (i64, i64, f64) = conn
            .query_row(
                "SELECT repetitions, lapses, ease FROM word_states WHERE word_id = 21",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(state.0, 0);
        assert_eq!(state.1, 1);
        assert!(state.2 < 2.65);

        // 首次回答会记录 introduced_at。
        let introduced: Option<i64> = conn
            .query_row(
                "SELECT introduced_at FROM word_states WHERE word_id = 21",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(introduced.is_some());
    }

    #[test]
    fn rating4_moves_to_mastered_and_never_due() {
        let mut conn = test_conn();
        let card = answer_word(&mut conn, 1, 4, 5000).unwrap();
        assert_eq!(card.stage, "mastered");
        assert!(card.mastered_at.is_some());
        // 熟词不会出现在复习队列与统计的到期数中。
        let queue = get_review_queue(&conn, Some(100)).unwrap();
        assert!(queue.iter().all(|c| c.id != 1));
        let stats = get_stats(&conn).unwrap();
        assert_eq!(stats.due_words, 53);
        assert_eq!(stats.mastered_words, 1);
        // 熟词本可查询到该词。
        let mastered = get_mastered_words(&conn).unwrap();
        assert_eq!(mastered.len(), 1);
        assert_eq!(mastered[0].word_id, 1);
        // 再次「重来」会重置回学习序列。
        let again = answer_word(&mut conn, 1, 1, 3000).unwrap();
        assert_eq!(again.stage, "new");
        assert!(again.mastered_at.is_none());
    }

    #[test]
    fn rating3_after_cap_interval_joins_mastered_book() {
        let mut conn = test_conn();
        let now = now_secs();
        // 卡片已排到封顶间隔（100 天）并到期，本次仍答「认识」：直接进入熟词本。
        conn.execute(
            "UPDATE word_states SET stage = 'reviewing', interval_days = 100, due_at = ?1
             WHERE word_id = 1",
            params![now],
        )
        .unwrap();
        let card = answer_word(&mut conn, 1, 3, 5000).unwrap();
        assert_eq!(card.stage, "mastered");
        assert!(card.mastered_at.is_some());
        assert_eq!(card.interval_days, 0);
        assert_eq!(card.due_at, i64::MAX);
        // 不再出现在复习队列，进入熟词本。
        let queue = get_review_queue(&conn, Some(100)).unwrap();
        assert!(queue.iter().all(|c| c.id != 1));
        assert_eq!(get_mastered_words(&conn).unwrap().len(), 1);
    }

    #[test]
    fn rating3_interval_caps_at_100_days() {
        let mut conn = test_conn();
        // 高难度词：基础间隔超过 100 天时封顶到 100。
        conn.execute(
            "UPDATE word_states SET stage = 'reviewing', interval_days = 80, ease = 2.8,
                     repetitions = 3
             WHERE word_id = 1",
            [],
        )
        .unwrap();
        let card = answer_word(&mut conn, 1, 3, 5000).unwrap();
        assert_eq!(card.interval_days, 100);
        assert_eq!(card.stage, "reviewing");
        // 低难度短间隔词：正常推进，不受封顶影响。
        conn.execute(
            "UPDATE word_states SET stage = 'learning', interval_days = 2, ease = 1.5,
                     repetitions = 1
             WHERE word_id = 21",
            [],
        )
        .unwrap();
        let short = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert!(short.interval_days < 100);
        assert_eq!(short.stage, "learning");
    }

    #[test]
    fn long_intervals_get_jittered_to_avoid_bunching() {
        let now = now_secs();
        // 间隔 > 5 天：按 ±20% 扰动，不同单词落点不同（避免同日扎堆）。
        let samples: Vec<i64> = (1..=40).map(|id| jitter_interval(10, id, now, false)).collect();
        assert!(samples.iter().all(|&d| (8..=12).contains(&d)));
        let mut unique = samples.clone();
        unique.sort_unstable();
        unique.dedup();
        assert!(unique.len() > 1);
        // 间隔 <= 5 天：不加扰动。
        assert_eq!(jitter_interval(5, 1, now, false), 5);
        assert_eq!(jitter_interval(1, 1, now, false), 1);
        // 首次「认识」：±1 天扰动。
        let first: Vec<i64> = (1..=40).map(|id| jitter_interval(3, id, now, true)).collect();
        assert!(first.iter().all(|&d| (2..=4).contains(&d)));
    }

    #[test]
    fn first_known_interval_scales_with_difficulty() {
        let mut conn = test_conn();
        // 低难度词（apple 0.7）首次「认识」：间隔更长（约 20 天 ±10%）。
        let easy = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert!((15..=25).contains(&easy.interval_days));
        assert_eq!(easy.stage, "learning");
        // 高难度词（threshold 1.3）首次「认识」：间隔更短（约 11 天 ±10%）。
        let hard = answer_word(&mut conn, 1, 3, 5000).unwrap();
        assert!((8..=14).contains(&hard.interval_days));
        assert_eq!(hard.stage, "learning");
    }

    #[test]
    fn subsequent_reviews_scale_from_first_known_interval() {
        let mut conn = test_conn();
        // 首次「认识」：低难度词（apple 0.7）得到较长间隔（约 20 天 ±10%）。
        let first = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert!((15..=25).contains(&first.interval_days));
        // 第二次「认识」：以首次间隔 × ease（2.5）推进并加 ±20% 随机。
        let second = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert!(second.interval_days > first.interval_days);
        assert!((30..=75).contains(&second.interval_days));
        assert_eq!(second.stage, "learning");
    }

    #[test]
    fn restart_interval_sequence_is_nonlinear() {
        // 第 n 次「认识」的基准序列：1, 2, 6, 15, 31, 56, 92, 141（调用方封顶 100）。
        let seq: Vec<i64> = (1..=8).map(restart_interval_squared).collect();
        assert_eq!(seq, vec![1, 2, 6, 15, 31, 56, 92, 141]);
        // 第 8 次后超过 100，由调用方封顶。
        assert!(restart_interval_squared(8) > 100);
    }

    #[test]
    fn rating1_then_known_reviews_next_day() {
        let mut conn = test_conn();
        // 新词「重来」：彻底忘记，10 分钟后当次重学，repetitions 归零。
        let reset = answer_word(&mut conn, 21, 1, 3000).unwrap();
        let after_reset: (i64, i64) = conn
            .query_row(
                "SELECT repetitions, lapses FROM word_states WHERE word_id = 21",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(after_reset.0, 0);
        assert_eq!(after_reset.1, 1);
        // 重来后再「认识」：不是首次认识，第二天再复习（固定 1 天），而非长间隔。
        let known = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(known.interval_days, 1);
        let after_known: (i64, i64) = conn
            .query_row(
                "SELECT repetitions, lapses FROM word_states WHERE word_id = 21",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(after_known.0, 1);
        assert_eq!(after_known.1, 1);
        assert_eq!(known.stage, "learning");
    }

    #[test]
    fn relapsed_then_known_counts_from_one() {
        let mut conn = test_conn();
        // 重来：重新开始（repetitions 归零、lapses+1）。
        answer_word(&mut conn, 21, 1, 3000).unwrap();
        // 重来后第 1 次「认识」→ 1 天（≤5 天不加随机）。
        let k1 = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(k1.interval_days, 1);
        // 第 2 次「认识」→ 2 天。
        let k2 = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(k2.interval_days, 2);
        // 第 3 次「认识」→ 基准 6 天 ±20%（5..=7）。
        let k3 = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert!((5..=7).contains(&k3.interval_days));
        assert!(k3.interval_days > k2.interval_days);
        // 第 4 次「认识」→ 基准 15 天 ±20%（12..=18）。
        let k4 = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert!((12..=18).contains(&k4.interval_days));
        assert!(k4.interval_days > k3.interval_days);
        // 再次「重来」：从 1 天重新开始。
        answer_word(&mut conn, 21, 1, 3000).unwrap();
        let k1_again = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(k1_again.interval_days, 1);
    }

    #[test]
    fn relapsed_known_twice_schedules_plan() {
        let mut conn = test_conn();
        // 重来：彻底忘记。
        answer_word(&mut conn, 21, 1, 3000).unwrap();
        // 恢复期预览：认识 = 1 天后，还需再认识一次。
        assert!(get_word_card(&conn, 21).unwrap().unwrap().preview.known_needs_again);
        // 第一次「认识」：只算确认，1 天后复习，还需再认识一次。
        let first = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(first.interval_days, 1);
        assert_eq!(first.known_streak, 1);
        // 第二次「认识」：确认完成，按平方序列推进到 2 天。
        let second = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(second.interval_days, 2);
        assert_eq!(second.known_streak, 0);
    }

    #[test]
    fn again_then_again_keeps_known_streak() {
        let mut conn = test_conn();
        // 连续两次「重来」：仍处于恢复起点，第一次「认识」依然是确认。
        answer_word(&mut conn, 21, 1, 3000).unwrap();
        answer_word(&mut conn, 21, 1, 3000).unwrap();
        let first = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(first.interval_days, 1);
        assert_eq!(first.known_streak, 1);
        let second = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(second.interval_days, 2);
        assert_eq!(second.known_streak, 0);
    }

    #[test]
    fn delete_learning_records_resets_word() {
        let mut conn = test_conn();
        // 先学习该词并加入生词本，制造学习记录。
        let card = answer_word(&mut conn, 21, 3, 5000).unwrap();
        assert_eq!(card.stage, "learning");
        add_to_vocabulary(&conn, 21).unwrap();
        let events_before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM study_events WHERE kind = 'word_review' AND ref_id = 21",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(events_before, 1);

        delete_learning_records(&mut conn, 21).unwrap();

        // 复习事件清空、移出生词本。
        let events_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM study_events WHERE kind = 'word_review' AND ref_id = 21",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(events_after, 0);
        let vocab_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM vocabulary WHERE word_id = 21",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(vocab_count, 0);
        // 状态重置为未学习：新词、无间隔、无复习计数，可再次作为新词学习。
        let reset = get_word_card(&conn, 21).unwrap().unwrap();
        assert_eq!(reset.stage, "new");
        assert_eq!(reset.interval_days, 0);
        assert!(reset.due_at <= now_secs());
        assert!(!reset.in_vocabulary);
        assert!(reset.mastered_at.is_none());
        assert_eq!(reset.known_streak, 0);
        // 难度恢复为词书基准。
        let base: f64 = conn
            .query_row(
                "SELECT COALESCE(
                    (SELECT MIN(MAX(b.difficulty * w.difficulty, 0.5), 2.0)
                     FROM wordbook_words ww JOIN wordbooks b ON b.id = ww.wordbook_id
                     JOIN words w ON w.id = ww.word_id
                     WHERE ww.word_id = 21 ORDER BY b.sort_order ASC LIMIT 1),
                    w.difficulty)
                 FROM words w WHERE w.id = 21",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!((reset.difficulty - base).abs() < 1e-9);
    }

    #[test]
    fn delete_learning_records_unknown_word_errors() {
        let mut conn = test_conn();
        let err = delete_learning_records(&mut conn, 999999).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn preview_reflects_recovery_confirm() {
        let mut conn = test_conn();
        // 新词预览：认识后按难度给较长间隔，不是确认态。
        let fresh = get_word_card(&conn, 21).unwrap().unwrap();
        assert!(!fresh.preview.known_needs_again);
        assert!(fresh.preview.known_days > 0);
        // 重来后预览：认识 = 确认（1 天后，还需再认识一次）；模糊 = 隔几个单词；重来 = 10 分钟后。
        answer_word(&mut conn, 21, 1, 3000).unwrap();
        let relapsed = get_word_card(&conn, 21).unwrap().unwrap();
        assert_eq!(relapsed.preview.again_days, 0);
        assert_eq!(relapsed.preview.fuzzy_days, 0);
        assert_eq!(relapsed.preview.known_days, 1);
        assert!(relapsed.preview.known_needs_again);
    }

    #[test]
    fn daily_queue_reviews_crowd_out_new_words() {
        let mut conn = test_conn();
        let config = LearningConfig {
            enabled: true,
            book_codes: vec!["senior".to_string(), "college".to_string()],
            selection_mode: "random".to_string(),
            custom_word_ids: Vec::new(),
            daily_total: 5,
        };
        save_learning_config(&mut conn, &config).unwrap();
        let now = now_secs();
        // 今日到期复习超过每日总量：全部配额用于复习，新学为 0。
        for id in 1..=8 {
            conn.execute(
                "UPDATE word_states SET stage = 'learning', interval_days = 1, due_at = ?1
                 WHERE word_id = ?2",
                params![now, id],
            )
            .unwrap();
        }
        let queue = get_daily_queue(&conn, Some(10)).unwrap();
        assert_eq!(queue.review_count, 5);
        assert_eq!(queue.new_count, 0);
        assert_eq!(queue.cards.len(), 5);
    }

    #[test]
    fn learning_config_legacy_migration() {
        let conn = test_conn();
        // 手动写入旧版结构（daily_new + daily_review），读取时应合并为每日学习总量并写回新结构。
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('learning_config', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [r#"{"enabled":true,"book_codes":["senior"],"selection_mode":"random","custom_word_ids":[],"daily_new":5,"daily_review":15}"#],
        )
        .unwrap();
        let config = get_learning_config(&conn).unwrap();
        assert_eq!(config.daily_total, 20);
        assert_eq!(config.book_codes, vec!["senior".to_string()]);
        // 迁移写回后再次读取仍是新结构。
        let config2 = get_learning_config(&conn).unwrap();
        assert_eq!(config2.daily_total, 20);
    }

    #[test]
    fn fuzzy_short_interval_when_shaky_history() {
        let mut conn = test_conn();
        // 先答错一次（重来），再「模糊」：历史不稳定 -> 隔几个单词立刻重学。
        answer_word(&mut conn, 1, 1, 3000).unwrap();
        let card = answer_word(&mut conn, 1, 2, 4000).unwrap();
        assert_eq!(card.interval_days, 0);
        assert_eq!(card.stage, "learning");
        // 稳定记忆的单词「模糊」：按短间隔推进。
        let stable = answer_word(&mut conn, 21, 2, 4000).unwrap();
        assert_eq!(stable.interval_days, 1);
    }

    #[test]
    fn daily_queue_interleaves_review_and_new() {
        let mut conn = test_conn();
        let config = LearningConfig {
            enabled: true,
            book_codes: vec!["senior".to_string(), "college".to_string()],
            selection_mode: "random".to_string(),
            custom_word_ids: Vec::new(),
            daily_total: 15,
        };
        save_learning_config(&mut conn, &config).unwrap();
        let now = now_secs();
        for id in 1..=3 {
            conn.execute(
                "UPDATE word_states SET stage = 'learning', interval_days = 1, due_at = ?1
                 WHERE word_id = ?2",
                params![now, id],
            )
            .unwrap();
        }
        let queue = get_daily_queue(&conn, Some(8)).unwrap();
        assert_eq!(queue.cards[0].id, 1);
        let review_ids: Vec<i64> = queue
            .cards
            .iter()
            .filter(|c| c.stage != "new")
            .map(|c| c.id)
            .collect();
        let new_ids: Vec<i64> = queue.cards.iter().filter(|c| c.stage == "new").map(|c| c.id).collect();
        assert_eq!(review_ids.len(), 3);
        assert_eq!(new_ids.len(), 5);
        // 前 2×复习数 的位置严格交替：偶数为复习、奇数为新词。
        let alternating_len = (review_ids.len() * 2).min(queue.cards.len());
        for i in 0..alternating_len {
            if i % 2 == 0 {
                assert_ne!(queue.cards[i].stage, "new");
            } else {
                assert_eq!(queue.cards[i].stage, "new");
            }
        }
    }

    #[test]
    fn daily_queue_preloads_early_reviews() {
        let mut conn = test_conn();
        let config = LearningConfig {
            enabled: true,
            book_codes: vec!["senior".to_string(), "college".to_string()],
            selection_mode: "random".to_string(),
            custom_word_ids: Vec::new(),
            daily_total: 7,
        };
        save_learning_config(&mut conn, &config).unwrap();
        let now = now_secs();
        // 未来 2 天内到期的复习：今天到期不足配额时提前分散到今天，避免扎堆。
        for id in 1..=3 {
            conn.execute(
                "UPDATE word_states SET stage = 'reviewing', interval_days = 3, due_at = ?1
                 WHERE word_id = ?2",
                params![now + 2 * 86400, id],
            )
            .unwrap();
        }
        let queue = get_daily_queue(&conn, Some(10)).unwrap();
        let future_ids: Vec<i64> = queue
            .cards
            .iter()
            .map(|c| c.id)
            .filter(|id| (1..=3).contains(id))
            .collect();
        assert_eq!(future_ids.len(), 3);
        assert!(queue.review_count >= 3);
        assert!(queue.new_count >= 0);
    }

    #[test]
    fn unmaster_returns_to_review() {
        let mut conn = test_conn();
        answer_word(&mut conn, 1, 4, 5000).unwrap();
        assert_eq!(get_mastered_words(&conn).unwrap().len(), 1);
        unmaster_word(&conn, 1).unwrap();
        assert_eq!(get_mastered_words(&conn).unwrap().len(), 0);
        let card = get_word_card(&conn, 1).unwrap().unwrap();
        assert_eq!(card.stage, "learning");
        assert!(card.mastered_at.is_none());
        // 移回复习后立即可复习。
        let queue = get_review_queue(&conn, Some(100)).unwrap();
        assert!(queue.iter().any(|c| c.id == 1));
        // 重复移除报错。
        assert!(unmaster_word(&conn, 1).is_err());
    }

    #[test]
    fn clear_resets_difficulty() {
        let dir = std::env::temp_dir().join(format!("janus-clear-diff-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut conn = test_conn();
        answer_word(&mut conn, 1, 3, 5000).unwrap();
        clear_learning_data(&mut conn, &dir).unwrap();
        let state: (String, i64, Option<i64>, f64) = conn
            .query_row(
                "SELECT stage, interval_days, introduced_at, difficulty
                 FROM word_states WHERE word_id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        assert_eq!(state.0, "new");
        assert_eq!(state.1, 0);
        assert!(state.2.is_none());
        assert!((state.3 - 1.3).abs() < 1e-6);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
#[test]
fn file_backed_database_persists_across_reopen() {
    let dir = std::env::temp_dir().join(format!("janus-persist-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("test.db");
    let _ = std::fs::remove_file(&db_path);

    {
        let mut conn = open(&db_path).unwrap();
        answer_word(&mut conn, 1, 3, 5000).unwrap();
        add_to_vocabulary(&conn, 2).unwrap();
    }

    let conn = open(&db_path).unwrap();
    let card = get_word_card(&conn, 1).unwrap().expect("word 1 should exist");
    assert!((8..=14).contains(&card.interval_days));
    assert_eq!(card.stage, "learning");
    assert_eq!(get_vocabulary(&conn).unwrap().len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_and_clear_learning_data() {
    let dir = std::env::temp_dir().join(format!("janus-export-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("test.db");
    let _ = std::fs::remove_file(&db_path);

    let mut conn = open(&db_path).unwrap();
    answer_word(&mut conn, 1, 3, 5000).unwrap();
    complete_sentence(&mut conn, 1, true, 3000).unwrap();

    let export = export_data(&conn, &dir).unwrap();
    assert!(Path::new(&export.path).exists());

    let clear = clear_learning_data(&mut conn, &dir).unwrap();
    assert!(Path::new(&clear.backup_path).exists());

    let stats = get_stats(&conn).unwrap();
    assert_eq!(stats.total_reviews, 0);
    assert_eq!(stats.vocabulary_count, 0);
    assert_eq!(stats.due_words, 54);
    let overview = get_overview(&conn).unwrap();
    assert_eq!(overview.learned_words, 0);

    let _ = std::fs::remove_dir_all(&dir);
}
