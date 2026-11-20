interface WaveformProps {
  bars?: number;
  compact?: boolean;
  label?: string;
}

const heights = [12, 30, 13, 30, 13, 30, 14, 30, 15, 30, 16, 30, 17, 30, 17, 29, 18, 29];

export function Waveform({ bars = heights.length, compact = false, label = "语音活动波形" }: WaveformProps) {
  return (
    <div className={compact ? "waveform waveform-compact" : "waveform"} role="img" aria-label={label}>
      {heights.slice(0, bars).map((height, index) => (
        <span
          key={index}
          className={index % 3 === 1 ? "waveform-brass" : undefined}
          style={{ height: `${compact ? Math.max(6, Math.round(height * 0.55)) : height}px` }}
        />
      ))}
    </div>
  );
}
