import { useCallback, useEffect, useState } from "react";
import { NotebookPen, RefreshCw, Trash2 } from "lucide-react";
import { api, toUserMessage } from "../lib/api";
import type { VocabularyEntry } from "../lib/api";

interface NotebookPageProps {
  onNotify: (message: string) => void;
}

function formatAddedAt(secs: number): string {
  const date = new Date(secs * 1000);
  const now = new Date();
  const diffDays = Math.floor((now.getTime() - date.getTime()) / 86400000);
  if (diffDays <= 0) return "今天";
  if (diffDays === 1) return "昨天";
  return `${diffDays} 天前`;
}

export function NotebookPage({ onNotify }: NotebookPageProps) {
  const [items, setItems] = useState<VocabularyEntry[]>([]);
  const [busyId, setBusyId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const entries = await api.getVocabulary();
      setItems(entries);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
      setItems([]);
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

  return (
    <div className="learning-page">
      <section className="page-heading learning-heading">
        <div>
          <p className="eyebrow">生词本 · M2</p>
          <h1>生词本</h1>
          <p>集中管理复习中标记的生词，可随时移除。</p>
        </div>
        <div className="learning-heading-stats">
          <span><b>{items.length}</b> 个生词</span>
        </div>
      </section>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      {items.length === 0 ? (
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
              >
                <Trash2 size={15} />
              </button>
            </article>
          ))}
        </section>
      )}
    </div>
  );
}
