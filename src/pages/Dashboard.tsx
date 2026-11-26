import { useCallback, useEffect, useState } from "react";
import { ArrowRight, BookOpen, Headphones, Mic, SlidersHorizontal, Sparkles } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Waveform } from "../components/Waveform";
import type { NavId } from "../data/navigation";
import { api, emptyLearningConfig, emptyTodayProgress, toUserMessage } from "../lib/api";
import type { DailyStat, LearningConfig, TodayProgress } from "../lib/api";

interface DashboardProps {
  onNavigate: (id: NavId) => void;
  onNotify: (message: string) => void;
}

interface FocusCard {
  id: NavId;
  icon: LucideIcon;
  title: string;
  detail: string;
  badge: string;
  badgeTone?: "due" | "live";
  metric: string;
  metricUnit: string;
  progress: number;
}

function weekdayLabel(dayStart: number): string {
  const date = new Date(dayStart * 1000);
  const labels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return labels[date.getDay()] ?? "";
}

function GoalRing({ progress, dailyTotal }: { progress: TodayProgress; dailyTotal: number }) {
  const percent = dailyTotal > 0 ? Math.min(100, Math.round((progress.word_reviews / dailyTotal) * 100)) : 0;
  return (
    <div className="goal">
      <div className="goal-ring" role="img" aria-label={`今日单词完成 ${percent}%`}>
        <b>{percent}%</b>
      </div>
      <div className="goal-copy">
        <strong>今日单词进度</strong>
        已复习 {progress.word_reviews} / {dailyTotal} 个
      </div>
    </div>
  );
}

