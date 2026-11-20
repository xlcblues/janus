mod db;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, WebviewWindow};

/// Shared application state. `rusqlite::Connection` is `Send` but not `Sync`,
/// so we wrap it in a `Mutex` to satisfy Tauri's `State` requirements.
struct AppState {
    db: Mutex<Connection>,
    data_dir: PathBuf,
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

fn app_builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::open(&data_dir.join("janus.db"))?;
            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir,
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
            clear_learning_data
        ])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app_builder()
        .run(tauri::generate_context!())
        .expect("error while running Janus");
}
