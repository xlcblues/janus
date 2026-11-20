import { useCallback, useEffect, useState } from "react";
import { Bell, CalendarCheck, CloudOff, Download, Mic, Save } from "lucide-react";
import {
  api,
  emptyMicrophoneSettings,
  emptyReminderSettings,
  emptyStudyPlan,
  toUserMessage,
} from "../lib/api";
import type { MicrophoneSettings, ReminderSettings, StudyPlan, UpdateCheck } from "../lib/api";
import { listMicrophoneDevices, probeMicrophoneDevices } from "../lib/recorder";
import type { MicrophoneDevice } from "../lib/recorder";

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
  const [mic, setMic] = useState<MicrophoneSettings>(emptyMicrophoneSettings);
  const [devices, setDevices] = useState<MicrophoneDevice[]>([]);
  const [notificationState, setNotificationState] = useState<"granted" | "denied" | "prompt" | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [busy, setBusy] = useState(false);
  const [testingNotification, setTestingNotification] = useState(false);
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [appDataDir, setAppDataDir] = useState<string | null>(null);
  const [updateCheck, setUpdateCheck] = useState<UpdateCheck | null>(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [nextPlan, nextReminder, nextMic] = await Promise.all([
        api.getStudyPlan(),
        api.getReminderSettings(),
        api.getMicrophoneSettings(),
      ]);
      setPlan(nextPlan);
      setReminder(nextReminder);
      setMic(nextMic);
      setLoaded(true);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
      setLoaded(false);
    }
  }, []);

  const refreshDevices = useCallback(async () => {
    const nextDevices = await listMicrophoneDevices();
    setDevices(nextDevices);
  }, []);

  useEffect(() => {
    void refreshDevices();
  }, [refreshDevices]);

  useEffect(() => {
    api
      .getNotificationPermission()
      .then((state) => setNotificationState(state))
      .catch(() => setNotificationState(null));
  }, []);

  useEffect(() => {
    Promise.all([api.getAppVersion(), api.getAppDataDir()])
      .then(([version, dataDir]) => {
        setAppVersion(version);
        setAppDataDir(dataDir);
      })
      .catch(() => {
        setAppVersion(null);
        setAppDataDir(null);
      });
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

  const updateMic = (deviceId: string, deviceLabel: string) => {
    setMic((current) => ({ ...current, device_id: deviceId, device_label: deviceLabel }));
  };

  const handleProbeMicrophones = async () => {
    setNotice(null);
    const nextDevices = await probeMicrophoneDevices();
    setDevices(nextDevices);
    if (nextDevices.length === 0) {
      setNotice("未检测到可用的麦克风设备，请检查系统录音设置。");
    } else if (mic.device_id && !nextDevices.some((device) => device.deviceId === mic.device_id)) {
      setNotice("已保存的麦克风当前不可用，请重新选择输入设备。");
    }
  };

  const handleTestNotification = async () => {
    setTestingNotification(true);
    try {
      await api.sendTestNotification();
      onNotify("测试通知已发送，请查看系统通知中心。");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setTestingNotification(false);
    }
  };

  const handleCheckUpdate = async () => {
    setCheckingUpdate(true);
    setUpdateError(null);
    setUpdateCheck(null);
    try {
      const result = await api.checkForUpdate();
      setUpdateCheck(result);
    } catch (error) {
      setUpdateError(toUserMessage(error));
    } finally {
      setCheckingUpdate(false);
    }
  };

  const handleOpenDownload = () => {
    if (!updateCheck?.download_url) return;
    api.openUrl(updateCheck.download_url).catch((error) => setUpdateError(toUserMessage(error)));
  };

  const handleSave = async () => {
    setBusy(true);
    try {
      const nextPlan = { ...plan, duration_min_target: clampMinutes(plan.duration_min_target) };
      setPlan(nextPlan);
      await api.saveStudyPlan(nextPlan);
      await api.saveReminderSettings(reminder);
      await api.saveMicrophoneSettings(mic);
      onNotify("学习计划、提醒与语音设备设置已保存。");
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
          <p className="eyebrow">设置 · M8</p>
          <h1>设置</h1>
          <p>学习计划、复习提醒、语音设备与数据边界。所有设置仅保存在本地数据库。</p>
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
              在设定的时间通过 Windows 桌面通知提醒你完成今日学习（应用运行时生效）。
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
            <div className="reminder-status-row">
              <span className="reminder-status-label">
                桌面通知：
                {notificationState === "granted"
                  ? "已就绪"
                  : notificationState === "denied"
                    ? "被系统拒绝，请到系统设置开启"
                    : notificationState === "prompt"
                      ? "等待授权"
                      : "仅桌面应用可用"}
              </span>
              <button
                type="button"
                className="button-secondary"
                onClick={() => void handleTestNotification()}
                disabled={testingNotification || !loaded}
              >
                {testingNotification ? "发送中…" : "发送测试通知"}
              </button>
            </div>
          </div>
        </section>

        <section className="panel settings-panel" aria-labelledby="mic-title">
          <div className="panel-heading">
            <Mic size={16} />
            <b id="mic-title">语音设备</b>
            <span>microphone</span>
          </div>
          <div className="settings-panel-body">
            <p className="settings-hint">
              选择朗读与 AI 对话使用的麦克风；不选择时使用系统默认设备。
            </p>
            <div className="mic-select-row">
              <label htmlFor="mic-select">输入设备</label>
              <select
                id="mic-select"
                value={mic.device_id}
                onChange={(event) => {
                  const selected = devices.find((device) => device.deviceId === event.target.value);
                  updateMic(event.target.value, selected?.label ?? "");
                }}
              >
                <option value="">系统默认麦克风</option>
                {mic.device_id && !devices.some((device) => device.deviceId === mic.device_id) ? (
                  <option value={mic.device_id}>
                    {mic.device_label || "已保存设备"}（当前不可用）
                  </option>
                ) : null}
                {devices.map((device) => (
                  <option key={device.deviceId} value={device.deviceId}>
                    {device.label || "未命名麦克风"}
                  </option>
                ))}
              </select>
              <button
                type="button"
                className="button-secondary"
                onClick={() => void handleProbeMicrophones()}
              >
                检测设备
              </button>
            </div>
            <p className="settings-mic-hint">
              {devices.length > 0
                ? `已检测到 ${devices.length} 个输入设备。`
                : "尚未检测到输入设备；点击「检测设备」授权后可查看具体名称。"}
            </p>
          </div>
        </section>

        <section className="panel settings-panel" aria-labelledby="update-title">
          <div className="panel-heading">
            <Download size={16} />
            <b id="update-title">版本与更新</b>
            <span>release</span>
          </div>
          <div className="settings-panel-body">
            <p className="settings-hint">查看当前版本与数据目录；有新版时可跳转 GitHub Releases 下载。</p>
            <div className="settings-about-row">
              <span className="settings-about-label">应用版本</span>
              <span className="settings-about-value">{appVersion ?? "仅桌面应用可用"}</span>
            </div>
            <div className="settings-about-row">
              <span className="settings-about-label">数据目录</span>
              <span className="settings-about-value settings-about-path" title={appDataDir ?? undefined}>
                {appDataDir ?? "仅桌面应用可用"}
              </span>
            </div>
            <div className="update-action-row">
              <button
                type="button"
                className="button-secondary"
                onClick={() => void handleCheckUpdate()}
                disabled={checkingUpdate}
              >
                {checkingUpdate ? "检查中…" : "检查更新"}
              </button>
              {updateCheck ? (
                updateCheck.has_update ? (
                  <button type="button" className="button-primary" onClick={() => void handleOpenDownload()}>
                    前往下载 v{updateCheck.latest}
                  </button>
                ) : (
                  <span className="settings-update-ok">已是最新版本 v{updateCheck.latest}</span>
                )
              ) : null}
            </div>
            {updateCheck?.published_at ? (
              <p className="settings-mic-hint">
                最新版本发布于 {new Date(updateCheck.published_at).toLocaleDateString("zh-CN")}。
              </p>
            ) : null}
            {updateError ? <p className="settings-mic-hint">{updateError}</p> : null}
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
