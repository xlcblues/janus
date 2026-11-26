mod asr;
mod db;
mod llm;
mod reminder;
mod seed_audio;
mod seed_words;
mod update;

use crate::asr::AsrEngine;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Manager, WebviewWindow};

/// Shared application state. `rusqlite::Connection` is `Send` but not
/// `Sync`, so it is wrapped in a `Mutex` to satisfy Tauri's `State`
/// requirements. The Whisper engine is loaded lazily and shared across async
/// commands via `Arc`.
struct AppState {
    db: Mutex<Connection>,
    data_dir: PathBuf,
    media_dir: PathBuf,
    asr: Option<Arc<AsrEngine>>,
    model_path: Option<PathBuf>,
    model_error: Option<String>,
    reminder: Arc<Mutex<db::ReminderSettings>>,
}

fn desktop_error(error: tauri::Error) -> String {
    error.to_string()
}

fn with_db_read<T>(
    state: &AppState,
    operation: impl FnOnce(&Connection) -> Result<T, db::AppError>,
) -> Result<T, String> {
    let guard = state
        .db
        .lock()
        .map_err(|_| "本地数据锁获取失败，请稍后重试。".to_string())?;
    operation(&guard).map_err(|error| error.user_message())
}

fn with_db_write<T>(
    state: &AppState,
    operation: impl FnOnce(&mut Connection) -> Result<T, db::AppError>,
) -> Result<T, String> {
    let mut guard = state
        .db
        .lock()
        .map_err(|_| "本地数据锁获取失败，请稍后重试。".to_string())?;
    operation(&mut guard).map_err(|error| error.user_message())
}

/// Locate the offline Whisper model across packaged resources, the dev
/// checkout and the app data dir. Returns None when the model has not been
/// downloaded yet.
fn resolve_model_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    const MODEL_NAME: &str = "ggml-tiny.en.bin";
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("models").join(MODEL_NAME));
        candidates.push(resource_dir.join(MODEL_NAME));
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("src-tauri").join("models").join(MODEL_NAME));
        candidates.push(current_dir.join("models").join(MODEL_NAME));
    }
    if let Ok(data_dir) = app.path().app_data_dir() {
        candidates.push(data_dir.join("models").join(MODEL_NAME));
    }
    candidates.into_iter().find(|path| path.exists())
}

#[tauri::command]
fn minimize_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(desktop_error)
}

#[tauri::command]
fn toggle_window_maximize(window: WebviewWindow) -> Result<bool, String> {
    let is_maximized = window.is_maximized().map_err(desktop_error)?;

    if is_maximized {
        window.unmaximize().map_err(desktop_error)?;
    } else {
        window.maximize().map_err(desktop_error)?;
    }

    Ok(!is_maximized)
}

#[tauri::command]
fn is_window_maximized(window: WebviewWindow) -> Result<bool, String> {
    window.is_maximized().map_err(desktop_error)
}

#[tauri::command]
fn close_window(window: WebviewWindow) -> Result<(), String> {
    window.close().map_err(desktop_error)
}

