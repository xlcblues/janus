import { useCallback, useEffect, useRef, useState } from "react";
import { Bookmark, BookmarkCheck, BookOpen, RotateCcw } from "lucide-react";
import { api, PREVIEW_MESSAGE, toUserMessage } from "../lib/api";
import type { DailyQueue, LearningConfig, Overview, WordCard, Wordbook } from "../lib/api";
import type { NavId } from "../data/navigation";

const RATINGS = [
  { rating: 1, label: "重来", hint: "10 分钟后", tone: "danger" },
  { rating: 2, label: "模糊", hint: "今天再练", tone: "warning" },
  { rating: 3, label: "认识", hint: "明天再见", tone: "accent" },
  { rating: 4, label: "熟练", hint: "拉长间隔", tone: "success" },
] as const;

interface WordsPageProps {
  onNavigate: (id: NavId) => void;
  onNotify: (message: string) => void;
}

function stageLabel(stage: string): string {
  if (stage === "mastered") return "已掌握";
  if (stage === "learning") return "学习中";
  if (stage === "reviewing") return "复习中";
  return "新词";
}

function nextReviewText(card: WordCard): string {
  if (card.interval_days <= 0) return "今天内复习";
  return `${card.interval_days} 天后复习`;
}

export function WordsPage({ onNotify }: WordsPageProps) {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [dailyQueue, setDailyQueue] = useState<DailyQueue | null>(null);
  const [wordbooks, setWordbooks] = useState<Wordbook[]>([]);
  const [learningConfig, setLearningConfig] = useState<LearningConfig | null>(null);
  const [queue, setQueue] = useState<WordCard[]>([]);
  const [cursor, setCursor] = useState(0);
  const [flipped, setFlipped] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const startedAt = useRef(Date.now());

  const refresh = useCallback(async () => {
    try {
      const [nextOverview, nextQueue, nextBooks, nextConfig] = await Promise.all([
        api.getOverview(),
        api.getDailyQueue(10),
        api.getWordbooks(),
        api.getLearningConfig(),
      ]);
      setOverview(nextOverview);
      setDailyQueue(nextQueue);
      setQueue(nextQueue.cards);
      setWordbooks(nextBooks);
      setLearningConfig(nextConfig);
      setCursor(0);
      setFlipped(false);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
      setQueue([]);
      setDailyQueue(null);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const current = queue[cursor] ?? null;

  const handleRating = async (rating: number) => {
    if (!current || busy) return;
    setBusy(true);
    setFeedback(null);
    const durationMs = Date.now() - startedAt.current;
    try {
      const updated = await api.answerWord(current.id, rating, durationMs);
      setFeedback(`已记录「${RATINGS[rating - 1].label}」→ ${stageLabel(updated.stage)}，${nextReviewText(updated)}，难度系数 ${updated.difficulty.toFixed(2)}`);
      startedAt.current = Date.now();
      if (cursor + 1 < queue.length) {
        setCursor((index) => index + 1);
        setFlipped(false);
      } else {
        setQueue((items) => items.filter((item) => item.id !== current.id));
        setCursor((index) => Math.max(0, index - 1));
        setFlipped(false);
      }
      try {
        setOverview(await api.getOverview());
      } catch {
        /* 概览刷新失败不阻断复习流程 */
      }
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusy(false);
    }
  };

  const handleBookmark = async () => {
    if (!current || busy) return;
    setBusy(true);
    try {
      if (current.in_vocabulary) {
        await api.removeFromVocabulary(current.id);
        onNotify("已移出生词本。");
      } else {
        await api.addToVocabulary(current.id);
        onNotify("已加入生词本。");
      }
      setQueue((items) =>
        items.map((item) => (item.id === current.id ? { ...item, in_vocabulary: !item.in_vocabulary } : item)),
      );
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="learning-page">
      <section className="page-heading learning-heading">
        <div>
          <p className="eyebrow">单词 · M2</p>
          <h1>单词复习</h1>
          <p>间隔重复、词义与语境。按真实记忆反馈推进，不伪造评分。</p>
        </div>
        <div className="learning-heading-stats">
          {overview ? (
            <>
              <span><b>{overview.due_words}</b> 到期</span>
              <span><b>{overview.mastered_words}</b> 已掌握</span>
              <span><b>{overview.learned_words}</b> 学过</span>
            </>
          ) : null}
          {dailyQueue ? (
            <>
              <span><b>{dailyQueue.new_count}</b> 新词</span>
              <span><b>{dailyQueue.review_count}</b> 复习</span>
            </>
          ) : null}
        </div>
      </section>

      {learningConfig && wordbooks.length > 0 ? (
        <section className="daily-plan-bar" aria-label="今日词书计划">
          <span className="daily-plan-label">今日词书</span>
          {wordbooks
            .filter((book) => learningConfig.book_codes.includes(book.code))
            .map((book) => (
              <span key={book.code} className="book-chip">{book.title}</span>
            ))}
          <span className="daily-plan-meta">
            新词 {learningConfig.daily_new} · 复习 {learningConfig.daily_review} ·{" "}
            {learningConfig.selection_mode === "custom" ? "自定义选词" : "随机选词"}
          </span>
        </section>
      ) : null}

      {notice ? <div className="notice-bar">{notice}</div> : null}

      {queue.length === 0 ? (
        <section className="panel learning-empty">
          <BookOpen size={22} />
          <h3>复习队列已清空</h3>
          <p>{notice ? "数据暂不可用。" : "所有单词都已到期排期，先去学习新词或稍后再来。"}</p>
          <button type="button" className="button-secondary" onClick={() => void refresh()}>
            <RotateCcw size={14} /> 重新加载
          </button>
        </section>
      ) : current ? (
        <>
          <section className="learning-progress" aria-label="复习进度">
            <i style={{ width: `${((cursor + 1) / queue.length) * 100}%` }} />
            <span>{cursor + 1} / {queue.length}</span>
          </section>

          <section className="word-stage" aria-label="单词卡片">
            <button
              type="button"
              className={`word-card ${flipped ? "is-flipped" : ""}`}
              onClick={() => setFlipped((value) => !value)}
              disabled={busy}
              aria-label={flipped ? "显示正面" : "显示背面"}
            >
              <span className="word-card-inner">
                <span className="word-face word-front">
                  <span className="word-flag">{stageLabel(current.stage)} · {nextReviewText(current)}</span>
                  <span className="word-term">{current.word}</span>
                  <span className="word-phonetic">{current.phonetic}</span>
                  <span className="word-hint">点击查看释义</span>
                </span>
                <span className="word-face word-back">
                  <span className="word-flag">释义与例句</span>
                  <span className="word-meaning">{current.meaning}</span>
                  <span className="word-example">
                    <b>{current.example_en}</b>
                    <span>{current.example_zh}</span>
                  </span>
                  <span className="word-hint">点击回到正面</span>
                </span>
              </span>
            </button>

            <div className="word-actions">
              <button
                type="button"
                className={`bookmark-toggle ${current.in_vocabulary ? "is-active" : ""}`}
                onClick={() => void handleBookmark()}
                disabled={busy}
              >
                {current.in_vocabulary ? <BookmarkCheck size={16} /> : <Bookmark size={16} />}
                {current.in_vocabulary ? "已在生词本" : "加入生词本"}
              </button>
            </div>
          </section>

          {feedback ? <div className="feedback-line">{feedback}</div> : null}

          <section className="rating-grid" aria-label="复习评分">
            {RATINGS.map((option) => (
              <button
                type="button"
                key={option.rating}
                className={`rating-button ${option.tone}`}
                onClick={() => void handleRating(option.rating)}
                disabled={busy}
              >
                <b>{option.label}</b>
                <span>{option.hint}</span>
              </button>
            ))}
          </section>
        </>
      ) : null}
    </div>
  );
}