function LearningChart({ stats, streak }: { stats: DailyStat[]; streak: number }) {
  const values = stats.map((day) => day.duration_min);
  const max = Math.max(24, ...values);
  const avg = Math.round(values.reduce((sum, value) => sum + value, 0) / Math.max(1, values.length));
  const accuracies = stats.filter((day) => day.accuracy_percent > 0);
  const avgAccuracy = accuracies.length
    ? Math.round(accuracies.reduce((sum, day) => sum + day.accuracy_percent, 0) / accuracies.length)
    : 0;
  const width = 560;
  const height = 96;
  const gap = 12;
  const barWidth = 44;
  const bars = stats.map((day, index) => {
    const barHeight = Math.max(4, Math.round((day.duration_min / max) * (height - 28)));
    const x = index * (barWidth + gap) + 6;
    return { x, barHeight, y: height - barHeight, label: weekdayLabel(day.day_start) };
  });
  return (
    <section className="panel chart-panel" aria-labelledby="chart-title">
      <div className="panel-heading">
        <b id="chart-title">本周学习节奏</b>
        <span>min / day</span>
      </div>
      <div className="chart-body">
        <div className="chart">
          <svg viewBox={`0 0 ${width} ${height}`} preserveAspectRatio="none" role="img" aria-label="本周每日学习时长柱状图">
            {bars.map((bar) => (
              <g key={bar.label}>
                <rect x={bar.x} y={bar.y} width={barWidth} height={bar.barHeight} rx="5" fill="var(--accent)" opacity="0.75" />
                <rect x={bar.x} y={bar.y} width={barWidth} height={bar.barHeight} rx="5" fill="none" stroke="var(--accent)" strokeWidth="1" />
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
          <div><span>平均每日</span><b>{avg}<small>min</small></b></div>
          <div><span>复习正确率</span><b>{avgAccuracy}<small>%</small></b></div>
          <div><span>连续天数</span><b>{String(streak)}<small>days</small></b></div>
        </div>
      </div>
    </section>
  );
}

export function Dashboard({ onNavigate, onNotify }: DashboardProps) {
  const [progress, setProgress] = useState<TodayProgress>(emptyTodayProgress);
  const [learningConfig, setLearningConfig] = useState<LearningConfig>(emptyLearningConfig);
  const [stats, setStats] = useState<DailyStat[]>([]);
  const [reviewQueue, setReviewQueue] = useState<
    Array<{ word: string; meaning: string; interval: string; tone: string }>
  >([]);
  const [loaded, setLoaded] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [today, weekly, overview, learning] = await Promise.all([
        api.getTodayProgress(),
        api.getWeeklyStats(7),
        api.getOverview(),
        api.getLearningConfig(),
      ]);
      setProgress(today);
      setLearningConfig(learning);
      setStats(weekly);
      setReviewQueue(
        overview.review_queue.slice(0, 5).map((card) => ({
          word: card.word,
          meaning: card.meaning,
          interval: card.interval_days > 0 ? `${card.interval_days} 天` : "今天内",
          tone: card.stage === "mastered" ? "green" : card.stage === "reviewing" ? "gold" : "iris",
        })),
      );
      setLoaded(true);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
      setLoaded(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const dailyTotal = learningConfig.daily_total;
  const focusCards: FocusCard[] = [
    {
      id: "words",
      icon: BookOpen,
      title: "单词复习",
      detail: dailyTotal > 0 ? `今日目标 ${dailyTotal} 个 · 已复习 ${progress.word_reviews} 个` : "今日单词练习",
      badge: progress.due_words > 0 ? "DUE" : "DONE",
      badgeTone: progress.due_words > 0 ? "due" : undefined,
      metric: String(progress.due_words),
      metricUnit: "due",
      progress: dailyTotal > 0 ? Math.min(100, Math.round((progress.word_reviews / dailyTotal) * 100)) : 0,
    },
    {
      id: "conversation",
      icon: Sparkles,
      title: "AI 对话",
      detail: `今日已完成 ${progress.conversation_turns} 轮`,
      badge: "LIVE",
      badgeTone: "live",
      metric: String(progress.conversation_turns),
      metricUnit: "turns",
      progress: 0,
    },
    {
      id: "listening",
      icon: Headphones,
      title: "精听片段",
      detail: `今日已完成 ${progress.dictations} 段`,
      badge: "LISTEN",
      metric: String(progress.dictations),
      metricUnit: "clips",
      progress: 0,
    },
    {
      id: "speaking",
      icon: Mic,
      title: "口语校准",
      detail: `今日已完成 ${progress.speaking} 次`,
      badge: "DRILL",
      metric: String(progress.speaking),
      metricUnit: "reads",
      progress: 0,
    },
  ];

  return (
    <div className="dashboard">
      <section className="page-heading">
        <div>
          <p className="eyebrow">
            连续学习 {progress.streak_days} 天 · {weekdayLabel(Math.floor(Date.now() / 1000))}
          </p>
          <h1>今天，向前一步。</h1>
          <p>
            {progress.due_words > 0
              ? `从 ${progress.due_words} 个到期单词开始，保持轻一点，但不要断。`
              : "今天的到期单词已清空，可以复习句子或精听一段材料。"}
          </p>
        </div>
        <GoalRing progress={progress} dailyTotal={dailyTotal} />
      </section>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      <section className="today-route" aria-labelledby="route-title">
        <div className="route-copy">
          <div className="route-tag"><i aria-hidden="true" />今日路线</div>
          <h2 id="route-title">先复习，再开口，最后对话。</h2>
          <p>
            {dailyTotal > 0
              ? `今日单词目标 ${dailyTotal} 个：先复习到期单词，剩余额度用于新学。`
              : "今日无单词目标，可自由安排学习内容。"}
          </p>
          <div className="route-actions">
            <button type="button" className="button-primary" onClick={() => onNavigate("words")}>
              继续学习 <ArrowRight size={15} />
            </button>
            <button type="button" className="button-secondary" onClick={() => onNavigate("settings")}>
              <SlidersHorizontal size={14} /> 调整每日数量
            </button>
            <span className="route-meta">
              <i aria-hidden="true" />今日 {progress.duration_min} 分钟
            </span>
          </div>
        </div>
        <div className="route-visual">
          <div className="focus-phrase">
            <span className="focus-label">今日进展</span>
            <span className="focus-en">{loaded ? `${progress.word_reviews} 词 · ${progress.dictations} 听 · ${progress.speaking} 读` : "加载中…"}</span>
            <span className="focus-zh">{loaded ? `句子 ${progress.sentences} · 对话 ${progress.conversation_turns} 轮` : "数据仅在桌面应用中可用"}</span>
          </div>
          <Waveform />
        </div>
      </section>

      <section className="focus-grid" aria-label="学习模块">
        {focusCards.map((card) => {
          const Icon = card.icon;
          return (
            <button type="button" className="focus-card" key={card.id} onClick={() => onNavigate(card.id)}>
              <div className="focus-card-head">
                <span className="focus-icon"><Icon size={15} /></span>
                <span className={`status-pill ${card.badgeTone ?? ""}`}>{card.badge}</span>
              </div>
              <h3>{card.title}</h3>
              <p>{card.detail}</p>
              <div className="mini-progress" aria-hidden="true"><i style={{ width: `${card.progress}%` }} /></div>
              <div className="card-metric"><b>{card.metric}</b><span>{card.metricUnit}</span></div>
            </button>
          );
        })}
      </section>

      <div className="dashboard-lower">
        <LearningChart stats={stats} streak={progress.streak_days} />
        <section className="panel review-panel" aria-labelledby="review-title">
          <div className="panel-heading">
            <b id="review-title">正在进入长期记忆</b>
            <span>review queue</span>
          </div>
          {reviewQueue.length === 0 ? (
            <div className="review-empty">
              <p>{loaded ? "当前没有到期复习。去学习新内容吧。" : "复习队列数据仅在桌面应用中可用。"}</p>
            </div>
          ) : (
            <div className="review-list">
              {reviewQueue.map((item) => (
                <div className="review-row" key={item.word}>
                  <div><span className="review-word">{item.word}</span><span className="review-meaning">{item.meaning}</span></div>
                  <span className="review-interval">{item.interval}</span>
                  <i className={`review-dot ${item.tone}`} aria-hidden="true" />
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
