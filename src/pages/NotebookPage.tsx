import { useCallback, useEffect, useState } from "react";
import { BookmarkCheck, Eraser, NotebookPen, RefreshCw, RotateCcw, Trash2 } from "lucide-react";
import { api, toUserMessage } from "../lib/api";
import type { MasteredWord, VocabularyEntry } from "../lib/api";

interface NotebookPageProps {
  onNotify: (message: string) => void;
}

const TABS = [
  { id: "favorites", label: "生词" },
  { id: "mastered", label: "熟词" },
] as const;
type TabId = (typeof TABS)[number]["id"];

function formatAddedAt(secs: number): string {
  const date = new Date(secs * 1000);
  const now = new Date();
  const diffDays = Math.floor((now.getTime() - date.getTime()) / 86400000);
  if (diffDays <= 0) return "今天";
  if (diffDays === 1) return "昨天";
  return `${diffDays} 天前`;
}

function formatMasteredAt(secs: number): string {
  const date = new Date(secs * 1000);
  const now = new Date();
  const diffDays = Math.floor((now.getTime() - date.getTime()) / 86400000);
  if (diffDays <= 0) return "今天加入";
  if (diffDays === 1) return "昨天加入";
  return `${diffDays} 天前加入`;
}

export function NotebookPage({ onNotify }: NotebookPageProps) {
  const [tab, setTab] = useState<TabId>("favorites");
  const [items, setItems] = useState<VocabularyEntry[]>([]);
  const [mastered, setMastered] = useState<MasteredWord[]>([]);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [entries, masteredWords] = await Promise.all([
        api.getVocabulary(),
        api.getMasteredWords(),
      ]);
      setItems(entries);
      setMastered(masteredWords);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
      setItems([]);
      setMastered([]);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleRemove = async (wordId: number) => {
    setBusyId(wordId);
    try {
      await api.removeFromVocabulary(wordId);
      setItems((current) => current.filter((item) => item.word_id !== wordId));
      onNotify("已移出生词本。");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusyId(null);
    }
  };

  const handleUnmaster = async (wordId: number) => {
    setBusyId(wordId);
    try {
      await api.unmasterWord(wordId);
      setMastered((current) => current.filter((item) => item.word_id !== wordId));
      onNotify("已移回复习队列。");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusyId(null);
    }
  };

  const handleDeleteRecord = async (wordId: number, word: string) => {
    if (!window.confirm(`确定删除「${word}」的学习记录吗？该词将重置为新词，并从生词本/熟词本中移除。`)) return;
    setBusyId(wordId);
    try {
      await api.deleteLearningRecords(wordId);
      setItems((current) => current.filter((item) => item.word_id !== wordId));
      setMastered((current) => current.filter((item) => item.word_id !== wordId));
      onNotify(`已删除「${word}」的学习记录。`);
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusyId(null);
    }
  };

  return (
    <div className="learning-page">
      <section className="page-heading learning-heading">
        <div>
          <p className="eyebrow">生词本 · M10</p>
          <h1>生词本</h1>
          <p>集中管理生词与熟词：生词可随时移除，熟词标记后不再安排复习。</p>
        </div>
        <div className="learning-heading-stats">
          <span><b>{items.length}</b> 个生词</span>
          <span><b>{mastered.length}</b> 个熟词</span>
        </div>
      </section>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      <section className="notebook-tabs" aria-label="生词本分类">
        {TABS.map((option) => (
          <button
            type="button"
            key={option.id}
            className={`notebook-tab ${tab === option.id ? "is-active" : ""}`}
            onClick={() => setTab(option.id)}
          >
            {option.label}
            <span className="notebook-tab-count">
              {option.id === "favorites" ? items.length : mastered.length}
            </span>
          </button>
        ))}
      </section>

      {tab === "favorites" ? (
        items.length === 0 ? (
          <div className="panel learning-empty">
            <NotebookPen size={22} />
            <h3>生词本是空的</h3>
            <p>{notice ? "数据暂不可用。" : "在单词复习中点击「加入生词本」，生词会出现在这里。"}</p>
            <button type="button" className="button-secondary" onClick={() => void refresh()}>
              <RefreshCw size={14} /> 重新加载
            </button>
          </div>
        ) : (
          <section className="vocabulary-list" aria-label="生词列表">
            {items.map((entry) => (
              <article className="vocabulary-row" key={entry.word_id}>
                <div className="vocabulary-copy">
                  <b>{entry.word}</b>
                  <span>{entry.phonetic}</span>
                  <p>{entry.meaning}</p>
                </div>
                <span className="vocabulary-added">{formatAddedAt(entry.added_at)}</span>
                <button
                  type="button"
                  className="icon-button"
                  onClick={() => void handleRemove(entry.word_id)}
                  disabled={busyId === entry.word_id}
                  aria-label={`移除 ${entry.word}`}
                  title="移出生词本"
                >
                  <Trash2 size={15} />
                </button>
                <button
                  type="button"
                  className="icon-button"
                  onClick={() => void handleDeleteRecord(entry.word_id, entry.word)}
                  disabled={busyId === entry.word_id}
                  aria-label={`删除 ${entry.word} 的学习记录`}
                  title="删除学习记录"
                >
                  <Eraser size={15} />
                </button>
              </article>
            ))}
          </section>
        )
      ) : mastered.length === 0 ? (
        <div className="panel learning-empty">
          <BookmarkCheck size={22} />
          <h3>熟词本是空的</h3>
          <p>{notice ? "数据暂不可用。" : "在单词复习中点击「熟悉」，单词会进入熟词本并永不安排复习。"}</p>
          <button type="button" className="button-secondary" onClick={() => void refresh()}>
            <RefreshCw size={14} /> 重新加载
          </button>
        </div>
      ) : (
        <section className="vocabulary-list" aria-label="熟词列表">
          {mastered.map((entry) => (
            <article className="vocabulary-row" key={entry.word_id}>
              <div className="vocabulary-copy">
                <b>{entry.word}</b>
                <span>{entry.phonetic}</span>
                <p>{entry.meaning}</p>
                <span className="vocabulary-difficulty">
                  难度系数 {entry.difficulty.toFixed(2)}
                </span>
              </div>
              <span className="vocabulary-added">
                {entry.mastered_at ? formatMasteredAt(entry.mastered_at) : "已加入"}
              </span>
              <button
                type="button"
                className="icon-button"
                onClick={() => void handleUnmaster(entry.word_id)}
                disabled={busyId === entry.word_id}
                aria-label={`将 ${entry.word} 移回复习`}
                title="移回复习"
              >
                <RotateCcw size={15} />
              </button>
              <button
                type="button"
                className="icon-button"
                onClick={() => void handleDeleteRecord(entry.word_id, entry.word)}
                disabled={busyId === entry.word_id}
                aria-label={`删除 ${entry.word} 的学习记录`}
                title="删除学习记录"
              >
                <Eraser size={15} />
              </button>
            </article>
          ))}
        </section>
      )}
    </div>
  );
}
