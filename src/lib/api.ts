import { invoke } from "@tauri-apps/api/core";
import { isTauriRuntime } from "./desktop";

/** 浏览器预览模式下提示文案：不伪造数据，只说明数据不可用。 */
export const PREVIEW_MESSAGE = "当前为浏览器预览模式，学习数据仅在桌面应用中可用。";

export interface WordCard {
  id: number;
  word: string;
  phonetic: string;
  meaning: string;
  example_en: string;
  example_zh: string;
  stage: string;
  interval_days: number;
  due_at: number;
  in_vocabulary: boolean;
}

export interface Overview {
  total_words: number;
  due_words: number;
  learned_words: number;
  mastered_words: number;
  total_reviews: number;
  today_reviews: number;
  review_queue: WordCard[];
}

export interface StudyStats {
  total_reviews: number;
  correct_reviews: number;
  accuracy_percent: number;
  total_duration_min: number;
  today_reviews: number;
  today_duration_min: number;
  due_words: number;
  mastered_words: number;
  vocabulary_count: number;
}

export interface SentenceCard {
  id: number;
  en: string;
  zh: string;
  status: string;
  attempts: number;
  correct_count: number;
}

export interface VocabularyEntry {
  word_id: number;
  word: string;
  phonetic: string;
  meaning: string;
  added_at: number;
}

export interface ExportResult {
  path: string;
}

export interface ClearResult {
  backup_path: string;
}


export interface ListeningClip {
  id: number;
  title: string;
  lang: string;
  source: string;
  file_name: string;
  duration_ms: number;
  segment_count: number;
  created_at: number;
}

export interface ListeningSegment {
  id: number;
  clip_id: number;
  seq: number;
  text_en: string;
  text_zh: string;
  start_ms: number;
  end_ms: number;
}

export interface ListeningDetail {
  clip: ListeningClip;
  segments: ListeningSegment[];
}

export interface AudioCatalogEntry {
  id: number;
  title: string;
  text_zh: string;
  source: string;
  file_name: string;
  file_path: string;
  duration_ms: number;
  segment_count: number;
  dictation_attempts: number;
  dictation_correct: number;
}

export interface DictationResult {
  id: number;
  clip_id: number;
  segment_id: number;
  expected: string;
  submitted: string;
  correct: boolean;
  duration_ms: number;
  created_at: number;
}

export interface WeakClip {
  clip_id: number;
  title: string;
  attempts: number;
  correct: number;
  accuracy_percent: number;
}

export interface DictationStats {
  total_attempts: number;
  correct_attempts: number;
  accuracy_percent: number;
  today_attempts: number;
  today_correct: number;
  weakest_clips: WeakClip[];
}

export interface DeleteResult {
  file_name: string;
}

export interface SpeakingStatus {
  available: boolean;
  model_path: string | null;
  error: string | null;
}

export type WordStatus = "correct" | "missing" | "changed" | "extra";

export interface WordFeedback {
  word: string;
  status: WordStatus;
}

export interface SpeakingResult {
  id: number;
  clip_id: number;
  segment_id: number;
  expected: string;
  transcribed: string;
  word_matches: number;
  word_total: number;
  accuracy_percent: number;
  confidence: number;
  low_confidence: boolean;
  duration_ms: number;
  created_at: number;
  words: WordFeedback[];
}

export interface SpeakingWeakSegment {
  segment_id: number;
  clip_id: number;
  title: string;
  attempts: number;
  accuracy_percent: number;
}

export interface SpeakingStats {
  total_attempts: number;
  today_attempts: number;
  accuracy_percent: number;
  low_confidence_attempts: number;
  total_duration_ms: number;
  weakest_segments: SpeakingWeakSegment[];
}

export const emptyOverview: Overview = {
  total_words: 0,
  due_words: 0,
  learned_words: 0,
  mastered_words: 0,
  total_reviews: 0,
  today_reviews: 0,
  review_queue: [],
};

export const emptyStats: StudyStats = {
  total_reviews: 0,
  correct_reviews: 0,
  accuracy_percent: 0,
  total_duration_min: 0,
  today_reviews: 0,
  today_duration_min: 0,
  due_words: 0,
  mastered_words: 0,
  vocabulary_count: 0,
};

/** 把后端返回的任意错误统一转成用户可理解的中文信息。 */
export function toUserMessage(error: unknown): string {
  if (typeof error === "string" && error.trim()) return error;
  if (error instanceof Error && error.message.trim()) return error.message;
  return "本地数据操作失败，请稍后重试。";
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime) throw new Error(PREVIEW_MESSAGE);
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw new Error(toUserMessage(error));
  }
}

export const api = {
  getReviewQueue: (limit?: number) => invokeCommand<WordCard[]>("get_review_queue", { limit }),
  getWordCatalog: () => invokeCommand<WordCard[]>("get_word_catalog"),
  getOverview: () => invokeCommand<Overview>("get_overview"),
  answerWord: (wordId: number, rating: number, durationMs: number) =>
    invokeCommand<WordCard>("answer_word", { wordId, rating, durationMs }),
  getSentences: (limit?: number) => invokeCommand<SentenceCard[]>("get_sentences", { limit }),
  completeSentence: (sentenceId: number, correct: boolean, durationMs: number) =>
    invokeCommand<SentenceCard>("complete_sentence", { sentenceId, correct, durationMs }),
  getVocabulary: () => invokeCommand<VocabularyEntry[]>("get_vocabulary"),
  addToVocabulary: (wordId: number) => invokeCommand<void>("add_to_vocabulary", { wordId }),
  removeFromVocabulary: (wordId: number) => invokeCommand<void>("remove_from_vocabulary", { wordId }),
  getStats: () => invokeCommand<StudyStats>("get_stats"),
  exportData: () => invokeCommand<ExportResult>("export_data"),
  clearLearningData: () => invokeCommand<ClearResult>("clear_learning_data"),
  getAudioCatalog: () => invokeCommand<AudioCatalogEntry[]>("get_audio_catalog"),
  getListeningDetail: (clipId: number) =>
    invokeCommand<ListeningDetail>("get_listening_detail", { clipId }),
  recordDictation: (clipId: number, segmentId: number, submitted: string, durationMs: number) =>
    invokeCommand<DictationResult>("record_dictation", { clipId, segmentId, submitted, durationMs }),
  getDictationStats: () => invokeCommand<DictationStats>("get_dictation_stats"),
  deleteSeedAudio: (clipId: number) => invokeCommand<DeleteResult>("delete_seed_audio", { clipId }),
  getSpeakingStatus: () => invokeCommand<SpeakingStatus>("get_speaking_status"),
  evaluateSpeaking: (
    clipId: number,
    segmentId: number,
    wav: number[],
    durationMs: number,
  ) => invokeCommand<SpeakingResult>("evaluate_speaking", { clipId, segmentId, wav, durationMs }),
  getSpeakingStats: () => invokeCommand<SpeakingStats>("get_speaking_stats"),
};

