import { useCallback, useEffect, useState } from "react";
import { Bell, CalendarCheck, CloudOff, Save } from "lucide-react";
import { api, emptyReminderSettings, emptyStudyPlan, toUserMessage } from "../lib/api";
import type { ReminderSettings, StudyPlan } from "../lib/api";

interface SettingsPageProps {
  onNotify: (message: string) => void;
}

type PlanNumberKey = Exclude<keyof StudyPlan, "enabled">;

const PLAN_FIELDS: Array<{
  key: PlanNumberKey;
  label: string;
  unit: string;
  hint: string;
}> = [
  { key: "word_target", label: "单词", unit: "个", hint: "每天复习的单词数量" },
  { key: "sentence_target", label: "句子", unit: "句", hint: "每天主动回忆的句子数量" },
  { key: "listening_target", label: "听力", unit: "段", hint: "每天完成的精听片段数量" },
  { key: "speaking_target", label: "朗读", unit: "次", hint: "每天完成的跟读练习次数" },
  { key: "conversation_target", label: "AI 对话", unit: "轮", hint: "每天完成的对话轮次" },
  { key: "duration_min_target", label: "时长", unit: "分钟", hint: "每天投入的学习总时长" },
];

function clampMinutes(value: number): number {
  if (Number.isNaN(value)) return 0;
  return Math.max(0, Math.min(480, Math.round(value)));
}

export function SettingsPage({ onNotify }: SettingsPageProps) {
  const [plan, setPlan] = useState<StudyPlan>(emptyStudyPlan);
  const [reminder, setReminder] = useState<ReminderSettings>(emptyReminderSettings);
  const [loaded, setLoaded] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [nextPlan, nextReminder] = await Promise.all([
        api.getStudyPlan(),
        api.getReminderSettings(),
      ]);
      setPlan(nextPlan);
      setReminder(nextReminder);
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

  const updatePlan = (key: PlanNumberKey, value: number) => {
    setPlan((current) => ({ ...current, [key]: value }));
  };

  const togglePlan = (enabled: boolean) => {
    setPlan((current) => ({ ...current, enabled }));
  };

  const updateReminder = (patch: Partial<ReminderSettings>) => {
    setReminder((current) => ({ ...current, ...patch }));
  };

  const handleSave = async () => {
    setBusy(true);
    try {
      const nextPlan = { ...plan, duration_min_target: clampMinutes(plan.duration_min_target) };
      setPlan(nextPlan);
      await api.saveStudyPlan(nextPlan);
      await api.saveReminderSettings(reminder);
      onNotify("学习计划与提醒设置已保存。");
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
          <p className="eyebrow">设置 · M6</p>
          <h1>设置</h1>
          <p>学习计划、复习提醒与数据边界。所有设置仅保存在本地数据库。</p>
        </div>
        <div className="learning-heading-actions">
          <button type="button" className="button-secondary" onClick={() => void refresh()} disabled={busy}>
            重新加载
          </button>
          <button type="button" className="button-primary" onClick={() => void handleSave()} disabled={busy || !loaded}>
            <Save size={14} /> 保存设置
          </button>
        </div>
      </section>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      <div className="settings-grid">
        <section className="panel settings-panel" aria-labelledby="plan-title">
          <div className="panel-heading">
            <CalendarCheck size={16} />
            <b id="plan-title">学习计划</b>
            <span>daily plan</span>
          </div>
          <div className="settings-panel-body">
            <p className="settings-hint">设置每天的学习目标，今日页会按这些配额计算完成度。</p>
            <label className="settings-toggle">
              <input
                type="checkbox"
                checked={plan.enabled}
                onChange={(event) => togglePlan(event.target.checked)}
              />
              <span>启用学习计划</span>
            </label>
            <div className="settings-fields">
              {PLAN_FIELDS.map((field) => (
                <div className="field" key={field.key}>
                  <span>
                    {field.label}（{field.hint}）
                  </span>
                  <input
                    type="number"
                    min={0}
                    value={plan[field.key]}
                    onChange={(event) => updatePlan(field.key, Number(event.target.value))}
                    aria-label={`${field.label}目标`}
                  />
                  <small>{field.unit}</small>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="panel settings-panel" aria-labelledby="reminder-title">
          <div className="panel-heading">
            <Bell size={16} />
            <b id="reminder-title">复习提醒</b>
            <span>reminder</span>
          </div>
          <div className="settings-panel-body">
            <p className="settings-hint">
              在设定的时间提醒你完成今日学习（桌面通知能力将在后续里程碑接入，当前仅保存时间偏好）。
            </p>
            <label className="settings-toggle">
              <input
                type="checkbox"
                checked={reminder.enabled}
                onChange={(event) => updateReminder({ enabled: event.target.checked })}
              />
              <span>启用每日提醒</span>
            </label>
            <div className="settings-time-row">
              <div className="field">
                <span>小时</span>
                <input
                  type="number"
                  min={0}
                  max={23}
                  value={reminder.hour}
                  onChange={(event) => updateReminder({ hour: Number(event.target.value) })}
                  aria-label="提醒小时"
                />
              </div>
              <div className="field">
                <span>分钟</span>
                <input
                  type="number"
                  min={0}
                  max={59}
                  value={reminder.minute}
                  onChange={(event) => updateReminder({ minute: Number(event.target.value) })}
                  aria-label="提醒分钟"
                />
              </div>
            </div>
          </div>
        </section>

        <section className="panel analytics-note" aria-labelledby="sync-title">
          <div className="boundary-title">
            <CloudOff size={18} />
            <b id="sync-title">数据边界</b>
          </div>
          <p>
            学习数据默认保存在本地，可随时在「学习数据」页导出 JSON 或清除并自动备份。
            云同步接口将在后续里程碑接入，当前不向任何服务器上传学习记录。
          </p>
        </section>
      </div>
    </div>
  );
}
