import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AlertCircle,
  CheckCircle2,
  ChevronLeft,
  Gauge,
  Loader2,
  Mic,
  Pause,
  Play,
  RefreshCw,
  RotateCcw,
  Sparkles,
  Square,
  Volume2,
} from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api, PREVIEW_MESSAGE, toUserMessage } from "../lib/api";
import type {
  AudioCatalogEntry,
  ListeningDetail,
  SpeakingResult,
  SpeakingStats,
  SpeakingStatus,
  WordFeedback,
} from "../lib/api";
import { isTauriRuntime } from "../lib/desktop";
import { useRecorder } from "../lib/recorder";
import type { NavId } from "../data/navigation";

interface SpeakingPageProps {
  onNavigate: (id: NavId) => void;
  onNotify: (message: string) => void;
}

function formatDuration(ms: number): string {
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatClock(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function wordStatusLabel(status: WordFeedback["status"]): string {
  switch (status) {
    case "correct":
      return "读准";
    case "missing":
      return "漏读";
    case "changed":
      return "替换";
    case "extra":
      return "多读";
  }
}

export function SpeakingPage({ onNavigate, onNotify }: SpeakingPageProps) {
  const [catalog, setCatalog] = useState<AudioCatalogEntry[]>([]);
  const [stats, setStats] = useState<SpeakingStats | null>(null);
  const [status, setStatus] = useState<SpeakingStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<string | null>(null);

  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [detail, setDetail] = useState<ListeningDetail | null>(null);
  const [activeSegmentId, setActiveSegmentId] = useState<number | null>(null);

  // 参考音频（本地片段）播放器
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [playing, setPlaying] = useState(false);
  const [currentMs, setCurrentMs] = useState(0);

  // 录音生命周期（优先使用设置页选择的麦克风）
  const [micDeviceId, setMicDeviceId] = useState<string>("");
  useEffect(() => {
    if (!isTauriRuntime) return;
    api
      .getMicrophoneSettings()
      .then((settings) => setMicDeviceId(settings.device_id))
      .catch(() => setMicDeviceId(""));
  }, []);
  const recorder = useRecorder(micDeviceId);
  const recordingAudioRef = useRef<HTMLAudioElement | null>(null);
  const [recordingUrl, setRecordingUrl] = useState<string | null>(null);

  // 评测
  const [evaluating, setEvaluating] = useState(false);
  const [result, setResult] = useState<SpeakingResult | null>(null);

  const selected = useMemo(
    () => catalog.find((entry) => entry.id === selectedId) ?? null,
    [catalog, selectedId],
  );

  const activeSegment = useMemo(() => {
    if (!detail || activeSegmentId == null) return null;
    return detail.segments.find((segment) => segment.id === activeSegmentId) ?? null;
  }, [detail, activeSegmentId]);

  const refreshAll = useCallback(async () => {
    try {
      const [nextCatalog, nextStats, nextStatus] = await Promise.all([
        api.getAudioCatalog(),
        api.getSpeakingStats(),
        api.getSpeakingStatus(),
      ]);
      setCatalog(nextCatalog);
      setStats(nextStats);
      setStatus(nextStatus);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshAll();
  }, [refreshAll]);

  // 选择片段时加载句段明细
  useEffect(() => {
    if (selectedId == null) {
      setDetail(null);
      setActiveSegmentId(null);
      return;
    }
    setActiveSegmentId(null);
    let cancelled = false;
    api
      .getListeningDetail(selectedId)
      .then((nextDetail) => {
        if (!cancelled) setDetail(nextDetail);
      })
      .catch((error) => {
        if (!cancelled) setNotice(toUserMessage(error));
      });
    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  // 录音回放 URL 的生命周期
  useEffect(() => {
    const previous = recordingUrl;
    return () => {
      if (previous) URL.revokeObjectURL(previous);
    };
  }, [recordingUrl]);

  // 卸载时释放麦克风
  useEffect(() => {
    return () => recorder.cancel();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const audioSrc = useMemo(() => {
    if (!selected || !isTauriRuntime) return "";
    return convertFileSrc(selected.file_path);
  }, [selected]);

  // 切换片段时重置
  useEffect(() => {
    const audio = audioRef.current;
    if (audio) {
      audio.pause();
      audio.currentTime = 0;
    }
    setCurrentMs(0);
    setPlaying(false);
    setResult(null);
  }, [audioSrc]);

  const handleToggleReference = () => {
    const audio = audioRef.current;
    if (!audio) return;
    if (playing) {
      audio.pause();
      setPlaying(false);
      return;
    }
    void audio.play().catch(() => {
      setNotice("参考音频播放失败，片段可能已被删除。");
      setPlaying(false);
    });
    setPlaying(true);
  };

  const handlePlaySegmentReference = (segment: { start_ms: number }) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = segment.start_ms / 1000;
    void audio.play().catch(() => {
      setNotice("参考音频播放失败，片段可能已被删除。");
      setPlaying(false);
    });
    setPlaying(true);
  };

  const handleStartRecording = async () => {
    setResult(null);
    try {
      await recorder.start();
    } catch (error) {
      setNotice(toUserMessage(error));
    }
  };

  const handleStopRecording = () => {
    const recorded = recorder.stop();
    if (!recorded) return;
    const url = URL.createObjectURL(recorded.wavBlob);
    setRecordingUrl((previous) => {
      if (previous) URL.revokeObjectURL(previous);
      return url;
    });
  };

  const handlePlayRecording = () => {
    const audio = recordingAudioRef.current;
    if (!audio) return;
    void audio.play().catch(() => setNotice("录音回放失败，请重新录制。"));
  };

  const handleDiscardRecording = () => {
    recorder.cancel();
    setRecordingUrl((previous) => {
      if (previous) URL.revokeObjectURL(previous);
      return null;
    });
    setResult(null);
  };

  const handleEvaluate = async () => {
    if (!selected || !activeSegment || !recorder.result || evaluating) return;
    setEvaluating(true);
    try {
      const nextResult = await api.evaluateSpeaking(
        selected.id,
        activeSegment.id,
        Array.from(recorder.result.wavBytes),
        recorder.result.durationMs,
      );
      setResult(nextResult);
      try {
        setStats(await api.getSpeakingStats());
      } catch {
        /* 统计刷新失败不阻断评测流程 */
      }
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setEvaluating(false);
    }
  };

  const modelUnavailable = status !== null && !status.available;

  if (loading) {
    return (
      <div className="listening-loading">
        <Loader2 size={20} className="spin" />
        <p>正在准备朗读模块…</p>
      </div>
    );
  }

  return (
    <div className="learning-page listening-page speaking-page">
      <section className="page-heading learning-heading">
        <div>
          <p className="eyebrow">朗读 · M4</p>
          <h1>跟读与发音反馈</h1>
          <p>录制后由本地离线语音识别转写，逐词比对并给出漏读、替换与多读提示。录音不保存。</p>
        </div>
        <div className="learning-heading-stats">
          <span>片段 <b>{catalog.length}</b></span>
          {stats && (
            <>
              <span>朗读 <b>{stats.total_attempts}</b></span>
              <span>准确率 <b>{stats.accuracy_percent}%</b></span>
            </>
          )}
        </div>
      </section>

      {notice && (
        <div className="notice-bar">
          <AlertCircle size={15} />
          <span>{notice}</span>
        </div>
      )}

      {modelUnavailable && (
        <section className="model-setup" aria-label="语音模型安装">
          <Sparkles size={20} />
          <div>
            <p className="eyebrow">离线语音模型未安装</p>
            <h3>先下载 tiny.en 模型即可开始朗读练习</h3>
            <p>
              朗读识别在本地离线运行，需要约 77MB 的 whisper tiny.en 模型。请在项目根目录执行以下命令：
            </p>
            <code>powershell -ExecutionPolicy Bypass -File scripts\download-whisper-model.ps1</code>
            <p className="model-setup-hint">
              下载完成后重启应用。模型只保存在本机，不随仓库分发。
              {status?.error ? ` 检测信息：${status.error}` : ""}
            </p>
          </div>
          <button type="button" className="button-secondary" onClick={() => void refreshAll()}>
            <RefreshCw size={14} /> 重新检测
          </button>
        </section>
      )}

      {modelUnavailable && (
        <section className="listening-stats" aria-label="朗读统计">
          <article className="stat-card">
            <Gauge size={18} />
            <div>
              <span>累计朗读</span>
              <b>{stats?.total_attempts ?? 0}</b>
            </div>
          </article>
          <article className="stat-card">
            <Mic size={18} />
            <div>
              <span>今日练习</span>
              <b>{stats?.today_attempts ?? 0}</b>
            </div>
          </article>
          <article className="stat-card">
            <Gauge size={18} />
            <div>
              <span>平均准确率</span>
              <b>{stats?.accuracy_percent ?? 0}%</b>
            </div>
          </article>
        </section>
      )}

      {!modelUnavailable && catalog.length === 0 && !notice && (
        <div className="listening-empty">
          <Mic size={22} />
          <p>{PREVIEW_MESSAGE}</p>
        </div>
      )}

      {!modelUnavailable && selected && detail ? (
        <div className="listening-player">
          <div className="player-toolbar">
            <button type="button" className="button-secondary" onClick={() => setSelectedId(null)}>
              <ChevronLeft size={14} /> 返回列表
            </button>
            <div className="player-title">
              <p className="eyebrow">正在朗读 · {formatDuration(selected.duration_ms)}</p>
              <h2>{selected.title}</h2>
              <p className="player-subtitle-zh">{selected.text_zh}</p>
            </div>
          </div>

          <audio
            ref={audioRef}
            src={audioSrc}
            onTimeUpdate={(event) =>
              setCurrentMs(Math.round((event.currentTarget.currentTime || 0) * 1000))
            }
            onEnded={() => setPlaying(false)}
            preload="auto"
          />

          <div className="player-transport">
            <div className="transport-buttons">
              <button
                type="button"
                className="transport-play"
                onClick={handleToggleReference}
                title={playing ? "暂停" : "播放整段参考"}
              >
                {playing ? <Pause size={18} /> : <Play size={18} />}
              </button>
            </div>
            <div className="seek-row">
              <input
                type="range"
                min={0}
                max={Math.max(0.1, selected.duration_ms / 1000)}
                step={0.01}
                value={Math.min(currentMs / 1000, selected.duration_ms / 1000)}
                onChange={(event) => {
                  const audio = audioRef.current;
                  if (audio) audio.currentTime = Number(event.target.value);
                  setCurrentMs(Number(event.target.value) * 1000);
                }}
                aria-label="参考音频进度"
              />
              <span className="seek-time">{formatClock(currentMs)}</span>
            </div>
          </div>

          <div className="segment-list" aria-label="朗读句段">
            {detail.segments.map((segment) => (
              <article
                key={segment.id}
                className={`speaking-segment ${activeSegmentId === segment.id ? "active" : ""}`}
              >
                <button
                  type="button"
                  className="segment-seq speaking-seq"
                  onClick={() => handlePlaySegmentReference(segment)}
                  title="播放该句参考"
                >
                  <Volume2 size={13} />
                  <span>{String(segment.seq).padStart(2, "0")}</span>
                </button>
                <div className="speaking-segment-copy">
                  <b>{segment.text_en}</b>
                  <p>{segment.text_zh}</p>
                </div>
                <button
                  type="button"
                  className={`button-primary speaking-practice ${
                    activeSegmentId === segment.id ? "active" : ""
                  }`}
                  onClick={() => {
                    setActiveSegmentId(segment.id);
                    setResult(null);
                    handleDiscardRecording();
                  }}
                >
                  跟读
                </button>
              </article>
            ))}
          </div>

          {activeSegment && (
            <div className="speaking-panel">
              <div className="speaking-target">
                <p className="eyebrow">朗读目标</p>
                <h3>{activeSegment.text_en}</h3>
                <p>{activeSegment.text_zh}</p>
                <button
                  type="button"
                  className="button-secondary"
                  onClick={() => handlePlaySegmentReference(activeSegment)}
                >
                  <Volume2 size={14} /> 听参考
                </button>
              </div>

              <div className="recorder-box">
                {!recorder.recording && !recorder.result ? (
                  <button
                    type="button"
                    className="recorder-button"
                    onClick={() => void handleStartRecording()}
                    title="开始录音"
                  >
                    <Mic size={22} />
                    <span>开始朗读</span>
                  </button>
                ) : recorder.recording ? (
                  <button
                    type="button"
                    className="recorder-button recording"
                    onClick={handleStopRecording}
                    title="结束录音"
                  >
                    <Square size={20} />
                    <span>结束 · 正在录音</span>
                  </button>
                ) : recorder.result ? (
                  <div className="recorder-recorded">
                    <button
                      type="button"
                      className="recorder-button replay"
                      onClick={handlePlayRecording}
                      title="回放录音"
                    >
                      <Play size={20} />
                    </button>
                    <div className="recorder-recorded-copy">
                      <b>录音完成</b>
                      <p>{formatDuration(recorder.result.durationMs)}</p>
                    </div>
                    <button
                      type="button"
                      className="button-secondary"
                      onClick={handleDiscardRecording}
                      disabled={evaluating}
                    >
                      <RotateCcw size={14} /> 重新录
                    </button>
                    <button
                      type="button"
                      className="button-primary"
                      onClick={() => void handleEvaluate()}
                      disabled={evaluating}
                    >
                      {evaluating ? <Loader2 size={14} className="spin" /> : "提交评测"}
                    </button>
                  </div>
                ) : null}
                {recorder.error && <p className="recorder-error">{recorder.error}</p>}
              </div>

              <audio
                ref={recordingAudioRef}
                src={recordingUrl ?? undefined}
                onEnded={() => undefined}
                style={{ display: "none" }}
              />

              {evaluating && (
                <div className="evaluating-hint">
                  <Loader2 size={15} className="spin" />
                  <span>正在本地识别并比对…（约数秒）</span>
                </div>
              )}

              {result && (
                <div className="speaking-result">
                  <div className="speaking-result-head">
                    <div>
                      <p className="eyebrow">评测结果</p>
                      <h3>准确率 {result.accuracy_percent}% · 匹配 {result.word_matches}/{result.word_total}</h3>
                    </div>
                    <span className="confidence-badge">
                      置信度 {Math.round(result.confidence * 100)}%
                    </span>
                  </div>

                  {result.low_confidence && (
                    <div className="low-confidence-hint">
                      <AlertCircle size={15} />
                      <span>识别置信度较低，结果可能不准确，建议对照原文重听后再读一次。</span>
                    </div>
                  )}

                  <div className="word-feedback" aria-label="逐词反馈">
                    {result.words.map((word, index) => (
                      <span
                        key={`${word.status}-${word.word}-${index}`}
                        className={`word-chip ${word.status}`}
                        title={`${word.word} · ${wordStatusLabel(word.status)}`}
                      >
                        {word.word}
                        <i>{wordStatusLabel(word.status)}</i>
                      </span>
                    ))}
                  </div>

                  <div className="speaking-transcript">
                    <p className="eyebrow">识别文本</p>
                    <p>{result.transcribed || "（未识别到语音）"}</p>
                  </div>

                  <button
                    type="button"
                    className="button-secondary"
                    onClick={() => {
                      setResult(null);
                      handleDiscardRecording();
                    }}
                  >
                    <RotateCcw size={14} /> 再练一次
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      ) : (
        !modelUnavailable && (
          <>
            <section className="listening-stats" aria-label="朗读统计">
              <article className="stat-card">
                <Gauge size={18} />
                <div>
                  <span>累计朗读</span>
                  <b>{stats?.total_attempts ?? 0}</b>
                </div>
              </article>
              <article className="stat-card">
                <Mic size={18} />
                <div>
                  <span>今日练习</span>
                  <b>{stats?.today_attempts ?? 0}</b>
                </div>
              </article>
              <article className="stat-card">
                <Gauge size={18} />
                <div>
                  <span>平均准确率</span>
                  <b>{stats?.accuracy_percent ?? 0}%</b>
                </div>
              </article>
              <article className="stat-card">
                <AlertCircle size={18} />
                <div>
                  <span>低置信次数</span>
                  <b>{stats?.low_confidence_attempts ?? 0}</b>
                </div>
              </article>
            </section>

            {stats && stats.weakest_segments.length > 0 && (
              <section className="weak-clips" aria-label="薄弱句段">
                <p className="eyebrow">需要加强的句段</p>
                <ul>
                  {stats.weakest_segments.map((weak) => (
                    <li key={weak.segment_id}>
                      <span className="weak-title">{weak.title}</span>
                      <span className="weak-accuracy">
                        {weak.accuracy_percent}% · {weak.attempts} 次
                      </span>
                    </li>
                  ))}
                </ul>
              </section>
            )}

            <section className="catalog-list" aria-label="朗读片段库">
              <div className="catalog-head">
                <p className="eyebrow">朗读片段库 · 本地缓存</p>
                <p className="catalog-note">
                  共 {catalog.length} 段真实语音，选择片段后逐句跟读。评分由本地数据层完成。
                </p>
              </div>
              {catalog.map((entry) => (
                <article
                  key={entry.id}
                  className={`catalog-item ${selectedId === entry.id ? "active" : ""}`}
                  onClick={() => setSelectedId(entry.id)}
                >
                  <button
                    type="button"
                    className="catalog-play"
                    onClick={(event) => {
                      event.stopPropagation();
                      setSelectedId(entry.id);
                    }}
                    title="选择此片段"
                  >
                    <Mic size={15} />
                  </button>
                  <div className="catalog-copy">
                    <h3>{entry.title}</h3>
                    <p>{entry.text_zh}</p>
                  </div>
                  <div className="catalog-meta">
                    <span className="catalog-duration">{formatDuration(entry.duration_ms)}</span>
                    <span className="catalog-accuracy">逐句跟读</span>
                  </div>
                </article>
              ))}
            </section>
          </>
        )
      )}

      {!modelUnavailable && catalog.length > 0 && (
        <p className="page-tip">
          提示：录音仅在内存中完成评测，不会保存音频文件；识别文本与得分会记录到学习数据。
          <button type="button" onClick={() => onNavigate("analytics")}>查看学习数据</button>
        </p>
      )}
    </div>
  );
}
