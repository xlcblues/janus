use tauri::WebviewWindow;

fn desktop_error(error: tauri::Error) -> String {
    error.to_string()
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            minimize_window,
            toggle_window_maximize,
            is_window_maximized,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running Janus");
}