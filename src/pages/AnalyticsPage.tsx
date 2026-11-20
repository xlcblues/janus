import { useCallback, useEffect, useState } from "react";
import { ChartNoAxesCombined, Download, RefreshCw, Trash2 } from "lucide-react";
import { api, emptyStats, toUserMessage } from "../lib/api";
import type { StudyStats } from "../lib/api";

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

export function AnalyticsPage({ onNotify }: AnalyticsPageProps) {
  const [stats, setStats] = useState<StudyStats>(emptyStats);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setStats(await api.getStats());
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
      "将清除全部学习记录（单词状态、句子状态、生词本与学习事件），并先自动备份当前数据库。确定继续吗？",
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
          <p className="eyebrow">学习数据 · M2</p>
          <h1>学习数据</h1>
          <p>所有数字都来自本地学习事件，可导出、可清除、可解释。</p>
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

      <section className="panel analytics-note">
        <div className="boundary-title"><ChartNoAxesCombined size={18} /><b>数据说明</b></div>
        <p>统计由「单词复习」与「句子判断」两类学习事件汇总而成；清除操作会先创建数据库备份再重置状态，避免误操作造成不可恢复的丢失。</p>
      </section>
    </div>
  );
}
