import { invoke } from "@tauri-apps/api/core";

export const isTauriRuntime =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

type VoidCommand = "minimize_window" | "close_window";

async function invokeVoid(command: VoidCommand): Promise<boolean> {
  if (!isTauriRuntime) return false;

  try {
    await invoke(command);
    return true;
  } catch (error) {
    console.error(`[Janus desktop] ${command} failed`, error);
    return false;
  }
}

async function invokeBoolean(command: "toggle_window_maximize" | "is_window_maximized") {
  if (!isTauriRuntime) return null;

  try {
    return await invoke<boolean>(command);
  } catch (error) {
    console.error(`[Janus desktop] ${command} failed`, error);
    return null;
  }
}

export const desktopWindow = {
  minimize: () => invokeVoid("minimize_window"),
  close: () => invokeVoid("close_window"),
  toggleMaximize: () => invokeBoolean("toggle_window_maximize"),
  isMaximized: () => invokeBoolean("is_window_maximized"),
};