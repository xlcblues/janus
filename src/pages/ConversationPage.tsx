import { useCallback, useEffect, useRef, useState } from "react";
import {
  AlertCircle,
  ChevronLeft,
  Loader2,
  MessageSquare,
  Mic,
  Play,
  Send,
  Settings,
  Sparkles,
  Square,
  StopCircle,
  Wand2,
} from "lucide-react";
import { api, toUserMessage } from "../lib/api";
import type {
  ConversationMessage,
  ConversationScene,
  ConversationSessionSummary,
  ConversationSettings,
  ConversationStats,
  Correction,
} from "../lib/api";
import { emptyConversationSettings, emptyConversationStats, PREVIEW_MESSAGE } from "../lib/api";
import { isTauriRuntime } from "../lib/desktop";
import { useRecorder } from "../lib/recorder";
import type { NavId } from "../data/navigation";

interface ConversationPageProps {
  onNavigate: (id: NavId) => void;
  onNotify: (message: string) => void;
}

function formatWhen(secs: number): string {
  const date = new Date(secs * 1000);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  }
  return date.toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" });
}

/** 从消息里解析已存储的纠错 JSON；"{}" 或无纠错时返回 null。 */
function parseStoredCorrection(message: ConversationMessage): Correction | null {
  if (!message.correction) return null;
  try {
    const parsed = JSON.parse(message.correction) as Correction;
    if (parsed.original || parsed.suggestion || parsed.note) return parsed;
  } catch {
    // 忽略格式异常的本地记录
  }
  return null;
}