#[tauri::command]
fn get_review_queue(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<db::WordCard>, String> {
    with_db_read(state.inner(), |conn| db::get_review_queue(conn, limit))
}

#[tauri::command]
fn get_wordbooks(state: tauri::State<'_, AppState>) -> Result<Vec<db::Wordbook>, String> {
    with_db_read(state.inner(), db::get_wordbooks)
}

#[tauri::command]
fn get_learning_config(state: tauri::State<'_, AppState>) -> Result<db::LearningConfig, String> {
    with_db_read(state.inner(), db::get_learning_config)
}

#[tauri::command]
fn save_learning_config(
    state: tauri::State<'_, AppState>,
    config: db::LearningConfig,
) -> Result<(), String> {
    with_db_write(state.inner(), |conn| db::save_learning_config(conn, &config))
}

#[tauri::command]
fn get_daily_queue(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> Result<db::DailyQueue, String> {
    with_db_read(state.inner(), |conn| db::get_daily_queue(conn, limit))
}

#[tauri::command]
fn get_wordbook_catalog(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::WordbookCatalogEntry>, String> {
    with_db_read(state.inner(), db::get_wordbook_catalog)
}

#[tauri::command]
fn get_word_catalog(state: tauri::State<'_, AppState>) -> Result<Vec<db::WordCard>, String> {
    with_db_read(state.inner(), db::get_word_catalog)
}

#[tauri::command]
fn get_overview(state: tauri::State<'_, AppState>) -> Result<db::Overview, String> {
    with_db_read(state.inner(), db::get_overview)
}

#[tauri::command]
fn answer_word(
    state: tauri::State<'_, AppState>,
    word_id: i64,
    rating: i64,
    duration_ms: i64,
) -> Result<db::WordCard, String> {
    with_db_write(state.inner(), |conn| {
        db::answer_word(conn, word_id, rating, duration_ms)
    })
}

#[tauri::command]
fn get_sentences(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<db::SentenceCard>, String> {
    with_db_read(state.inner(), |conn| db::get_sentences(conn, limit))
}

#[tauri::command]
fn complete_sentence(
    state: tauri::State<'_, AppState>,
    sentence_id: i64,
    correct: bool,
    duration_ms: i64,
) -> Result<db::SentenceCard, String> {
    with_db_write(state.inner(), |conn| {
        db::complete_sentence(conn, sentence_id, correct, duration_ms)
    })
}

#[tauri::command]
fn get_vocabulary(state: tauri::State<'_, AppState>) -> Result<Vec<db::VocabularyEntry>, String> {
    with_db_read(state.inner(), db::get_vocabulary)
}

#[tauri::command]
fn add_to_vocabulary(state: tauri::State<'_, AppState>, word_id: i64) -> Result<(), String> {
    with_db_read(state.inner(), |conn| db::add_to_vocabulary(conn, word_id))
}

#[tauri::command]
fn remove_from_vocabulary(state: tauri::State<'_, AppState>, word_id: i64) -> Result<(), String> {
    with_db_read(state.inner(), |conn| db::remove_from_vocabulary(conn, word_id))
}

#[tauri::command]
fn get_stats(state: tauri::State<'_, AppState>) -> Result<db::StudyStats, String> {
    with_db_read(state.inner(), db::get_stats)
}

#[tauri::command]
fn export_data(state: tauri::State<'_, AppState>) -> Result<db::ExportResult, String> {
    with_db_read(state.inner(), |conn| db::export_data(conn, &state.inner().data_dir))
}

#[tauri::command]
fn clear_learning_data(state: tauri::State<'_, AppState>) -> Result<db::ClearResult, String> {
    with_db_write(state.inner(), |conn| {
        db::clear_learning_data(conn, &state.inner().data_dir)
    })
}

#[tauri::command]
fn get_mastered_words(state: tauri::State<'_, AppState>) -> Result<Vec<db::MasteredWord>, String> {
    with_db_read(state.inner(), db::get_mastered_words)
}

#[tauri::command]
fn unmaster_word(state: tauri::State<'_, AppState>, word_id: i64) -> Result<(), String> {
    with_db_read(state.inner(), |conn| db::unmaster_word(conn, word_id))
}

#[tauri::command]
fn delete_learning_records(state: tauri::State<'_, AppState>, word_id: i64) -> Result<(), String> {
    with_db_write(state.inner(), |conn| db::delete_learning_records(conn, word_id))
}

#[tauri::command]
fn get_audio_catalog(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::AudioCatalogEntry>, String> {
    let media_dir = state.inner().media_dir.clone();
    with_db_read(state.inner(), |conn| db::get_audio_catalog(conn, &media_dir))
}

#[tauri::command]
fn get_listening_detail(
    state: tauri::State<'_, AppState>,
    clip_id: i64,
) -> Result<db::ListeningDetail, String> {
    with_db_read(state.inner(), |conn| db::get_listening_detail(conn, clip_id))
}

#[tauri::command]
fn record_dictation(
    state: tauri::State<'_, AppState>,
    clip_id: i64,
    segment_id: i64,
    submitted: String,
    duration_ms: i64,
) -> Result<db::DictationResult, String> {
    with_db_write(state.inner(), |conn| {
        db::record_dictation(conn, clip_id, segment_id, submitted, duration_ms)
    })
}

#[tauri::command]
fn get_dictation_stats(
    state: tauri::State<'_, AppState>,
) -> Result<db::DictationStats, String> {
    with_db_read(state.inner(), db::get_dictation_stats)
}

#[tauri::command]
fn delete_seed_audio(
    state: tauri::State<'_, AppState>,
    clip_id: i64,
) -> Result<db::DeleteResult, String> {
    let result = with_db_write(state.inner(), |conn| db::delete_seed_audio(conn, clip_id))?;
    let target = state.inner().media_dir.join(&result.file_name);
    if target.exists() {
        std::fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    Ok(result)
}

#[derive(serde::Serialize)]
struct SpeakingStatus {
    available: bool,
    model_path: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn get_speaking_status(state: tauri::State<'_, AppState>) -> Result<SpeakingStatus, String> {
    let inner = state.inner();
    Ok(SpeakingStatus {
        available: inner.asr.is_some(),
        model_path: inner
            .model_path
            .as_ref()
            .map(|path| path.display().to_string()),
        error: inner.model_error.clone(),
    })
}

/// Transcribe a recorded WAV blob offline, then align it against the target
/// segment and persist the attempt. Heavy inference runs on a blocking thread
/// pool; the raw audio is discarded after transcription (never stored).
#[tauri::command]
async fn evaluate_speaking(
    state: tauri::State<'_, AppState>,
    clip_id: i64,
    segment_id: i64,
    wav: Vec<u8>,
    duration_ms: i64,
) -> Result<db::SpeakingResult, String> {
    let engine = state.inner().asr.clone().ok_or_else(|| {
        "离线语音模型未安装：请运行 scripts/download-whisper-model.ps1 下载后重启应用。".to_string()
    })?;
    let transcription =
        match tauri::async_runtime::spawn_blocking(move || engine.transcribe(&wav)).await {
            Ok(Ok(transcription)) => transcription,
            Ok(Err(message)) => return Err(message),
            Err(error) => return Err(format!("语音识别任务执行失败：{error}")),
        };
    with_db_write(state.inner(), |conn| {
        db::record_speaking_attempt(
            conn,
            clip_id,
            segment_id,
            transcription.text,
            transcription.confidence as f64,
            transcription.low_confidence,
            duration_ms,
        )
    })
}

#[tauri::command]
fn get_speaking_stats(state: tauri::State<'_, AppState>) -> Result<db::SpeakingStats, String> {
    with_db_read(state.inner(), db::get_speaking_stats)
}


#[tauri::command]
fn get_conversation_settings(
    state: tauri::State<'_, AppState>,
) -> Result<db::ConversationSettings, String> {
    with_db_read(state.inner(), db::get_conversation_settings)
}

#[tauri::command]
fn save_conversation_settings(
    state: tauri::State<'_, AppState>,
    settings: db::ConversationSettings,
) -> Result<(), String> {
    with_db_write(state.inner(), |conn| {
        db::save_conversation_settings(conn, &settings)
    })
}

#[tauri::command]
async fn test_conversation_connection(
    state: tauri::State<'_, AppState>,
    settings: Option<db::ConversationSettings>,
) -> Result<llm::LlmTestResult, String> {
    let settings = match settings {
        Some(value) => value,
        None => with_db_read(state.inner(), db::get_conversation_settings)?,
    };
    let base_url = settings.base_url.trim().to_string();
    let api_key = settings.api_key.trim().to_string();
    let model = settings.model.trim().to_string();
    if base_url.is_empty() || api_key.is_empty() || model.is_empty() {
        return Ok(llm::LlmTestResult {
            ok: false,
            latency_ms: 0,
            message: "请先填写 Base URL、API Key 与模型名称。".to_string(),
        });
    }
    let config = llm::LlmConfig {
        base_url,
        api_key,
        model,
    };
    let result = tauri::async_runtime::spawn_blocking(move || llm::test_connection(&config))
        .await
        .map_err(|error| format!("连接测试任务执行失败：{error}"))?;
    Ok(result)
}

#[tauri::command]
fn get_conversation_scenes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::ConversationScene>, String> {
    with_db_read(state.inner(), db::get_conversation_scenes)
}

#[tauri::command]
fn start_conversation(
    state: tauri::State<'_, AppState>,
    scene_id: i64,
) -> Result<db::ConversationStartResult, String> {
    with_db_write(state.inner(), |conn| {
        db::start_conversation_session(conn, scene_id)
    })
}

#[tauri::command]
fn get_conversation_history(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<Vec<db::ConversationMessage>, String> {
    with_db_read(state.inner(), |conn| {
        db::get_conversation_history(conn, session_id)
    })
}

#[tauri::command]
fn get_conversation_sessions(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<db::ConversationSessionSummary>, String> {
    with_db_read(state.inner(), |conn| {
        db::get_conversation_sessions(conn, limit)
    })
}

#[tauri::command]
fn get_conversation_stats(
    state: tauri::State<'_, AppState>,
) -> Result<db::ConversationStats, String> {
    with_db_read(state.inner(), db::get_conversation_stats)
}

#[tauri::command]
fn end_conversation(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<db::ConversationSessionSummary, String> {
    with_db_write(state.inner(), |conn| {
        let turns = db::get_conversation_session_turns(conn, session_id)?;
        let summary = format!("完成 {turns} 轮 AI 对话练习。");
        db::end_conversation_session(conn, session_id, &summary)
    })
}

/// Send one user turn: build the scene prompt + history, call the LLM, and
/// persist the turn atomically. The reply is returned immediately; a
/// lightweight correction pass runs separately via
/// `analyze_conversation_correction` so it never delays the reply.
#[tauri::command]
async fn send_conversation_message(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    content: String,
) -> Result<db::ConversationTurnResult, String> {
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Err("消息内容为空，请输入或录制语音后再发送。".to_string());
    }
    let (settings, scene, history) = with_db_read(state.inner(), |conn| {
        let settings = db::get_conversation_settings(conn)?;
        let scene = db::get_conversation_scene_by_session(conn, session_id)?;
        let history = db::get_conversation_history(conn, session_id)?;
        Ok((settings, scene, history))
    })?;
    let config = llm::LlmConfig {
        base_url: settings.base_url.trim().to_string(),
        api_key: settings.api_key.trim().to_string(),
        model: settings.model.trim().to_string(),
    };
    if config.base_url.is_empty() || config.api_key.is_empty() || config.model.is_empty() {
        return Err(
            "尚未配置 AI 对话服务：请先在“对话设置”中填写 Base URL、API Key 与模型名称。"
                .to_string(),
        );
    }
    let system_prompt = format!(
        "You are a role-playing English conversation partner. Stay in character and never break immersion.\n\
         Scene title: {title}\nScene goal: {goal}\nYour role: {role}\nLearner goal: {user_goal}\n\
         Target vocabulary: {target_words}\nCEFR level: {level}\n\n\
         Rules:\n\
         - Reply naturally in English with 1-3 short sentences (shorter for lower levels).\n\
         - Move the conversation forward; ask a follow-up question when appropriate.\n\
         - If the learner's message is unclear, ask a clarifying question instead of guessing.\n\
         - Do not provide corrections here; corrections are handled separately.",
        title = scene.title,
        goal = scene.goal,
        role = scene.role,
        user_goal = scene.user_goal,
        target_words = scene.target_words,
        level = scene.level,
    );
    let mut messages = vec![llm::ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];
    for message in &history {
        if message.role == "user" || message.role == "assistant" {
            messages.push(llm::ChatMessage {
                role: message.role.clone(),
                content: message.content.clone(),
            });
        }
    }
    messages.push(llm::ChatMessage {
        role: "user".to_string(),
        content: trimmed.clone(),
    });
    let reply =
        match tauri::async_runtime::spawn_blocking(move || llm::chat_completion(&config, &messages, 300))
            .await
        {
            Ok(Ok(reply)) => reply,
            Ok(Err(message)) => return Err(message),
            Err(error) => return Err(format!("AI 回复任务执行失败：{error}")),
        };
    with_db_write(state.inner(), |conn| {
        db::record_conversation_turn(conn, session_id, &trimmed, &reply.content, reply.latency_ms)
    })
}

/// Run the best-effort grammar/naturalness pass on a user message and store
/// the result. Returns None when the model found nothing worth correcting or
/// the analysis could not be produced (this never blocks the conversation).
#[tauri::command]
async fn analyze_conversation_correction(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    message_id: i64,
) -> Result<Option<db::Correction>, String> {
    let (message, settings) = with_db_read(state.inner(), |conn| {
        let message = db::get_conversation_message(conn, session_id, message_id)?;
        let settings = db::get_conversation_settings(conn)?;
        Ok((message, settings))
    })?;
    if message.role != "user" {
        return Ok(None);
    }
    if let Some(existing) = &message.correction {
        if !existing.trim().is_empty() {
            return Ok(db::parse_correction(existing));
        }
    }
    let config = llm::LlmConfig {
        base_url: settings.base_url.trim().to_string(),
        api_key: settings.api_key.trim().to_string(),
        model: settings.model.trim().to_string(),
    };
    if config.base_url.is_empty() || config.api_key.is_empty() || config.model.is_empty() {
        return Ok(None);
    }
    let user_text = message.content;
    let prompt = format!(
        "You are an English tutor. The learner said: \"{user_text}\"\n\
         Check the sentence for grammar, word choice, or naturalness issues worth learning.\n\
         - If there is a notable issue, reply with ONLY a JSON object (no markdown code fences):\n\
         {{\"original\":\"the learner's sentence\",\"suggestion\":\"a more natural corrected version\",\"note\":\"one short Chinese explanation\"}}\n\
         - If the sentence is fine, reply with exactly: {{\"original\":\"\",\"suggestion\":\"\",\"note\":\"\"}}"
    );
    let correction_messages = vec![llm::ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];
    let reply =
        match tauri::async_runtime::spawn_blocking(move || llm::chat_completion(&config, &correction_messages, 120))
            .await
        {
            Ok(Ok(reply)) => reply,
            Ok(Err(_)) | Err(_) => return Ok(None),
        };
    let correction = db::parse_correction(&reply.content);
    let stored = db::correction_to_json(correction.as_ref());
    with_db_write(state.inner(), |conn| {
        db::update_conversation_correction(conn, session_id, message_id, &stored)
    })?;
    Ok(correction)
}

#[derive(serde::Serialize)]
struct Transcript {
    text: String,
    confidence: f64,
    low_confidence: bool,
}

/// Free-form offline transcription for the conversation module (no segment
/// alignment, no persistence). Reuses the M4 Whisper engine.
#[tauri::command]
async fn transcribe_speech(
    state: tauri::State<'_, AppState>,
    wav: Vec<u8>,
) -> Result<Transcript, String> {
    let engine = state.inner().asr.clone().ok_or_else(|| {
        "离线语音模型未安装：请运行 scripts/download-whisper-model.ps1 下载后重启应用。".to_string()
    })?;
    let transcription =
        match tauri::async_runtime::spawn_blocking(move || engine.transcribe(&wav)).await {
            Ok(Ok(transcription)) => transcription,
            Ok(Err(message)) => return Err(message),
            Err(error) => return Err(format!("语音识别任务执行失败：{error}")),
        };
    Ok(Transcript {
        text: transcription.text,
        confidence: transcription.confidence as f64,
        low_confidence: transcription.low_confidence,
    })
}


/* ============================================================
 * M6 · 数据、复习与计划 IPC
 * ============================================================ */

#[tauri::command]
fn get_today_progress(state: tauri::State<'_, AppState>) -> Result<db::TodayProgress, String> {
    with_db_read(state.inner(), db::get_today_progress)
}

#[tauri::command]
fn get_weekly_stats(
    state: tauri::State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<db::DailyStat>, String> {
    with_db_read(state.inner(), |conn| db::get_weekly_stats(conn, days))
}

#[tauri::command]
fn get_mastery_progress(state: tauri::State<'_, AppState>) -> Result<db::MasteryProgress, String> {
    with_db_read(state.inner(), db::get_mastery_progress)
}

#[tauri::command]
fn get_mistake_book(state: tauri::State<'_, AppState>) -> Result<db::MistakeBook, String> {
    with_db_read(state.inner(), db::get_mistake_book)
}

#[tauri::command]
fn get_reminder_settings(state: tauri::State<'_, AppState>) -> Result<db::ReminderSettings, String> {
    with_db_read(state.inner(), db::get_reminder_settings)
}

#[tauri::command]
fn get_microphone_settings(state: tauri::State<'_, AppState>) -> Result<db::MicrophoneSettings, String> {
    with_db_read(state.inner(), db::get_microphone_settings)
}

#[tauri::command]
fn save_microphone_settings(
    state: tauri::State<'_, AppState>,
    settings: db::MicrophoneSettings,
) -> Result<(), String> {
    with_db_write(state.inner(), |conn| db::save_microphone_settings(conn, &settings))
}

#[tauri::command]
fn get_notification_permission(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_notification::NotificationExt;
    let state = app
        .notification()
        .permission_state()
        .map_err(|error| format!("无法读取通知权限状态：{error}"))?;
    Ok(match state {
        tauri::plugin::PermissionState::Granted => "granted".to_string(),
        tauri::plugin::PermissionState::Denied => "denied".to_string(),
        _ => "prompt".to_string(),
    })
}

#[tauri::command]
fn send_test_notification(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title("Janus · 测试通知")
        .body("桌面提醒已就绪：每日复习提醒将按设置的时间出现。")
        .show()
        .map_err(|error| format!("发送测试通知失败：{error}"))
}

#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

#[tauri::command]
fn get_app_data_dir(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.data_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn check_for_update(app: tauri::AppHandle) -> Result<update::UpdateCheck, String> {
    let current = app.package_info().version.to_string();
    update::check_for_update(&current, "xlcblues/janus")
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("仅支持打开 http(s) 链接。".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|error| format!("无法打开链接：{error}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = url;
        return Err("当前平台暂不支持打开外部链接。".to_string());
    }
    Ok(())
}

#[tauri::command]
fn save_reminder_settings(
    state: tauri::State<'_, AppState>,
    settings: db::ReminderSettings,
) -> Result<(), String> {
    with_db_write(state.inner(), |conn| db::save_reminder_settings(conn, &settings))?;
    if let Ok(mut guard) = state.reminder.lock() {
        *guard = settings;
    }
    Ok(())
}

fn app_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let media_dir = data_dir.join("media_cache");
            std::fs::create_dir_all(&media_dir)?;
            let mut conn = db::open(&data_dir.join("janus.db"))?;
            for (index, clip) in seed_audio::SEED_CLIPS.iter().enumerate() {
                let target = media_dir.join(format!("seed-{:02}.wav", index + 1));
                if !target.exists() {
                    std::fs::write(&target, clip.wav)?;
                }
            }
            db::ensure_seed_listening_clips(&mut conn)?;
            let model_path = resolve_model_path(app.handle());
            let (asr, model_error) = match &model_path {
                Some(path) => match AsrEngine::load(path) {
                    Ok(engine) => (Some(Arc::new(engine)), None),
                    Err(error) => (None, Some(error)),
                },
                None => (None, None),
            };
            let reminder = Arc::new(Mutex::new(db::get_reminder_settings(&conn)?));
            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir,
                media_dir,
                asr,
                model_path,
                model_error,
                reminder: reminder.clone(),
            });
            reminder::start_reminder_scheduler(app.handle().clone(), reminder);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_window_maximize,
            is_window_maximized,
            close_window,
            get_review_queue,
            get_wordbooks,
            get_learning_config,
            save_learning_config,
            get_daily_queue,
            get_wordbook_catalog,
            get_word_catalog,
            get_overview,
            answer_word,
            get_sentences,
            complete_sentence,
            get_vocabulary,
            add_to_vocabulary,
            remove_from_vocabulary,
            get_stats,
            export_data,
            clear_learning_data,
            get_mastered_words,
            unmaster_word,
            delete_learning_records,
            get_audio_catalog,
            get_listening_detail,
            record_dictation,
            get_dictation_stats,
            delete_seed_audio,
            get_speaking_status,
            evaluate_speaking,
            get_speaking_stats,
            get_conversation_settings,
            save_conversation_settings,
            test_conversation_connection,
            get_conversation_scenes,
            start_conversation,
            get_conversation_history,
            get_conversation_sessions,
            get_conversation_stats,
            end_conversation,
            send_conversation_message,
            analyze_conversation_correction,
            transcribe_speech,
            get_today_progress,
            get_weekly_stats,
            get_mastery_progress,
            get_mistake_book,
            get_reminder_settings,
            save_reminder_settings,
            get_microphone_settings,
            save_microphone_settings,
            get_notification_permission,
            send_test_notification,
            get_app_version,
            get_app_data_dir,
            check_for_update,
            open_url
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app_builder()
        .run(tauri::generate_context!())
        .expect("error while running Janus");
}
