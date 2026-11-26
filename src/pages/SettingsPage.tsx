import { useCallback, useEffect, useState } from "react";
import { Bell, CloudOff, Download, Library, Mic, Save } from "lucide-react";
import {
  api,
  emptyLearningConfig,
  emptyMicrophoneSettings,
  emptyReminderSettings,
  toUserMessage,
} from "../lib/api";
import type {
  LearningConfig,
  MicrophoneSettings,
  ReminderSettings,
  UpdateCheck,
  Wordbook,
  WordbookCatalogEntry,
} from "../lib/api";
import { listMicrophoneDevices, probeMicrophoneDevices } from "../lib/recorder";
import type { MicrophoneDevice } from "../lib/recorder";

interface SettingsPageProps {
  onNotify: (message: string) => void;
}

export function SettingsPage({ onNotify }: SettingsPageProps) {
  const [reminder, setReminder] = useState<ReminderSettings>(emptyReminderSettings);
  const [mic, setMic] = useState<MicrophoneSettings>(emptyMicrophoneSettings);
  const [learningConfig, setLearningConfig] = useState<LearningConfig>(emptyLearningConfig);
  const [wordbooks, setWordbooks] = useState<Wordbook[]>([]);
  const [catalog, setCatalog] = useState<WordbookCatalogEntry[]>([]);
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
      const [nextReminder, nextMic, nextLearning, nextBooks, nextCatalog] = await Promise.all([
        api.getReminderSettings(),
        api.getMicrophoneSettings(),
        api.getLearningConfig(),
        api.getWordbooks(),
        api.getWordbookCatalog(),
      ]);
      setReminder(nextReminder);
      setMic(nextMic);
      setLearningConfig(nextLearning);
      setWordbooks(nextBooks);
      setCatalog(nextCatalog);
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

  const updateReminder = (patch: Partial<ReminderSettings>) => {
    setReminder((current) => ({ ...current, ...patch }));
  };

  const updateMic = (deviceId: string, deviceLabel: string) => {
    setMic((current) => ({ ...current, device_id: deviceId, device_label: deviceLabel }));
  };

  const updateLearning = (patch: Partial<LearningConfig>) => {
    setLearningConfig((current) => ({ ...current, ...patch }));
  };

  const toggleBook = (code: string) => {
    setLearningConfig((current) => {
      const has = current.book_codes.includes(code);
      const book_codes = has
        ? current.book_codes.filter((item) => item !== code)
        : [...current.book_codes, code];
      return { ...current, book_codes };
    });
  };

  const toggleCustomWord = (wordId: number) => {
    setLearningConfig((current) => {
      const has = current.custom_word_ids.includes(wordId);
      const custom_word_ids = has
        ? current.custom_word_ids.filter((id) => id !== wordId)
        : [...current.custom_word_ids, wordId];
      return { ...current, custom_word_ids };
    });
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
      await api.saveReminderSettings(reminder);
      await api.saveMicrophoneSettings(mic);
      await api.saveLearningConfig(learningConfig);
      onNotify("词库、每日学习数量、提醒与语音设备设置已保存。");
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
          <p>每日学习数量、复习提醒、语音设备与数据边界。所有设置仅保存在本地数据库。</p>
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
        <section className="panel settings-panel" aria-labelledby="wordbook-title">
          <div className="panel-heading">
            <Library size={16} />
            <b id="wordbook-title">词库与每日学习数量</b>
            <span>wordbook</span>
          </div>
          <div className="settings-panel-body">
            <p className="settings-hint">
              选择词书并设定每日学习总量；每天先按到期复习数分配，总量减去复习数即为今日新学数，记忆算法会按单词难度与你的评分自动安排复习天数。
            </p>
            <label className="settings-toggle">
              <input
                type="checkbox"
                checked={learningConfig.enabled}
                onChange={(event) => updateLearning({ enabled: event.target.checked })}
              />
              <span>启用词书学习范围</span>
            </label>
            <div className="wordbook-grid" role="group" aria-label="选择词书">
              {wordbooks.map((book) => (
                <label key={book.code} className="wordbook-option">
                  <input
                    type="checkbox"
                    checked={learningConfig.book_codes.includes(book.code)}
                    onChange={() => toggleBook(book.code)}
                    disabled={!learningConfig.enabled}
                  />
                  <span className="wordbook-option-title">{book.title}</span>
                  <small>{book.word_count} 词 · 难度 {book.difficulty.toFixed(1)}</small>
                </label>
              ))}
            </div>
            <div className="selection-mode-row" role="radiogroup" aria-label="选词方式">
              <span className="settings-subtitle">选词方式</span>
              <label className="settings-toggle">
                <input
                  type="radio"
                  name="selection-mode"
                  value="random"
                  checked={learningConfig.selection_mode === "random"}
                  onChange={() => updateLearning({ selection_mode: "random" })}
                />
                <span>随机选词</span>
              </label>
              <label className="settings-toggle">
                <input
                  type="radio"
                  name="selection-mode"
                  value="custom"
                  checked={learningConfig.selection_mode === "custom"}
                  onChange={() => updateLearning({ selection_mode: "custom" })}
                />
                <span>自定义选词</span>
              </label>
            </div>
            {learningConfig.selection_mode === "custom" ? (
              <div className="custom-word-panel">
                <p className="settings-hint">从所选词书中勾选要学习的单词；未勾选时回退为随机选词。</p>
                <div className="custom-word-scroll">
                  {catalog.map((entry) => (
                    <label key={entry.word_id} className="custom-word-item">
                      <input
                        type="checkbox"
                        checked={learningConfig.custom_word_ids.includes(entry.word_id)}
                        onChange={() => toggleCustomWord(entry.word_id)}
                        disabled={!learningConfig.enabled}
                      />
                      <span>{entry.word}</span>
                      <small>{entry.meaning}</small>
                    </label>
                  ))}
                  {catalog.length === 0 ? (
                    <span className="settings-hint">暂无可用单词，请先选择词书。</span>
                  ) : null}
                </div>
              </div>
            ) : null}
            <div className="settings-fields">
              <div className="field">
                <span>每日学习数量</span>
                <input
                  type="number"
                  min={0}
                  max={300}
                  value={learningConfig.daily_total}
                  onChange={(event) => updateLearning({ daily_total: Number(event.target.value) })}
                  aria-label="每日学习数量"
                />
                <small>个</small>
              </div>
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
