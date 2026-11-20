use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use serde::Serialize;
use std::fmt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const SCHEMA_VERSION: i64 = 1;

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
    tx.execute("DELETE FROM vocabulary", [])?;
    tx.execute("DELETE FROM word_states", [])?;
    tx.execute("DELETE FROM sentence_states", [])?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        open_in_memory().expect("in-memory database should open")
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
