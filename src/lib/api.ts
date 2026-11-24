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
  difficulty: number;
  in_vocabulary: boolean;
  mastered_at: number | null;
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

export interface MasteredWord {
  word_id: number;
  word: string;
  phonetic: string;
  meaning: string;
  difficulty: number;
  mastered_at: number | null;
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


export interface ConversationSettings {
  base_url: string;
  api_key: string;
  model: string;
}

export interface ConversationScene {
  id: number;
  title: string;
  goal: string;
  role: string;
  user_goal: string;
  target_words: string;
  level: string;
  greeting: string;
  lang: string;
}

export interface ConversationMessage {
  id: number;
  session_id: number;
  role: string;
  content: string;
  correction: string | null;
  latency_ms: number;
  created_at: number;
}

export interface ConversationSessionSummary {
  id: number;
  scene_id: number;
  scene_title: string;
  started_at: number;
  ended_at: number | null;
  turn_count: number;
  summary: string;
  message_count: number;
}

export interface ConversationStartResult {
  session: ConversationSessionSummary;
  greeting: ConversationMessage;
}

export interface ConversationTurnResult {
  user_message: ConversationMessage;
  assistant_message: ConversationMessage;
  turn_count: number;
}

export interface SceneUsage {
  scene_id: number;
  title: string;
  sessions: number;
}

export interface ConversationStats {
  total_sessions: number;
  total_turns: number;
  today_sessions: number;
  today_turns: number;
  recent_scenes: SceneUsage[];
}

export interface Correction {
  original: string;
  suggestion: string;
  note: string;
}

export interface Transcript {
  text: string;
  confidence: number;
  low_confidence: boolean;
}

export interface LlmTestResult {
  ok: boolean;
  latency_ms: number;
  message: string;
}


/* ============================================================
 * M6 · 数据、复习与计划
 * ============================================================ */

export interface DailyStat {
  day_start: number;
  reviews: number;
  dictations: number;
  speaking: number;
  conversation_turns: number;
  duration_min: number;
  accuracy_percent: number;
}

export interface StageCount {
  stage: string;
  count: number;
}

export interface MasteryProgress {
  word_stages: StageCount[];
  sentence_statuses: StageCount[];
  learned_words: number;
  mastered_words: number;
  vocabulary_count: number;
  mastery_percent: number;
}

export interface CorrectionEntry {
  session_id: number;
  scene_title: string;
  original: string;
  suggestion: string;
  note: string;
  created_at: number;
}

export interface WeakWord {
  word_id: number;
  word: string;
  meaning: string;
  wrong_count: number;
  total_count: number;
}

export interface MistakeBook {
  weak_dictation: WeakClip[];
  weak_speaking: SpeakingWeakSegment[];
  conversation_corrections: CorrectionEntry[];
  low_score_words: WeakWord[];
}

export interface StudyPlan {
  enabled: boolean;
  word_target: number;
  sentence_target: number;
  listening_target: number;
  speaking_target: number;
  conversation_target: number;
  duration_min_target: number;
}

export interface ReminderSettings {
  enabled: boolean;
  hour: number;
  minute: number;
}
export interface MicrophoneSettings {
  device_id: string;
  device_label: string;
}

export interface Wordbook {
  id: number;
  code: string;
  title: string;
  description: string;
  level: string;
  difficulty: number;
  word_count: number;
  sort_order: number;
  lang: string;
}

export interface LearningConfig {
  enabled: boolean;
  book_codes: string[];
  selection_mode: string;
  custom_word_ids: number[];
  daily_new: number;
  daily_review: number;
}

export interface DailyQueue {
  cards: WordCard[];
  new_count: number;
  review_count: number;
  total_done_today: number;
}

export interface WordbookCatalogEntry {
  word_id: number;
  word: string;
  phonetic: string;
  meaning: string;
  difficulty: number;
  level: string;
  book_codes: string[];
}

export type NotificationPermissionState = "granted" | "denied" | "prompt";

export interface UpdateCheck {
  current: string;
  latest: string;
  has_update: boolean;
  download_url: string;
  published_at: string | null;
}

export interface TodayProgress {
  word_reviews: number;
  sentences: number;
  dictations: number;
  speaking: number;
  conversation_turns: number;
  duration_min: number;
  due_words: number;
  streak_days: number;
  plan_percent: number;
  plan: StudyPlan;
}

export const emptyConversationSettings: ConversationSettings = {
  base_url: "https://api.openai.com/v1",
  api_key: "",
  model: "gpt-4o-mini",
};

export const emptyConversationStats: ConversationStats = {
  total_sessions: 0,
  total_turns: 0,
  today_sessions: 0,
  today_turns: 0,
  recent_scenes: [],
};

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


export const emptyStudyPlan: StudyPlan = {
  enabled: true,
  word_target: 10,
  sentence_target: 5,
  listening_target: 3,
  speaking_target: 3,
  conversation_target: 4,
  duration_min_target: 18,
};

export const emptyReminderSettings: ReminderSettings = {
  enabled: true,
  hour: 20,
  minute: 0,
};

export const emptyMicrophoneSettings: MicrophoneSettings = {
  device_id: "",
  device_label: "",
};

export const emptyLearningConfig: LearningConfig = {
  enabled: true,
  book_codes: ["senior", "college"],
  selection_mode: "random",
  custom_word_ids: [],
  daily_new: 10,
  daily_review: 20,
};

export const emptyTodayProgress: TodayProgress = {
  word_reviews: 0,
  sentences: 0,
  dictations: 0,
  speaking: 0,
  conversation_turns: 0,
  duration_min: 0,
  due_words: 0,
  streak_days: 0,
  plan_percent: 0,
  plan: emptyStudyPlan,
};

export const emptyMasteryProgress: MasteryProgress = {
  word_stages: [],
  sentence_statuses: [],
  learned_words: 0,
  mastered_words: 0,
  vocabulary_count: 0,
  mastery_percent: 0,
};

export const emptyMistakeBook: MistakeBook = {
  weak_dictation: [],
  weak_speaking: [],
  conversation_corrections: [],
  low_score_words: [],
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
  getWordbooks: () => invokeCommand<Wordbook[]>("get_wordbooks"),
  getLearningConfig: () => invokeCommand<LearningConfig>("get_learning_config"),
  saveLearningConfig: (config: LearningConfig) =>
    invokeCommand<void>("save_learning_config", { config }),
  getDailyQueue: (limit?: number) => invokeCommand<DailyQueue>("get_daily_queue", { limit }),
  getWordbookCatalog: () => invokeCommand<WordbookCatalogEntry[]>("get_wordbook_catalog"),
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
  getMasteredWords: () => invokeCommand<MasteredWord[]>("get_mastered_words"),
  unmasterWord: (wordId: number) => invokeCommand<void>("unmaster_word", { wordId }),
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
  getConversationSettings: () => invokeCommand<ConversationSettings>("get_conversation_settings"),
  saveConversationSettings: (settings: ConversationSettings) =>
    invokeCommand<void>("save_conversation_settings", { settings }),
  testConversationConnection: (settings?: ConversationSettings) =>
    invokeCommand<LlmTestResult>("test_conversation_connection", settings ? { settings } : {}),
  getConversationScenes: () => invokeCommand<ConversationScene[]>("get_conversation_scenes"),
  startConversation: (sceneId: number) =>
    invokeCommand<ConversationStartResult>("start_conversation", { sceneId }),
  getConversationHistory: (sessionId: number) =>
    invokeCommand<ConversationMessage[]>("get_conversation_history", { sessionId }),
  getConversationSessions: (limit?: number) =>
    invokeCommand<ConversationSessionSummary[]>("get_conversation_sessions", { limit }),
  getConversationStats: () => invokeCommand<ConversationStats>("get_conversation_stats"),
  endConversation: (sessionId: number) =>
    invokeCommand<ConversationSessionSummary>("end_conversation", { sessionId }),
  sendConversationMessage: (sessionId: number, content: string) =>
    invokeCommand<ConversationTurnResult>("send_conversation_message", { sessionId, content }),
  analyzeConversationCorrection: (sessionId: number, messageId: number) =>
    invokeCommand<Correction | null>("analyze_conversation_correction", { sessionId, messageId }),
  transcribeSpeech: (wav: number[]) => invokeCommand<Transcript>("transcribe_speech", { wav }),
  getTodayProgress: () => invokeCommand<TodayProgress>("get_today_progress"),
  getWeeklyStats: (days?: number) => invokeCommand<DailyStat[]>("get_weekly_stats", { days }),
  getMasteryProgress: () => invokeCommand<MasteryProgress>("get_mastery_progress"),
  getMistakeBook: () => invokeCommand<MistakeBook>("get_mistake_book"),
  getStudyPlan: () => invokeCommand<StudyPlan>("get_study_plan"),
  saveStudyPlan: (plan: StudyPlan) => invokeCommand<void>("save_study_plan", { plan }),
  getReminderSettings: () => invokeCommand<ReminderSettings>("get_reminder_settings"),
  saveReminderSettings: (settings: ReminderSettings) =>
    invokeCommand<void>("save_reminder_settings", { settings }),
  getMicrophoneSettings: () => invokeCommand<MicrophoneSettings>("get_microphone_settings"),
  saveMicrophoneSettings: (settings: MicrophoneSettings) =>
    invokeCommand<void>("save_microphone_settings", { settings }),
  getNotificationPermission: () =>
    invokeCommand<NotificationPermissionState>("get_notification_permission"),
  sendTestNotification: () => invokeCommand<void>("send_test_notification"),
  getAppVersion: () => invokeCommand<string>("get_app_version"),
  getAppDataDir: () => invokeCommand<string>("get_app_data_dir"),
  checkForUpdate: () => invokeCommand<UpdateCheck>("check_for_update"),
  openUrl: (url: string) => invokeCommand<void>("open_url", { url })
};

