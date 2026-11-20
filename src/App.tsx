import { useCallback, useEffect, useRef, useState } from "react";
import { TitleBar } from "./components/TitleBar";
import { Sidebar } from "./components/Sidebar";
import { CommandPalette } from "./components/CommandPalette";
import { Toast } from "./components/Toast";
import { Dashboard } from "./pages/Dashboard";
import { ModulePage } from "./pages/ModulePage";
import { WordsPage } from "./pages/WordsPage";
import { SentencesPage } from "./pages/SentencesPage";
import { ListeningPage } from "./pages/ListeningPage";
import { SpeakingPage } from "./pages/SpeakingPage";
import { ConversationPage } from "./pages/ConversationPage";
import { NotebookPage } from "./pages/NotebookPage";
import { AnalyticsPage } from "./pages/AnalyticsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { navItems } from "./data/navigation";
import type { NavId } from "./data/navigation";
import { applyTheme, getInitialTheme, nextTheme } from "./lib/theme";
import type { Theme } from "./lib/theme";
import { desktopWindow, isTauriRuntime } from "./lib/desktop";

export default function App() {
  const [theme, setTheme] = useState<Theme>(getInitialTheme);
  const [activeId, setActiveId] = useState<NavId>("today");
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const contentRef = useRef<HTMLElement>(null);

  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  useEffect(() => {
    if (!toast) return;
    const timeout = window.setTimeout(() => setToast(null), 2800);
    return () => window.clearTimeout(timeout);
  }, [toast]);

  const navigate = useCallback((id: NavId) => {
    setActiveId(id);
    contentRef.current?.scrollTo({ top: 0 });
  }, []);

  useEffect(() => {
    const handleKeyboard = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key.toLocaleLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
        return;
      }
      if (event.ctrlKey && /^[1-6]$/.test(event.key)) {
        const item = navItems[Number(event.key) - 1];
        if (item) {
          event.preventDefault();
          navigate(item.id);
        }
      }
    };
    window.addEventListener("keydown", handleKeyboard);
    return () => window.removeEventListener("keydown", handleKeyboard);
  }, [navigate]);

  useEffect(() => {
    if (!isTauriRuntime) return;
    let timeoutId = 0;
    const readState = () => {
      window.clearTimeout(timeoutId);
      timeoutId = window.setTimeout(() => {
        void desktopWindow.isMaximized().then((value) => {
          if (value !== null) setIsMaximized(value);
        });
      }, 80);
    };
    readState();
    window.addEventListener("resize", readState);
    return () => {
      window.clearTimeout(timeoutId);
      window.removeEventListener("resize", readState);
    };
  }, []);



  const handleToggleMaximize = async () => {
    const value = await desktopWindow.toggleMaximize();
    if (value === null) {
      setToast("桌面窗口控制仅在 Tauri 窗口中可用。");
      return;
    }
    setIsMaximized(value);
  };

  const activeItem = navItems.find((item) => item.id === activeId) ?? navItems[0];

  return (
    <div className="app-window">
      <div className="ambient" aria-hidden="true" />
      <TitleBar
        theme={theme}
        isMaximized={isMaximized}
        onToggleTheme={() => setTheme((current) => nextTheme(current))}
        onOpenSearch={() => setPaletteOpen(true)}
        onToggleMaximize={handleToggleMaximize}
        onDesktopUnavailable={() => setToast("桌面窗口控制仅在 Tauri 窗口中可用。")}
      />

      <div className="app-body">
        <Sidebar activeId={activeId} onNavigate={navigate} />
        <main className="content" ref={contentRef}>
          {activeId === "today" ? (
            <Dashboard onNavigate={navigate} onNotify={setToast} />
          ) : activeId === "words" ? (
            <WordsPage onNavigate={navigate} onNotify={setToast} />
          ) : activeId === "sentences" ? (
            <SentencesPage onNotify={setToast} />
          ) : activeId === "listening" ? (
            <ListeningPage onNavigate={navigate} onNotify={setToast} />
          ) : activeId === "speaking" ? (
            <SpeakingPage onNavigate={navigate} onNotify={setToast} />
          ) : activeId === "conversation" ? (
            <ConversationPage onNavigate={navigate} onNotify={setToast} />
          ) : activeId === "notebook" ? (
            <NotebookPage onNotify={setToast} />
          ) : activeId === "analytics" ? (
            <AnalyticsPage onNotify={setToast} />
          ) : activeId === "settings" ? (
            <SettingsPage onNotify={setToast} />
          ) : (
            <ModulePage id={activeItem.id} onNavigate={navigate} onNotify={setToast} />
          )}
        </main>
      </div>

      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} onSelect={navigate} />
      {toast ? <Toast message={toast} /> : null}
    </div>
  );
}
