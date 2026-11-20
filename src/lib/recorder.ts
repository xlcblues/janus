import { useCallback, useRef, useState } from "react";

/**
 * 麦克风录音：捕获浏览器默认采样率下的单声道浮点音频，
 * 重采样到 16kHz，并封装为 16-bit PCM 单声道 WAV。
 *
 * 后端（Rust/whisper.cpp）只接受 16kHz / mono / PCM16 的 WAV，
 * 因此这里不使用 MediaRecorder（默认产出 Opus/MP4，无法直接喂给 ASR）。
 */

const TARGET_RATE = 16000;
const BYTES_PER_SAMPLE = 2;

export interface RecordingResult {
  /** 完整的 16kHz mono PCM16 WAV 字节（可直接传给后端转写）。 */
  wavBytes: Uint8Array;
  /** 供本地回放使用的 WAV Blob。 */
  wavBlob: Blob;
  durationMs: number;
  sampleCount: number;
}

export interface RecorderHandle {
  start: () => Promise<void>;
  stop: () => RecordingResult;
  cancel: () => void;
}

function downsample(input: Float32Array, inputRate: number): Float32Array {
  if (inputRate === TARGET_RATE) return input;
  const ratio = inputRate / TARGET_RATE;
  const outLength = Math.floor(input.length / ratio);
  const output = new Float32Array(outLength);
  for (let i = 0; i < outLength; i += 1) {
    const start = Math.floor(i * ratio);
    const end = Math.min(input.length, Math.floor((i + 1) * ratio));
    let sum = 0;
    for (let j = start; j < end; j += 1) sum += input[j];
    output[i] = sum / Math.max(1, end - start);
  }
  return output;
}

function encodePcm16Wav(samples: Float32Array, sampleRate: number): Uint8Array {
  const dataSize = samples.length * BYTES_PER_SAMPLE;
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);

  const writeString = (offset: number, value: string) => {
    for (let i = 0; i < value.length; i += 1) {
      view.setUint8(offset + i, value.charCodeAt(i));
    }
  };

  writeString(0, "RIFF");
  view.setUint32(4, 36 + dataSize, true);
  writeString(8, "WAVE");
  writeString(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * BYTES_PER_SAMPLE, true);
  view.setUint16(32, BYTES_PER_SAMPLE, true);
  view.setUint16(34, 16, true);
  writeString(36, "data");
  view.setUint32(40, dataSize, true);

  let offset = 44;
  for (let i = 0; i < samples.length; i += 1) {
    const clamped = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(offset, clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff, true);
    offset += BYTES_PER_SAMPLE;
  }
  return new Uint8Array(buffer);
}

function getAudioContextCtor(): typeof AudioContext {
  const win = window as unknown as {
    AudioContext?: typeof AudioContext;
    webkitAudioContext?: typeof AudioContext;
  };
  return win.AudioContext ?? win.webkitAudioContext!;
}

export function createRecorder(): RecorderHandle {
  let context: AudioContext | null = null;
  let stream: MediaStream | null = null;
  let source: MediaStreamAudioSourceNode | null = null;
  let processor: ScriptProcessorNode | null = null;
  let chunks: Float32Array[] = [];
  let cleanedUp = false;

  const cleanup = () => {
    if (cleanedUp) return;
    cleanedUp = true;
    try {
      processor?.disconnect();
    } catch {
      /* noop */
    }
    try {
      source?.disconnect();
    } catch {
      /* noop */
    }
    stream?.getTracks().forEach((track) => track.stop());
    void context?.close().catch(() => undefined);
    processor = null;
    source = null;
    stream = null;
    context = null;
  };

  return {
    async start() {
      if (!navigator.mediaDevices?.getUserMedia) {
        throw new Error("当前环境不支持麦克风录音（缺少媒体设备 API）。");
      }
      const nextStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true,
        },
      });
      const Ctor = getAudioContextCtor();
      const nextContext = new Ctor();
      const nextSource = nextContext.createMediaStreamSource(nextStream);
      const nextProcessor = nextContext.createScriptProcessor(4096, 1, 1);
      nextProcessor.onaudioprocess = (event) => {
        const data = event.inputBuffer.getChannelData(0);
        const copy = new Float32Array(data.length);
        copy.set(data);
        chunks.push(copy);
      };
      nextSource.connect(nextProcessor);
      nextProcessor.connect(nextContext.destination);
      context = nextContext;
      stream = nextStream;
      source = nextSource;
      processor = nextProcessor;
      chunks = [];
      cleanedUp = false;
    },

    stop() {
      const inputRate = context?.sampleRate ?? TARGET_RATE;
      const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
      const input = new Float32Array(totalLength);
      let cursor = 0;
      for (const chunk of chunks) {
        input.set(chunk, cursor);
        cursor += chunk.length;
      }
      const pcm16 = downsample(input, inputRate);
      const wavBytes = encodePcm16Wav(pcm16, TARGET_RATE);
      cleanup();
      return {
        wavBytes,
        wavBlob: new Blob([wavBytes.buffer as ArrayBuffer], { type: "audio/wav" }),
        durationMs: Math.round((pcm16.length / TARGET_RATE) * 1000),
        sampleCount: pcm16.length,
      };
    },

    cancel() {
      chunks = [];
      cleanup();
    },
  };
}

/** React Hook：管理一次录音的生命周期，返回状态与动作。 */
export function useRecorder() {
  const [recording, setRecording] = useState(false);
  const [result, setResult] = useState<RecordingResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const handleRef = useRef<RecorderHandle | null>(null);

  const start = useCallback(async () => {
    setError(null);
    try {
      const handle = createRecorder();
      await handle.start();
      handleRef.current = handle;
      setResult(null);
      setRecording(true);
    } catch (reason) {
      const message =
        reason instanceof Error && reason.message
          ? reason.message
          : "无法访问麦克风，请检查系统录音权限后重试。";
      setError(message);
      setRecording(false);
      throw new Error(message);
    }
  }, []);

  const stop = useCallback(() => {
    const handle = handleRef.current;
    if (!handle) return null;
    handleRef.current = null;
    const nextResult = handle.stop();
    setResult(nextResult);
    setRecording(false);
    return nextResult;
  }, []);

  const cancel = useCallback(() => {
    handleRef.current?.cancel();
    handleRef.current = null;
    setRecording(false);
  }, []);

  const clearError = useCallback(() => setError(null), []);

  return { recording, result, error, start, stop, cancel, clearError };
}
