import { Minimize, Moon, Search, Sun, X } from "lucide-react";
import { BrandMark } from "./BrandMark";
import { desktopWindow } from "../lib/desktop";
import type { Theme } from "../lib/theme";

interface TitleBarProps {
  theme: Theme;
  isMaximized: boolean;
  onToggleTheme: () => void;
  onOpenSearch: () => void;
  onToggleMaximize: () => void;
  onDesktopUnavailable: () => void;
}

function RestoreIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <rect x="2.5" y="5.5" width="8" height="8" rx="1" fill="none" stroke="currentColor" />
      <path d="M5.5 5.5V3.5a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1h-2" fill="none" stroke="currentColor" />
    </svg>
  );
}

export function TitleBar({
  theme,
  isMaximized,
  onToggleTheme,
  onOpenSearch,
  onToggleMaximize,
  onDesktopUnavailable,
}: TitleBarProps) {
  const handleMinimize = async () => {
    if (!(await desktopWindow.minimize())) onDesktopUnavailable();
  };

  const handleClose = async () => {
    if (!(await desktopWindow.close())) onDesktopUnavailable();
  };

  return (
    <header className="titlebar" data-tauri-drag-region="deep">
      <div className="titlebar-brand">
        <BrandMark className="titlebar-mark" />
        <b>Janus</b>
        <span className="titlebar-context">英语 · B1</span>
      </div>

      <div className="titlebar-status" aria-label="学习空间状态">
        <i aria-hidden="true" />
        本地学习空间
      </div>

      <div className="titlebar-actions">
        <button
          type="button"
          className="titlebar-button"
          aria-label={theme === "dark" ? "切换到浅色主题" : "切换到深色主题"}
          title={theme === "dark" ? "浅色主题" : "深色主题"}
          onClick={onToggleTheme}
        >
          {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
        </button>
        <button type="button" className="titlebar-button" aria-label="搜索和快速跳转" title="搜索  Ctrl+K" onClick={onOpenSearch}>
          <Search size={16} />
        </button>
        <div className="window-actions" aria-label="窗口控制">
          <button type="button" className="window-button" aria-label="最小化" title="最小化" onClick={handleMinimize}>
            <Minimize size={16} />
          </button>
          <button
            type="button"
            className="window-button"
            aria-label={isMaximized ? "还原窗口" : "最大化"}
            title={isMaximized ? "还原" : "最大化"}
            onClick={onToggleMaximize}
          >
            {isMaximized ? <RestoreIcon /> : <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true"><rect x="2.5" y="2.5" width="11" height="11" rx="1" fill="none" stroke="currentColor" /></svg>}
          </button>
          <button type="button" className="window-button window-close" aria-label="关闭 Janus" title="关闭" onClick={handleClose}>
            <X size={16} />
          </button>
        </div>
      </div>
    </header>
  );
}