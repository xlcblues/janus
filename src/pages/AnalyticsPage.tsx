import { useCallback, useEffect, useState } from "react";
import {
  AlertCircle,
  ChartNoAxesCombined,
  Download,
  RefreshCw,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import {
  api,
  emptyMasteryProgress,
  emptyMistakeBook,
  emptyStats,
  toUserMessage,
} from "../lib/api";
import type { DailyStat, MasteryProgress, MistakeBook, StudyStats } from "../lib/api";

interface AnalyticsPageProps {
  onNotify: (message: string) => void;
}

const STAT_CARDS: Array<{ key: keyof StudyStats; label: string; unit: string }> = [
  { key: "total_reviews", label: "累计复习", unit: "次" },
  { key: "accuracy_percent", label: "正确率", unit: "%" },
  { key: "total_duration_min", label: "累计时长", unit: "min" },
  { key: "today_reviews", label: "今日复习", unit: "次" },
  { key: "today_duration_min", label: "今日时长", unit: "min" },
  { key: "due_words", label: "到期单词", unit: "个" },
  { key: "mastered_words", label: "已掌握", unit: "个" },
  { key: "vocabulary_count", label: "生词数", unit: "个" },
];

const STAGE_LABELS: Record<string, string> = {
  new: "新词",
  learning: "学习中",
  reviewing: "复习中",
  mastered: "已掌握",
};

function weekdayLabel(dayStart: number): string {
  const date = new Date(dayStart * 1000);
  const labels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return labels[date.getDay()] ?? "";
}

function formatCorrectionTime(secs: number): string {
  const date = new Date(secs * 1000);
  return `${date.getMonth() + 1}月${date.getDate()}日 ${String(date.getHours()).padStart(2, "0")}:${String(
    date.getMinutes(),
  ).padStart(2, "0")}`;
}

function WeeklyTrend({ stats }: { stats: DailyStat[] }) {
  const values = stats.map((day) => day.duration_min);
  const max = Math.max(20, ...values);
  const width = 560;
  const height = 96;
  const gap = 12;
  const barWidth = 44;
  const bars = stats.map((day, index) => {
    const barHeight = Math.max(4, Math.round((day.duration_min / max) * (height - 26)));
    const x = index * (barWidth + gap) + 6;
    return { x, barHeight, y: height - barHeight, label: weekdayLabel(day.day_start), day };
  });
  return (
    <section className="panel chart-panel" aria-labelledby="weekly-title">
      <div className="panel-heading">
        <b id="weekly-title">周趋势</b>
        <span>min / day · 最近 {stats.length} 天</span>
      </div>
      <div className="chart-body">
        <div className="chart">
          <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" role="img" aria-label="最近一周每日学习时长">
            {bars.map((bar) => (
              <g key={bar.label}>
                <rect x={bar.x} y={bar.y} width={barWidth} height={bar.barHeight} rx="5" fill="var(--accent)" opacity="0.7" />
                <rect x={bar.x} y={bar.y} width={barWidth} height={bar.barHeight} rx="5" fill="none" stroke="var(--accent)" />
              </g>
            ))}
          </svg>
          <div className="chart-axis" aria-hidden="true">
            {stats.map((day) => (
              <span key={day.day_start}>{weekdayLabel(day.day_start)}</span>
            ))}
          </div>
        </div>
        <div className="chart-summary">
          <div><span>复习次数</span><b>{stats.reduce((sum, day) => sum + day.reviews, 0)}<small>次</small></b></div>
          <div><span>听写</span><b>{stats.reduce((sum, day) => sum + day.dictations, 0)}<small>段</small></b></div>
          <div><span>朗读</span><b>{stats.reduce((sum, day) => sum + day.speaking, 0)}<small>次</small></b></div>
          <div><span>对话</span><b>{stats.reduce((sum, day) => sum + day.conversation_turns, 0)}<small>轮</small></b></div>
        </div>
      </div>
    </section>
  );
}

function MasterySection({ mastery }: { mastery: MasteryProgress }) {
  return (
    <section className="panel mastery-panel" aria-labelledby="mastery-title">
      <div className="panel-heading">
        <b id="mastery-title">掌握度</b>
        <span>mastery</span>
      </div>
      <div className="mastery-body">
        <div className="mastery-ring" style={{ "--mastery-pct": `${mastery.mastery_percent}%` } as React.CSSProperties} role="img" aria-label={`整体掌握度 ${mastery.mastery_percent}%`}>
          <b>{mastery.mastery_percent}%</b>
        </div>
        <div className="mastery-copy">
          <p>
            已开始学习 <b>{mastery.learned_words}</b> / {mastery.word_stages.reduce((sum, s) => sum + s.count, 0)} 个单词，
            已掌握 <b>{mastery.mastered_words}</b> 个，生词本 <b>{mastery.vocabulary_count}</b> 个。
          </p>
          <div className="mastery-bars">
            {mastery.word_stages.map((stage) => (
              <div className="mastery-row" key={stage.stage}>
                <span>{STAGE_LABELS[stage.stage] ?? stage.stage}</span>
                <div className="mini-progress" aria-hidden="true">
                  <i style={{ width: `${stage.count}%` }} />
                </div>
                <b>{stage.count}</b>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function MistakeSection({ book }: { book: MistakeBook }) {
  const isEmpty =
    book.low_score_words.length === 0 &&
    book.weak_dictation.length === 0 &&
    book.weak_speaking.length === 0 &&
    book.conversation_corrections.length === 0;
  return (
    <section className="panel mistake-panel" aria-labelledby="mistake-title">
      <div className="panel-heading">
        <b id="mistake-title">错题本</b>
        <span>mistake book</span>
      </div>
      {isEmpty ? (
        <p className="mistake-empty">还没有错题记录。继续保持，薄弱点会在这里自动汇总。</p>
      ) : (
        <div className="mistake-grid">
          {book.low_score_words.length > 0 ? (
            <div className="mistake-group">
              <h4>反复记错的单词</h4>
              {book.low_score_words.map((word) => (
                <div className="mistake-row" key={word.word_id}>
                  <b>{word.word}</b>
                  <span>{word.meaning}</span>
                  <small>错 {word.wrong_count}/{word.total_count} 次</small>
                </div>
              ))}
            </div>
          ) : null}
          {book.weak_dictation.length > 0 ? (
            <div className="mistake-group">
              <h4>听写薄弱片段</h4>
              {book.weak_dictation.map((clip) => (
                <div className="mistake-row" key={clip.clip_id}>
                  <b>{clip.title}</b>
                  <span>正确率 {clip.accuracy_percent}%</span>
                  <small>{clip.attempts} 次听写</small>
                </div>
              ))}
            </div>
          ) : null}
          {book.weak_speaking.length > 0 ? (
            <div className="mistake-group">
              <h4>朗读薄弱句段</h4>
              {book.weak_speaking.map((segment) => (
                <div className="mistake-row" key={segment.segment_id}>
                  <b>{segment.title}</b>
                  <span>准确率 {segment.accuracy_percent}%</span>
                  <small>{segment.attempts} 次朗读</small>
                </div>
              ))}
            </div>
          ) : null}
          {book.conversation_corrections.length > 0 ? (
            <div className="mistake-group">
              <h4>对话表达纠错</h4>
              {book.conversation_corrections.map((correction) => (
                <div className="mistake-row" key={correction.session_id}>
                  <b>{correction.original}</b>
                  <span>→ {correction.suggestion}</span>
                  <small>{correction.note} · {formatCorrectionTime(correction.created_at)}</small>
                </div>
              ))}
            </div>
          ) : null}
        </div>
      )}
    </section>
  );
}

export function AnalyticsPage({ onNotify }: AnalyticsPageProps) {
  const [stats, setStats] = useState<StudyStats>(emptyStats);
  const [weekly, setWeekly] = useState<DailyStat[]>([]);
  const [mastery, setMastery] = useState<MasteryProgress>(emptyMasteryProgress);
  const [mistakeBook, setMistakeBook] = useState<MistakeBook>(emptyMistakeBook);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const [nextStats, nextWeekly, nextMastery, nextMistake] = await Promise.all([
        api.getStats(),
        api.getWeeklyStats(7),
        api.getMasteryProgress(),
        api.getMistakeBook(),
      ]);
      setStats(nextStats);
      setWeekly(nextWeekly);
      setMastery(nextMastery);
      setMistakeBook(nextMistake);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handleExport = async () => {
    setBusy(true);
    try {
      const result = await api.exportData();
      onNotify(`数据已导出到 ${result.path}`);
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setBusy(false);
    }
  };

  const handleClear = async () => {
    const confirmed = window.confirm(
      "将清除全部学习记录（单词状态、句子状态、生词本、听写、朗读与对话记录），并先自动备份当前数据库。确定继续吗？",
    );
    if (!confirmed) return;
    setBusy(true);
    try {
      const result = await api.clearLearningData();
      onNotify(`已重置学习数据，备份保存在 ${result.backup_path}`);
      await refresh();
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
          <p className="eyebrow">学习数据 · M6</p>
          <h1>学习数据</h1>
          <p>周趋势、掌握度与错题本全部来自本地事件，可导出、可清除、可解释。</p>
        </div>
        <div className="learning-heading-actions">
          <button type="button" className="button-secondary" onClick={() => void refresh()} disabled={busy}>
            <RefreshCw size={14} /> 刷新
          </button>
          <button type="button" className="button-secondary" onClick={() => void handleExport()} disabled={busy}>
            <Download size={14} /> 导出 JSON
          </button>
          <button type="button" className="button-danger" onClick={() => void handleClear()} disabled={busy}>
            <Trash2 size={14} /> 清除学习数据
          </button>
        </div>
      </section>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      <section className="stats-grid" aria-label="学习统计">
        {STAT_CARDS.map((card) => (
          <article className="stat-card" key={card.key}>
            <span>{card.label}</span>
            <b>{String(stats[card.key])}<small>{card.unit}</small></b>
          </article>
        ))}
      </section>

      <div className="analytics-grid">
        <WeeklyTrend stats={weekly} />
        <MasterySection mastery={mastery} />
      </div>

      <MistakeSection book={mistakeBook} />

      <section className="panel analytics-note">
        <div className="boundary-title"><ShieldCheck size={18} /><b>数据说明</b></div>
        <p>
          周趋势按 UTC 日边界聚合单词/句子/听写/朗读/对话事件；掌握度由单词阶段分布推算；
          错题本只收录「重复记错 ≥ 2 次」「正确率低」与「对话纠错」三类可追溯证据。
          清除操作会先创建数据库备份再重置状态，避免误操作造成不可恢复的丢失。
        </p>
      </section>
    </div>
  );
}
