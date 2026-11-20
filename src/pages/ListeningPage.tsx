import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AlertCircle,
  CheckCircle2,
  ChevronLeft,
  Gauge,
  Headphones,
  Loader2,
  Pause,
  Play,
  Repeat,
  RotateCcw,
  Subtitles,
  Trash2,
  XCircle,
} from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api, PREVIEW_MESSAGE, toUserMessage } from "../lib/api";
import type {
  AudioCatalogEntry,
  DictationResult,
  DictationStats,
  ListeningDetail,
} from "../lib/api";
import { isTauriRuntime } from "../lib/desktop";
import type { NavId } from "../data/navigation";

interface ListeningPageProps {
  onNavigate: (id: NavId) => void;
  onNotify: (message: string) => void;
}

const SPEEDS = [0.75, 1, 1.25] as const;

function formatClock(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function formatDuration(ms: number): string {
  return `${(ms / 1000).toFixed(1)}s`;
}

function accuracyLabel(attempts: number, correct: number): string {
  if (attempts <= 0) return "未练习";
  return `${Math.round((correct / attempts) * 100)}%`;
}

export function ListeningPage({ onNavigate, onNotify }: ListeningPageProps) {
  const [catalog, setCatalog] = useState<AudioCatalogEntry[]>([]);
  const [stats, setStats] = useState<DictationStats | null>(null);
  const [selectedId, setSelectedId] = useState<number | null>(() => {
    const saved = Number(localStorage.getItem("janus.listening.selectedId"));
    return Number.isFinite(saved) && saved > 0 ? saved : null;
  });
  const [detail, setDetail] = useState<ListeningDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [notice, setNotice] = useState<string | null>(null);

  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [playing, setPlaying] = useState(false);
  const [currentMs, setCurrentMs] = useState(0);
  const [durationMs, setDurationMs] = useState(0);
  const [speed, setSpeed] = useState<number>(() => {
    const saved = Number(localStorage.getItem("janus.listening.speed"));
    return (SPEEDS as readonly number[]).includes(saved) ? saved : 1;
  });
  const [loopSegment, setLoopSegment] = useState<boolean>(
    () => localStorage.getItem("janus.listening.loop") === "1",
  );
  const [showSubtitle, setShowSubtitle] = useState<boolean>(
    () => localStorage.getItem("janus.listening.subtitle") !== "0",
  );

  const [dictation, setDictation] = useState("");
  const [dictationBusy, setDictationBusy] = useState(false);
  const [lastResult, setLastResult] = useState<DictationResult | null>(null);
  const dictationStartAt = useRef(Date.now());

  const selected = useMemo(
    () => catalog.find((entry) => entry.id === selectedId) ?? null,
    [catalog, selectedId],
  );

  const refreshCatalog = useCallback(async () => {
    try {
      const [nextCatalog, nextStats] = await Promise.all([
        api.getAudioCatalog(),
        api.getDictationStats(),
      ]);
      setCatalog(nextCatalog);
      setStats(nextStats);
      setNotice(null);
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refreshCatalog();
  }, [refreshCatalog]);

  // Drop a stale selection (e.g. the clip was deleted).
  useEffect(() => {
    if (selectedId != null && catalog.length > 0 && !catalog.some((entry) => entry.id === selectedId)) {
      setSelectedId(null);
      setDetail(null);
    }
  }, [catalog, selectedId]);

  useEffect(() => {
    if (selectedId == null) {
      setDetail(null);
      return;
    }
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

  useEffect(() => {
    localStorage.setItem("janus.listening.speed", String(speed));
    localStorage.setItem("janus.listening.loop", loopSegment ? "1" : "0");
    localStorage.setItem("janus.listening.subtitle", showSubtitle ? "1" : "0");
    if (selectedId != null) {
      localStorage.setItem("janus.listening.selectedId", String(selectedId));
    }
  }, [speed, loopSegment, showSubtitle, selectedId]);

  const audioSrc = useMemo(() => {
    if (!selected || !isTauriRuntime) return "";
    return convertFileSrc(selected.file_path);
  }, [selected]);

  // Reset transport whenever the clip changes.
  useEffect(() => {
    const audio = audioRef.current;
    if (audio) {
      audio.pause();
      audio.currentTime = 0;
    }
    setCurrentMs(0);
    setDurationMs(0);
    setPlaying(false);
    setLastResult(null);
    setDictation("");
    dictationStartAt.current = Date.now();
  }, [audioSrc]);

  // Keep playback rate in sync with the audio element.
  useEffect(() => {
    const audio = audioRef.current;
    if (audio) audio.playbackRate = speed;
  }, [speed, audioSrc]);

  const activeSegment = useMemo(() => {
    if (!detail) return null;
    return (
      detail.segments.find(
        (segment) => currentMs >= segment.start_ms && currentMs < segment.end_ms,
      ) ?? null
    );
  }, [detail, currentMs]);

  const handleTimeUpdate = () => {
    const audio = audioRef.current;
    if (!audio) return;
    const nextMs = Math.round(audio.currentTime * 1000);
    setCurrentMs(nextMs);
    if (loopSegment && detail && detail.segments.length > 0) {
      const segment =
        detail.segments.find((item) => nextMs >= item.start_ms && nextMs < item.end_ms) ??
        detail.segments[detail.segments.length - 1];
      if (nextMs >= segment.end_ms) {
        audio.currentTime = segment.start_ms / 1000;
      }
    }
  };

  const handleLoadedMetadata = () => {
    const audio = audioRef.current;
    if (audio) setDurationMs(Math.round((audio.duration || 0) * 1000));
  };

  const handleEnded = () => setPlaying(false);

  const handleTogglePlay = () => {
    const audio = audioRef.current;
    if (!audio) return;
    if (playing) {
      audio.pause();
      setPlaying(false);
    } else {
      void audio.play().catch(() => {
        setNotice("播放失败：音频文件可能已被删除，请重新选择片段。");
        setPlaying(false);
      });
      setPlaying(true);
    }
  };

  const handleReplay = () => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = 0;
    setCurrentMs(0);
    void audio.play().catch(() => setPlaying(false));
    setPlaying(true);
  };

  const handleSeek = (seconds: number) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = seconds;
    setCurrentMs(Math.round(seconds * 1000));
  };

  const handleSelect = (entry: AudioCatalogEntry) => {
    if (entry.id === selectedId) return;
    setSelectedId(entry.id);
  };

  const handleDelete = async (entry: AudioCatalogEntry) => {
    if (!window.confirm(`删除「${entry.title}」？该片段的听写记录也会一并删除。`)) return;
    try {
      await api.deleteSeedAudio(entry.id);
      setCatalog((items) => items.filter((item) => item.id !== entry.id));
      if (selectedId === entry.id) {
        setSelectedId(null);
        setDetail(null);
      }
      onNotify("已删除该听力片段。");
    } catch (error) {
      setNotice(toUserMessage(error));
    }
  };

  const handleDictationSubmit = async () => {
    if (!selected || !detail || detail.segments.length === 0 || dictationBusy) return;
    const segment = activeSegment ?? detail.segments[0];
    const durationMs = Date.now() - dictationStartAt.current;
    setDictationBusy(true);
    try {
      const result = await api.recordDictation(
        selected.id,
        segment.id,
        dictation.trim(),
        durationMs,
      );
      setLastResult(result);
      setDictation("");
      dictationStartAt.current = Date.now();
      setCatalog((items) =>
        items.map((item) =>
          item.id === selected.id
            ? {
                ...item,
                dictation_attempts: item.dictation_attempts + 1,
                dictation_correct: item.dictation_correct + (result.correct ? 1 : 0),
              }
            : item,
        ),
      );
      try {
        setStats(await api.getDictationStats());
      } catch {
        /* 统计刷新失败不阻断听写流程 */
      }
    } catch (error) {
      setNotice(toUserMessage(error));
    } finally {
      setDictationBusy(false);
    }
  };

  if (loading) {
    return (
      <div className="listening-loading">
        <Loader2 size={20} className="spin" />
        <p>正在读取本地音频库…</p>
      </div>
    );
  }

  return (
    <div className="learning-page listening-page">
      <section className="page-heading learning-heading">
        <div>
          <p className="eyebrow">听力 · M3</p>
          <h1>精听训练</h1>
          <p>真实语音片段、逐句定位、变速复听与听写判断。不伪造语音与评分。</p>
        </div>
        <div className="learning-heading-stats">
          <span>片段 <b>{catalog.length}</b></span>
          {stats && (
            <>
              <span>听写 <b>{stats.total_attempts}</b></span>
              <span>正确率 <b>{stats.accuracy_percent}%</b></span>
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

      {catalog.length === 0 && !notice && (
        <div className="listening-empty">
          <Headphones size={22} />
          <p>{PREVIEW_MESSAGE}</p>
        </div>
      )}

      {selected && detail ? (
        <div className="listening-player">
          <div className="player-toolbar">
            <button type="button" className="button-secondary" onClick={() => setSelectedId(null)}>
              <ChevronLeft size={14} /> 返回列表
            </button>
            <div className="player-title">
              <p className="eyebrow">正在精听 · {formatDuration(durationMs || selected.duration_ms)}</p>
              <h2>{selected.title}</h2>
              <p className="player-subtitle-zh">{selected.text_zh}</p>
            </div>
            <button
              type="button"
              className="button-danger"
              onClick={() => handleDelete(selected)}
              title="删除片段与听写记录"
            >
              <Trash2 size={14} /> 删除
            </button>
          </div>

          <audio
            ref={audioRef}
            src={audioSrc}
            onTimeUpdate={handleTimeUpdate}
            onLoadedMetadata={handleLoadedMetadata}
            onEnded={handleEnded}
            preload="auto"
          />

          <div className="player-transport">
            <div className="transport-buttons">
              <button
                type="button"
                className="transport-play"
                onClick={handleTogglePlay}
                title={playing ? "暂停" : "播放"}
              >
                {playing ? <Pause size={18} /> : <Play size={18} />}
              </button>
              <button
                type="button"
                className="button-secondary transport-icon"
                onClick={handleReplay}
                title="从头播放"
              >
                <RotateCcw size={15} />
              </button>
            </div>
            <div className="seek-row">
              <input
                type="range"
                min={0}
                max={Math.max(0.1, durationMs / 1000 || selected.duration_ms / 1000)}
                step={0.01}
                value={Math.min(currentMs / 1000, (durationMs / 1000) || selected.duration_ms / 1000)}
                onChange={(event) => handleSeek(Number(event.target.value))}
                aria-label="播放进度"
              />
              <span className="seek-time">
                {formatClock(currentMs)} / {formatClock(durationMs || selected.duration_ms)}
              </span>
            </div>
            <div className="transport-options">
              <div className="speed-group" aria-label="播放速度">
                {SPEEDS.map((value) => (
                  <button
                    type="button"
                    key={value}
                    className={`speed-button ${speed === value ? "active" : ""}`}
                    onClick={() => setSpeed(value)}
                  >
                    {value}×
                  </button>
                ))}
              </div>
              <button
                type="button"
                className={`option-button ${loopSegment ? "active" : ""}`}
                onClick={() => setLoopSegment((value) => !value)}
                title="到达句末后自动回到句首"
              >
                <Repeat size={14} /> 单句循环
              </button>
              <button
                type="button"
                className={`option-button ${showSubtitle ? "active" : ""}`}
                onClick={() => setShowSubtitle((value) => !value)}
                title="显示 / 隐藏字幕"
              >
                <Subtitles size={14} /> 字幕
              </button>
            </div>
          </div>

          {showSubtitle && detail.segments.length > 0 && (
            <div className="subtitle-strip">
              <span className="subtitle-en">{activeSegment?.text_en ?? detail.segments[0].text_en}</span>
              <span className="subtitle-zh">{activeSegment?.text_zh ?? detail.segments[0].text_zh}</span>
            </div>
          )}

          {detail.segments.length > 1 && (
            <div className="segment-list" aria-label="句段定位">
              {detail.segments.map((segment) => (
                <button
                  type="button"
                  key={segment.id}
                  className={`segment-chip ${activeSegment?.id === segment.id ? "active" : ""}`}
                  onClick={() => handleSeek(segment.start_ms / 1000)}
                  title={segment.text_en}
                >
                  <span className="segment-seq">{String(segment.seq).padStart(2, "0")}</span>
                  {segment.text_en}
                </button>
              ))}
            </div>
          )}

          <div className="dictation-panel">
            <div className="dictation-head">
              <p className="eyebrow">听写判断</p>
              <h3>写下你听到的内容</h3>
              <p>提交后由本地数据层判断正误，不采用客户端自评分。</p>
            </div>
            <div className="dictation-row">
              <input
                type="text"
                value={dictation}
                onChange={(event) => setDictation(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") void handleDictationSubmit();
                }}
                placeholder="输入英文原文，回车提交…"
                disabled={dictationBusy}
              />
              <button
                type="button"
                className="button-primary"
                onClick={handleDictationSubmit}
                disabled={dictationBusy || dictation.trim().length === 0}
              >
                {dictationBusy ? <Loader2 size={14} className="spin" /> : "提交判断"}
              </button>
            </div>
            {lastResult && (
              <div className={`dictation-result ${lastResult.correct ? "correct" : "wrong"}`}>
                {lastResult.correct ? <CheckCircle2 size={16} /> : <XCircle size={16} />}
                <div>
                  <p>{lastResult.correct ? "听写正确，继续加油。" : "听写有出入，对照原文再听一遍。"}</p>
                  {!lastResult.correct && <p className="dictation-expected">原文：{lastResult.expected}</p>}
                </div>
              </div>
            )}
          </div>
        </div>
      ) : (
        <>
          <section className="listening-stats" aria-label="听写统计">
            <article className="stat-card">
              <Gauge size={18} />
              <div>
                <span>累计听写</span>
                <b>{stats?.total_attempts ?? 0}</b>
              </div>
            </article>
            <article className="stat-card">
              <CheckCircle2 size={18} />
              <div>
                <span>今日正确</span>
                <b>{stats?.today_correct ?? 0}/{stats?.today_attempts ?? 0}</b>
              </div>
            </article>
            <article className="stat-card">
              <Gauge size={18} />
              <div>
                <span>总正确率</span>
                <b>{stats?.accuracy_percent ?? 0}%</b>
              </div>
            </article>
          </section>

          {stats && stats.weakest_clips.length > 0 && (
            <section className="weak-clips" aria-label="薄弱片段">
              <p className="eyebrow">需要重听的片段</p>
              <ul>
                {stats.weakest_clips.map((weak) => (
                  <li key={weak.clip_id}>
                    <span className="weak-title">{weak.title}</span>
                    <span className="weak-accuracy">{weak.accuracy_percent}% · {weak.correct}/{weak.attempts}</span>
                  </li>
                ))}
              </ul>
            </section>
          )}

          <section className="catalog-list" aria-label="音频片段库">
            <div className="catalog-head">
              <p className="eyebrow">音频片段库 · 本地缓存</p>
              <p className="catalog-note">共 {catalog.length} 段真实语音，可逐段播放、变速与听写。</p>
            </div>
            {catalog.map((entry) => (
              <article
                key={entry.id}
                className={`catalog-item ${selectedId === entry.id ? "active" : ""}`}
                onClick={() => handleSelect(entry)}
              >
                <button
                  type="button"
                  className="catalog-play"
                  onClick={(event) => {
                    event.stopPropagation();
                    handleSelect(entry);
                  }}
                  title="播放此片段"
                >
                  <Play size={15} />
                </button>
                <div className="catalog-copy">
                  <h3>{entry.title}</h3>
                  <p>{entry.text_zh}</p>
                </div>
                <div className="catalog-meta">
                  <span className="catalog-duration">{formatDuration(entry.duration_ms)}</span>
                  <span className="catalog-accuracy">
                    {accuracyLabel(entry.dictation_attempts, entry.dictation_correct)}
                  </span>
                  <button
                    type="button"
                    className="catalog-delete"
                    onClick={(event) => {
                      event.stopPropagation();
                      void handleDelete(entry);
                    }}
                    title="删除片段"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </article>
            ))}
          </section>
        </>
      )}
    </div>
  );
}