export function ConversationPage({ onNavigate, onNotify }: ConversationPageProps) {
  const [scenes, setScenes] = useState<ConversationScene[]>([]);
  const [sessions, setSessions] = useState<ConversationSessionSummary[]>([]);
  const [stats, setStats] = useState<ConversationStats>(emptyConversationStats);
  const [settings, setSettings] = useState<ConversationSettings>(emptyConversationSettings);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<string | null>(null);

  // 进行中的会话
  const [session, setSession] = useState<ConversationSessionSummary | null>(null);
  const [activeScene, setActiveScene] = useState<ConversationScene | null>(null);
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [pendingCorrections, setPendingCorrections] = useState<number[]>([]);
  const chatEndRef = useRef<HTMLDivElement | null>(null);

  // 录音与转写（优先使用设置页选择的麦克风）
  const [micDeviceId, setMicDeviceId] = useState<string>("");
  useEffect(() => {
    if (!isTauriRuntime) return;
    api
      .getMicrophoneSettings()
      .then((settings) => setMicDeviceId(settings.device_id))
      .catch(() => setMicDeviceId(""));
  }, []);
  const recorder = useRecorder(micDeviceId);
  const [transcribing, setTranscribing] = useState(false);
  const playbackRef = useRef<HTMLAudioElement | null>(null);
  const [playbackUrl, setPlaybackUrl] = useState<string | null>(null);

  // 设置面板
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [draftSettings, setDraftSettings] = useState<ConversationSettings>(emptyConversationSettings);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [savingSettings, setSavingSettings] = useState(false);

  const configured = Boolean(
    settings.base_url.trim() && settings.api_key.trim() && settings.model.trim(),
  );

  const refreshData = useCallback(async () => {
    if (!isTauriRuntime) {
      setLoading(false);
      return;
    }
    try {
      const [nextScenes, nextSessions, nextStats, nextSettings] = await Promise.all([
        api.getConversationScenes(),
        api.getConversationSessions(),
        api.getConversationStats(),
        api.getConversationSettings(),
      ]);
      setScenes(nextScenes);
      setSessions(nextSessions);
      setStats(nextStats);
      setSettings(nextSettings);
      setDraftSettings(nextSettings);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshData();
  }, [refreshData]);

  // 新消息到达时自动滚到底部
  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  }, [messages, pendingCorrections]);

  // 录音回放 URL 生命周期
  useEffect(() => {
    return () => {
      if (playbackUrl) URL.revokeObjectURL(playbackUrl);
    };
  }, [playbackUrl]);

  // 卸载时释放麦克风
  useEffect(() => {
    return () => recorder.cancel();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleStartScene = async (sceneId: number) => {
    setSending(true);
    setNotice(null);
    try {
      const result = await api.startConversation(sceneId);
      setSession(result.session);
      setActiveScene(scenes.find((item) => item.id === sceneId) ?? null);
      setMessages([result.greeting]);
      setDraft("");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setSending(false);
    }
  };

  const handleOpenSession = async (sessionId: number) => {
    setSending(true);
    setNotice(null);
    try {
      const [history, nextSessions] = await Promise.all([
        api.getConversationHistory(sessionId),
        api.getConversationSessions(),
      ]);
      const target = nextSessions.find((item) => item.id === sessionId) ?? null;
      setSessions(nextSessions);
      setSession(target);
      setActiveScene(target ? scenes.find((item) => item.id === target.scene_id) ?? null : null);
      setMessages(history);
      setDraft("");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setSending(false);
    }
  };

  const handleEndSession = async () => {
    if (!session) return;
    setSending(true);
    try {
      const ended = await api.endConversation(session.id);
      setNotice(`会话已结束：${ended.summary}`);
      setSession(null);
      setActiveScene(null);
      setMessages([]);
      await refreshData();
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setSending(false);
    }
  };

  const analyzeTurn = useCallback(async (userMessage: ConversationMessage) => {
    if (!session) return;
    setPendingCorrections((prev) => [...prev, userMessage.id]);
    try {
      const correction = await api.analyzeConversationCorrection(session.id, userMessage.id);
      if (correction) {
        setMessages((prev) =>
          prev.map((item) =>
            item.id === userMessage.id
              ? { ...item, correction: JSON.stringify(correction) }
              : item,
          ),
        );
      }
    } catch {
      // 纠错分析是增强功能，失败不影响对话
    } finally {
      setPendingCorrections((prev) => prev.filter((id) => id !== userMessage.id));
    }
  }, [session]);

  const sendTurn = async (raw: string) => {
    const content = raw.trim();
    if (!content || !session || sending) return;
    setSending(true);
    setNotice(null);
    try {
      const result = await api.sendConversationMessage(session.id, content);
      setMessages((prev) => [...prev, result.user_message, result.assistant_message]);
      setDraft("");
      setSession((prev) => (prev ? { ...prev, turn_count: result.turn_count } : prev));
      void analyzeTurn(result.user_message);
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setSending(false);
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void sendTurn(draft);
    }
  };

  const handleStartRecording = async () => {
    setNotice(null);
    try {
      await recorder.start();
    } catch (error) {
      setNotice(toUserMessage(error));
    }
  };

  const handleStopRecording = async () => {
    const result = recorder.stop();
    if (!result) return;
    if (playbackUrl) URL.revokeObjectURL(playbackUrl);
    setPlaybackUrl(URL.createObjectURL(result.wavBlob));
    setTranscribing(true);
    try {
      const transcript = await api.transcribeSpeech(Array.from(result.wavBytes));
      if (transcript.text.trim()) {
        setDraft((prev) => (prev ? `${prev} ${transcript.text}` : transcript.text));
      } else {
        setNotice("未能识别到语音，请靠近麦克风再试一次。");
      }
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setTranscribing(false);
    }
  };

  const handlePlayback = () => {
    const audio = playbackRef.current;
    if (!audio || !playbackUrl) return;
    if (!audio.paused) {
      audio.pause();
      return;
    }
    void audio.play();
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await api.testConversationConnection(draftSettings);
      setTestResult(result.message);
    } catch (error) {
      setTestResult(toUserMessage(error));
    } finally {
      setTesting(false);
    }
  };

  const handleSaveSettings = async () => {
    setSavingSettings(true);
    try {
      await api.saveConversationSettings(draftSettings);
      setSettings(draftSettings);
      setSettingsOpen(false);
      onNotify("AI 对话服务配置已保存。");
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setSavingSettings(false);
    }
  };

  if (!isTauriRuntime) {
    return (
      <div className="learning-page conversation-page">
        <header className="learning-heading page-heading">
          <div>
            <p className="eyebrow">Janus · M5</p>
            <h1>AI 对话</h1>
            <p>场景化角色扮演，语音输入 + AI 实时回复 + 表达纠错反馈。</p>
          </div>
        </header>
        <div className="listening-empty">
          <Sparkles size={22} />
          <p>{PREVIEW_MESSAGE}</p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="learning-page">
        <header className="learning-heading page-heading">
          <div>
            <p className="eyebrow">Janus · M5</p>
            <h1>AI 对话</h1>
          </div>
        </header>
        <div className="conversation-loading">
          <Loader2 size={18} className="spin" />
          <span>正在加载对话模块…</span>
        </div>
      </div>
    );
  }

  return (
    <div className="learning-page conversation-page">
      <header className="learning-heading page-heading">
        <div>
          <p className="eyebrow">Janus · M5</p>
          <h1>AI 对话</h1>
          <p>场景化角色扮演，语音输入 + AI 实时回复 + 表达纠错反馈。</p>
        </div>
        <div className="learning-heading-actions">
          <button
            type="button"
            className="button-secondary"
            onClick={() => setSettingsOpen((open) => !open)}
          >
            <Settings size={14} /> 对话设置
          </button>
        </div>
      </header>

      {notice ? <div className="notice-bar">{notice}</div> : null}

      {!configured && !notice ? (
        <div className="conversation-setup-hint">
          <AlertCircle size={15} />
          <span>尚未配置 AI 对话服务。请先填写 Base URL、API Key 与模型名称，即可开始实时对话。</span>
          <button type="button" className="button-secondary" onClick={() => setSettingsOpen(true)}>
            前往配置
          </button>
        </div>
      ) : null}

      {settingsOpen ? (
        <section className="conversation-settings panel" aria-label="AI 对话设置">
          <div className="panel-heading">
            <b>对话设置</b>
            <span>仅保存在本地数据库，不会写入前端构建产物</span>
          </div>
          <div className="conversation-settings-body">
            <label className="field">
              <span>Base URL</span>
              <input
                type="text"
                value={draftSettings.base_url}
                onChange={(event) =>
                  setDraftSettings({ ...draftSettings, base_url: event.target.value })
                }
                placeholder="https://api.openai.com/v1"
              />
            </label>
            <label className="field">
              <span>API Key</span>
              <input
                type="password"
                value={draftSettings.api_key}
                onChange={(event) =>
                  setDraftSettings({ ...draftSettings, api_key: event.target.value })
                }
                placeholder="sk-..."
              />
            </label>
            <label className="field">
              <span>模型</span>
              <input
                type="text"
                value={draftSettings.model}
                onChange={(event) =>
                  setDraftSettings({ ...draftSettings, model: event.target.value })
                }
                placeholder="gpt-4o-mini"
              />
            </label>
            <div className="conversation-settings-actions">
              <button
                type="button"
                className="button-secondary"
                disabled={testing}
                onClick={() => void handleTestConnection()}
              >
                {testing ? <Loader2 size={14} className="spin" /> : <Wand2 size={14} />} 测试连接
              </button>
              <button
                type="button"
                className="button-primary"
                disabled={savingSettings}
                onClick={() => void handleSaveSettings()}
              >
                {savingSettings ? <Loader2 size={14} className="spin" /> : null} 保存设置
              </button>
            </div>
            {testResult ? <p className="conversation-test-result">{testResult}</p> : null}
          </div>
        </section>
      ) : null}

      {session ? (
        <section className="conversation-chat panel" aria-label="AI 对话练习">
          <div className="conversation-chat-head">
            <button
              type="button"
              className="button-secondary"
              onClick={() => {
                setSession(null);
                setActiveScene(null);
                setMessages([]);
              }}
              disabled={sending}
            >
              <ChevronLeft size={14} /> 返回场景
            </button>
            <div className="conversation-chat-title">
              <h3>{activeScene?.title ?? session.scene_title}</h3>
              <p>{activeScene?.goal}</p>
            </div>
            <button
              type="button"
              className="button-secondary"
              onClick={() => void handleEndSession()}
              disabled={sending}
            >
              <StopCircle size={14} /> 结束会话
            </button>
          </div>

          <div className="conversation-stream">
            {messages.map((message) => {
              const correction = parseStoredCorrection(message);
              const analyzing = pendingCorrections.includes(message.id);
              return (
                <div
                  key={message.id}
                  className={`conversation-bubble ${message.role === "user" ? "user" : "assistant"}`}
                >
                  <div className="conversation-bubble-meta">
                    <span>{message.role === "user" ? "我" : activeScene?.title ?? "AI"}</span>
                    <time>{formatWhen(message.created_at)}</time>
                  </div>
                  <p className="conversation-bubble-text">{message.content}</p>
                  {message.role === "user" ? (
                    <>
                      {analyzing ? (
                        <div className="conversation-correction pending">
                          <Loader2 size={12} className="spin" /> 正在检查表达…
                        </div>
                      ) : null}
                      {correction ? (
                        <div className="conversation-correction">
                          <b>表达建议</b>
                          {correction.original ? (
                            <p className="conversation-correction-original">
                              原句：{correction.original}
                            </p>
                          ) : null}
                          {correction.suggestion ? (
                            <p className="conversation-correction-suggestion">
                              建议：{correction.suggestion}
                            </p>
                          ) : null}
                          {correction.note ? (
                            <p className="conversation-correction-note">{correction.note}</p>
                          ) : null}
                        </div>
                      ) : null}
                    </>
                  ) : null}
                </div>
              );
            })}
            {sending ? (
              <div className="conversation-bubble assistant">
                <div className="conversation-bubble-meta">
                  <span>{activeScene?.title ?? "AI"}</span>
                </div>
                <p className="conversation-bubble-text typing">
                  <Loader2 size={12} className="spin" /> 正在思考…
                </p>
              </div>
            ) : null}
            <div ref={chatEndRef} />
          </div>

          {configured ? (
            <div className="conversation-composer">
              {recorder.recording ? (
                <button
                  type="button"
                  className="recorder-button recording"
                  onClick={() => void handleStopRecording()}
                >
                  <Square size={16} /> 停止录音
                </button>
              ) : (
                <button
                  type="button"
                  className="recorder-button"
                  onClick={() => void handleStartRecording()}
                  disabled={transcribing || sending}
                >
                  {transcribing ? <Loader2 size={16} className="spin" /> : <Mic size={16} />} 语音输入
                </button>
              )}
              {playbackUrl ? (
                <button type="button" className="recorder-button replay" onClick={handlePlayback}>
                  <Play size={14} />
                </button>
              ) : null}
              <textarea
                className="conversation-input"
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="输入英文，或点击麦克风录音转写…"
                disabled={sending}
                rows={2}
              />
              <button
                type="button"
                className="button-primary conversation-send"
                onClick={() => void sendTurn(draft)}
                disabled={sending || !draft.trim()}
              >
                {sending ? <Loader2 size={14} className="spin" /> : <Send size={14} />} 发送
              </button>
            </div>
          ) : (
            <div className="conversation-composer-hint">
              <AlertCircle size={14} />
              <span>完成“对话设置”后即可开始语音 / 文字对话。</span>
              <button type="button" className="button-secondary" onClick={() => setSettingsOpen(true)}>
                <Settings size={14} /> 打开设置
              </button>
            </div>
          )}
          {recorder.error ? <p className="recorder-error">{recorder.error}</p> : null}
          <audio ref={playbackRef} src={playbackUrl ?? undefined} />
        </section>
      ) : (
        <>
          <section className="listening-stats" aria-label="对话统计">
            <article className="stat-card">
              <MessageSquare size={18} />
              <div>
                <span>累计会话</span>
                <b>{stats.total_sessions}</b>
              </div>
            </article>
            <article className="stat-card">
              <Sparkles size={18} />
              <div>
                <span>累计轮次</span>
                <b>{stats.total_turns}</b>
              </div>
            </article>
            <article className="stat-card">
              <MessageSquare size={18} />
              <div>
                <span>今日会话</span>
                <b>{stats.today_sessions}</b>
              </div>
            </article>
            <article className="stat-card">
              <Sparkles size={18} />
              <div>
                <span>今日轮次</span>
                <b>{stats.today_turns}</b>
              </div>
            </article>
          </section>

          <section className="conversation-scenes" aria-label="对话场景">
            <div className="catalog-head">
              <p className="eyebrow">选择场景开始对话</p>
              <p className="catalog-note">
                与角色进行 3–8 轮真实对话，AI 会给出回复并在后台检查你的表达。
              </p>
            </div>
            {scenes.length > 0 ? (
              <div className="conversation-scene-grid">
                {scenes.map((scene) => (
                  <article key={scene.id} className="conversation-scene-card">
                    <div className="conversation-scene-top">
                      <span className="stage-badge">{scene.level.toUpperCase()}</span>
                      <span className="conversation-scene-lang">
                        {scene.lang === "en" ? "English" : scene.lang}
                      </span>
                    </div>
                    <h3>{scene.title}</h3>
                    <p className="conversation-scene-goal">{scene.goal}</p>
                    <p className="conversation-scene-greeting">开场白：{scene.greeting}</p>
                    <button
                      type="button"
                      className="button-primary"
                      disabled={sending || !configured}
                      onClick={() => void handleStartScene(scene.id)}
                    >
                      {sending ? <Loader2 size={14} className="spin" /> : <Sparkles size={14} />} 开始对话
                    </button>
                  </article>
                ))}
              </div>
            ) : (
              !notice && (
                <div className="learning-empty">
                  <Sparkles size={28} />
                  <h3>暂无可用场景</h3>
                  <p>场景数据仅桌面端可用，请先确认应用已正常运行。</p>
                </div>
              )
            )}
          </section>

          {sessions.length > 0 ? (
            <section className="conversation-history" aria-label="历史会话">
              <div className="catalog-head">
                <p className="eyebrow">历史会话</p>
                <p className="catalog-note">继续之前的对话，或查看已完成的练习记录。</p>
              </div>
              <ul>
                {sessions.map((item) => (
                  <li key={item.id}>
                    <button
                      type="button"
                      onClick={() => void handleOpenSession(item.id)}
                      disabled={sending}
                    >
                      <span className="conversation-history-title">{item.scene_title}</span>
                      <span className="conversation-history-meta">
                        {item.turn_count} 轮 · {item.message_count} 条消息 · {formatWhen(item.started_at)}
                        {item.ended_at ? "" : " · 进行中"}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          ) : null}

          <p className="page-tip conversation-tip">
            提示：语音由本地 Whisper 转写，AI 回复与纠错通过网络请求完成；对话记录保存在本地学习数据。
            <button type="button" onClick={() => onNavigate("analytics")}>查看学习数据</button>
          </p>
        </>
      )}
    </div>
  );
}
