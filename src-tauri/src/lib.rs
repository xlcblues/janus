mod asr;
mod db;
mod seed_audio;

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

fn app_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
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
            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir,
                media_dir,
                asr,
                model_path,
                model_error,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_window_maximize,
            is_window_maximized,
            close_window,
            get_review_queue,
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
            get_audio_catalog,
            get_listening_detail,
            record_dictation,
            get_dictation_stats,
            delete_seed_audio,
            get_speaking_status,
            evaluate_speaking,
            get_speaking_stats
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app_builder()
        .run(tauri::generate_context!())
        .expect("error while running Janus");
}
