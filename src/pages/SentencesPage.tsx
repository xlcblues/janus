import { useCallback, useEffect, useState } from "react";
import { Check, Eye, EyeOff, MessagesSquare, RefreshCw, X } from "lucide-react";
import { api, toUserMessage } from "../lib/api";
import type { SentenceCard } from "../lib/api";

interface SentencesPageProps {
  onNotify: (message: string) => void;
}

function statusLabel(status: string): string {
  if (status === "reviewing") return "复习中";
  if (status === "learned") return "已掌握";
  return "新句子";
}

export function SentencesPage({ onNotify }: SentencesPageProps) {
  const [sentences, setSentences] = useState<SentenceCard[]>([]);
  const [revealed, setRevealed] = useState<Set<number>>(new Set());
  const [busyId, setBusyId] = useState<number | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const items = await api.getSentences(20);
      setSentences(items);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
      setSentences([]);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleResult = async (sentenceId: number, correct: boolean) => {
    setBusyId(sentenceId);
    try {
      const updated = await api.completeSentence(sentenceId, correct, 4000);
      setSentences((items) =>
        items.map((item) => (item.id === sentenceId ? { ...item, ...updated } : item)),
      );
      onNotify(correct ? "已记录为理解正确，继续保持。" : "已记录为未理解，稍后会再次出现。");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusyId(null);
    }
  };

  const toggleReveal = (sentenceId: number) => {
    setRevealed((current) => {
      const next = new Set(current);
      if (next.has(sentenceId)) next.delete(sentenceId);
      else next.add(sentenceId);
      return next;
    });
  };

  const mastered = sentences.filter((item) => item.status === "learned").length;
  const attempted = sentences.filter((item) => item.attempts > 0).length;

  return (
    <div className="learning-page">
      <section className="page-heading learning-heading">
        <div>
          <p className="eyebrow">句子 · M2</p>
          <h1>句子学习</h1>
          <p>先主动回忆，再核对翻译。每一次判断都会进入学习事件。</p>
        </div>
        <div className="learning-heading-stats">
          <span><b>{sentences.length}</b> 句库</span>
          <span><b>{attempted}</b> 练过</span>
          <span><b>{mastered}</b> 掌握</span>
        </div>
      </section>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      <section className="sentence-list" aria-label="句子列表">
        {sentences.length === 0 ? (
          <div className="panel learning-empty">
            <MessagesSquare size={22} />
            <h3>暂无句子</h3>
            <p>{notice ? "数据暂不可用。" : "句库尚未初始化。"}</p>
            <button type="button" className="button-secondary" onClick={() => void refresh()}>
              <RefreshCw size={14} /> 重新加载
            </button>
          </div>
        ) : (
          sentences.map((sentence) => {
            const show = revealed.has(sentence.id);
            return (
              <article className={`sentence-card ${sentence.status}`} key={sentence.id}>
                <div className="sentence-main">
                  <p className="sentence-en">{sentence.en}</p>
                  {show ? <p className="sentence-zh">{sentence.zh}</p> : <p className="sentence-hidden">翻译已隐藏，先回忆中文意思。</p>}
                </div>
                <div className="sentence-meta">
                  <span className="status-pill">{statusLabel(sentence.status)}</span>
                  <span>尝试 {sentence.attempts} · 正确 {sentence.correct_count}</span>
                  <button
                    type="button"
                    className="reveal-toggle"
                    onClick={() => toggleReveal(sentence.id)}
                  >
                    {show ? <EyeOff size={13} /> : <Eye size={13} />}
                    {show ? "隐藏翻译" : "显示翻译"}
                  </button>
                </div>
                <div className="sentence-actions">
                  <button
                    type="button"
                    className="button-secondary"
                    onClick={() => void handleResult(sentence.id, false)}
                    disabled={busyId === sentence.id}
                  >
                    <X size={14} /> 没记住
                  </button>
                  <button
                    type="button"
                    className="button-primary"
                    onClick={() => void handleResult(sentence.id, true)}
                    disabled={busyId === sentence.id}
                  >
                    <Check size={14} /> 记住了
                  </button>
                </div>
              </article>
            );
          })
        )}
      </section>
    </div>
  );
}
