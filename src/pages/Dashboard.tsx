import { ArrowRight, BookOpen, Headphones, Mic, SlidersHorizontal, Sparkles } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Waveform } from "../components/Waveform";
import type { NavId } from "../data/navigation";

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
  metric?: string;
  metricUnit?: string;
  progress?: number;
  waveform?: boolean;
}

const focusCards: FocusCard[] = [
  { id: "words", icon: BookOpen, title: "单词复习", detail: "今天到期 · 已掌握 62%", badge: "DUE", badgeTone: "due", metric: "24", metricUnit: "words" },
  { id: "conversation", icon: Sparkles, title: "AI 对话", detail: "情景 · 在咖啡馆点单", badge: "LIVE", badgeTone: "live", waveform: true },
  { id: "listening", icon: Headphones, title: "精听片段", detail: "How cities breathe · 02:14", badge: "NEW", progress: 54 },
  { id: "speaking", icon: Mic, title: "口语校准", detail: "/θ/ 与 /ð/ · 连读", badge: "DRILL", metric: "82", metricUnit: "score" },
];

const reviewQueue = [
  { word: "serendipity", meaning: "意外发现美好事物的能力", interval: "12 天", tone: "green" },
  { word: "threshold", meaning: "门槛；临界点", interval: "7 天", tone: "iris" },
  { word: "nuance", meaning: "细微差别", interval: "4 天", tone: "gold" },
  { word: "deliberate", meaning: "深思熟虑的；刻意的", interval: "明天", tone: "iris" },
];

function GoalRing() {
  return (
    <div className="goal">
      <div className="goal-ring" role="img" aria-label="今日目标完成 66%">
        <b>66%</b>
      </div>
      <div className="goal-copy">
        <strong>今日目标</strong>
        24 / 36 分钟
      </div>
    </div>
  );
}

function LearningChart() {
  return (
    <section className="panel chart-panel" aria-labelledby="chart-title">
      <div className="panel-heading">
        <b id="chart-title">本周学习节奏</b>
        <span>min / day</span>
      </div>
      <div className="chart-body">
        <div className="chart">
          <svg viewBox="0 0 560 104" preserveAspectRatio="none" role="img" aria-label="本周每日学习时长趋势，周六最高">
            <defs>
              <linearGradient id="janus-area" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0" stopColor="var(--accent)" stopOpacity=".18" />
                <stop offset="1" stopColor="var(--accent)" stopOpacity="0" />
              </linearGradient>
            </defs>
            <path d="M0 82 C35 78 46 62 78 66 C112 70 128 38 160 44 C195 51 210 68 242 62 C280 54 291 27 326 33 C356 39 375 49 404 43 C438 36 458 14 489 20 C520 26 535 35 560 30 L560 104 L0 104Z" fill="url(#janus-area)" />
            <path d="M0 82 C35 78 46 62 78 66 C112 70 128 38 160 44 C195 51 210 68 242 62 C280 54 291 27 326 33 C356 39 375 49 404 43 C438 36 458 14 489 20 C520 26 535 35 560 30" fill="none" stroke="var(--accent)" strokeWidth="2" />
            <g fill="var(--accent)">
              <circle cx="0" cy="82" r="2" /><circle cx="78" cy="66" r="2" /><circle cx="160" cy="44" r="2" /><circle cx="242" cy="62" r="2" /><circle cx="326" cy="33" r="2" /><circle cx="404" cy="43" r="2" /><circle cx="489" cy="20" r="2" /><circle cx="560" cy="30" r="2" />
            </g>
            <circle cx="489" cy="20" r="5" fill="var(--gold)" />
          </svg>
          <div className="chart-axis" aria-hidden="true">
            <span>MON</span><span>TUE</span><span>WED</span><span>THU</span><span>FRI</span><span>SAT</span><span>SUN</span>
          </div>
        </div>
        <div className="chart-summary">
          <div><span>平均每日</span><b>31<small>min</small></b></div>
          <div><span>保持率</span><b>92<small>%</small></b></div>
          <div><span>连续天数</span><b>05<small>days</small></b></div>
        </div>
      </div>
    </section>
  );
}

export function Dashboard({ onNavigate, onNotify }: DashboardProps) {
  return (
    <div className="dashboard">
      <section className="page-heading">
        <div>
          <p className="eyebrow">第 12 天 · 星期四</p>
          <h1>今天，向前一步。</h1>
          <p>输入与输出各 18 分钟。保持轻一点，但不要断。</p>
        </div>
        <GoalRing />
      </section>

      <section className="today-route" aria-labelledby="route-title">
        <div className="route-copy">
          <div className="route-tag"><i aria-hidden="true" />今日路线</div>
          <h2 id="route-title">先说出口，再把它说得更准。</h2>
          <p>从 24 个到期单词开始，进入一段 8 分钟精听，最后和 AI 完成一次咖啡馆对话。</p>
          <div className="route-actions">
            <button type="button" className="button-primary" onClick={() => onNavigate("words")}>
              继续学习 <ArrowRight size={15} />
            </button>
            <button type="button" className="button-secondary" onClick={() => onNotify("学习计划编辑器将在 M2 接入。")}>
              <SlidersHorizontal size={14} /> 调整今日计划
            </button>
            <span className="route-meta"><i aria-hidden="true" />约 18 分钟</span>
          </div>
        </div>
        <div className="route-visual">
          <div className="focus-phrase">
            <span className="focus-label">重点表达</span>
            <span className="focus-en">“It depends on how you look at it.”</span>
            <span className="focus-zh">取决于你从哪个角度看。</span>
          </div>
          <Waveform />
        </div>
      </section>

      <section className="focus-grid" aria-label="学习模块">
        {focusCards.map((card) => {
          const Icon = card.icon;
          return (
            <button type="button" className="focus-card" data-feature={card.waveform || undefined} key={card.id} onClick={() => onNavigate(card.id)}>
              <div className="focus-card-head">
                <span className="focus-icon"><Icon size={15} /></span>
                <span className={`status-pill ${card.badgeTone ?? ""}`}>{card.badge}</span>
              </div>
              <h3>{card.title}</h3>
              <p>{card.detail}</p>
              {card.waveform ? <Waveform compact bars={12} label="AI 对话语音活动" /> : null}
              {card.progress ? <div className="mini-progress" aria-hidden="true"><i style={{ width: `${card.progress}%` }} /></div> : null}
              {card.metric ? <div className="card-metric"><b>{card.metric}</b><span>{card.metricUnit}</span></div> : null}
            </button>
          );
        })}
      </section>

      <div className="dashboard-lower">
        <LearningChart />
        <section className="panel review-panel" aria-labelledby="review-title">
          <div className="panel-heading">
            <b id="review-title">正在进入长期记忆</b>
            <span>review queue</span>
          </div>
          <div className="review-list">
            {reviewQueue.map((item) => (
              <div className="review-row" key={item.word}>
                <div><span className="review-word">{item.word}</span><span className="review-meaning">{item.meaning}</span></div>
                <span className="review-interval">{item.interval}</span>
                <i className={`review-dot ${item.tone}`} aria-hidden="true" />
              </div>
            ))}
          </div>
        </section>
      </div>
    </div>
  );
}
