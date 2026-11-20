use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::seed_audio::{wav_duration_ms, SEED_CLIPS};

pub const SCHEMA_VERSION: i64 = 4;

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
    pub in_vocabulary: bool,
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
    Ok(())
}

pub fn open(path: &Path) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&mut conn)?;
    Ok(conn)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn open_in_memory() -> Result<Connection, AppError> {
    let mut conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&mut conn)?;
    Ok(conn)
}

fn word_card_from_row(row: &rusqlite::Row<'_>) -> SqliteResult<WordCard> {
    Ok(WordCard {
        id: row.get(0)?,
        word: row.get(1)?,
        phonetic: row.get(2)?,
        meaning: row.get(3)?,
        example_en: row.get(4)?,
        example_zh: row.get(5)?,
        stage: row.get(6)?,
        interval_days: row.get(7)?,
        due_at: row.get(8)?,
        in_vocabulary: row.get::<_, i64>(9)? != 0,
    })
}

const WORD_CARD_SELECT: &str = "
SELECT w.id, w.word, w.phonetic, w.meaning, w.example_en, w.example_zh,
       s.stage, s.interval_days, s.due_at,
       EXISTS(SELECT 1 FROM vocabulary v WHERE v.word_id = w.id) AS in_vocabulary
FROM words w
JOIN word_states s ON s.word_id = w.id
";

pub fn get_review_queue(conn: &Connection, limit: Option<i64>) -> Result<Vec<WordCard>, AppError> {
    let limit = limit.unwrap_or(10).clamp(1, 100);
    let now = now_secs();
    let mut stmt = conn.prepare(&format!(
        "{WORD_CARD_SELECT} WHERE s.due_at <= ?1 ORDER BY s.due_at ASC, w.id ASC LIMIT ?2"
    ))?;
    let cards = stmt
        .query_map(params![now, limit], word_card_from_row)?
        .collect::<SqliteResult<Vec<_>>>()?;
    Ok(cards)
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
    let total_words: i64 = conn.query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))?;
    let due_words: i64 = conn.query_row(
        "SELECT COUNT(*) FROM word_states WHERE due_at <= ?1",
        [now],
        |row| row.get(0),
    )?;
    let learned_words: i64 = conn.query_row(
        "SELECT COUNT(*) FROM word_states WHERE stage <> 'new'",
        [],
        |row| row.get(0),
    )?;
    let mastered_words: i64 = conn.query_row(
        "SELECT COUNT(*) FROM word_states WHERE stage = 'mastered'",
        [],
        |row| row.get(0),
    )?;
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

fn next_srs(current: &SrsState, rating: i64) -> SrsState {
    let now = now_secs();
    let mut next = SrsState {
        stage: current.stage.clone(),
        interval_days: current.interval_days,
        ease: current.ease,
        repetitions: current.repetitions,
        lapses: current.lapses,
        due_at: current.due_at,
    };

    match rating {
        1 => {
            next.repetitions = 0;
            next.lapses += 1;
            next.interval_days = 0;
            next.ease = (current.ease - 0.20).max(1.30);
            next.due_at = now + 600;
        }
        2 => {
            next.repetitions += 1;
            next.ease = (current.ease - 0.15).max(1.30);
            next.interval_days = if next.repetitions == 1 {
                1
            } else {
                ((current.interval_days as f64) * next.ease).round().max(1.0) as i64
            };
            next.due_at = now + next.interval_days * 86400;
        }
        3 => {
            next.repetitions += 1;
            next.interval_days = match next.repetitions {
                1 => 1,
                2 => 6,
                _ => ((current.interval_days as f64) * current.ease).round().max(1.0) as i64,
            };
            next.due_at = now + next.interval_days * 86400;
        }
        4 => {
            next.repetitions += 1;
            next.ease = (current.ease + 0.15).min(3.00);
            next.interval_days = match next.repetitions {
                1 => 4,
                2 => 14,
                _ => ((current.interval_days as f64) * current.ease * 1.30)
                    .round()
                    .max(1.0) as i64,
            };
            next.due_at = now + next.interval_days * 86400;
        }
        _ => {}
    }

    if next.repetitions >= 5 && next.interval_days >= 21 {
        next.stage = "mastered".to_string();
    } else if next.interval_days > 1 && next.repetitions >= 2 {
        next.stage = "reviewing".to_string();
    } else if next.repetitions > 0 {
        next.stage = "learning".to_string();
    } else {
        next.stage = "new".to_string();
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
            "SELECT stage, interval_days, ease, repetitions, lapses, due_at
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
                })
            },
        )
        .optional()?;
    let current = current.ok_or_else(|| AppError::NotFound("没有找到这个单词的复习状态。".to_string()))?;

    let next = next_srs(&current, rating);
    tx.execute(
        "UPDATE word_states
         SET stage = ?1, interval_days = ?2, ease = ?3, repetitions = ?4,
             lapses = ?5, due_at = ?6, last_reviewed_at = ?7
         WHERE word_id = ?8",
        params![
            next.stage,
            next.interval_days,
            next.ease,
            next.repetitions,
            next.lapses,
            next.due_at,
            now,
            word_id
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
    let due_words: i64 = conn.query_row(
        "SELECT COUNT(*) FROM word_states WHERE due_at <= ?1",
        [now],
        |row| row.get(0),
    )?;
    let mastered_words: i64 = conn.query_row(
        "SELECT COUNT(*) FROM word_states WHERE stage = 'mastered'",
        [],
        |row| row.get(0),
    )?;
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
        "INSERT INTO word_states (word_id, stage, interval_days, ease, repetitions, lapses, due_at, created_at)
         SELECT id, 'new', 0, 2.5, 0, 0, ?1, ?1 FROM words",
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
        assert_eq!(words, 20);
        assert_eq!(sentences, 20);
        let due: i64 = conn
            .query_row("SELECT COUNT(*) FROM word_states WHERE due_at <= ?1", [now_secs()], |row| row.get(0))
            .unwrap();
        assert_eq!(due, 20);
    }

    #[test]
    fn answer_word_advances_state() {
        let mut conn = test_conn();
        let card = answer_word(&mut conn, 1, 3, 5000).unwrap();
        assert_eq!(card.interval_days, 1);
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
    assert_eq!(card.interval_days, 1);
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
    assert_eq!(stats.due_words, 20);
    let overview = get_overview(&conn).unwrap();
    assert_eq!(overview.learned_words, 0);

    let _ = std::fs::remove_dir_all(&dir);
}
