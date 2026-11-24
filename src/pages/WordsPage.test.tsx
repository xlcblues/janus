import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { WordsPage } from "./WordsPage";
import { api } from "../lib/api";
import type { WordCard } from "../lib/api";

function makeCard(over: Partial<WordCard> & { id: number; word: string }): WordCard {
  return {
    phonetic: "/test/",
    meaning: "测试释义",
    example_en: "Test sentence.",
    example_zh: "测试例句。",
    stage: "learning",
    interval_days: 1,
    due_at: 0,
    difficulty: 1,
    in_vocabulary: false,
    mastered_at: null,
    known_streak: 0,
    preview: {
      again_days: 0,
      fuzzy_days: 0,
      known_days: 2,
      known_needs_again: false,
    },
    ...over,
  };
}

vi.mock("../lib/api", () => ({
  PREVIEW_MESSAGE: "当前为浏览器预览模式",
  toUserMessage: (error: unknown) => (error instanceof Error ? error.message : String(error)),
  api: {
    getOverview: vi.fn(),
    getDailyQueue: vi.fn(),
    getWordbooks: vi.fn(),
    getLearningConfig: vi.fn(),
    answerWord: vi.fn(),
    deleteLearningRecords: vi.fn(),
  },
}));

const queue = (cards: WordCard[]) => ({
  cards,
  new_count: 0,
  review_count: cards.length,
  total_done_today: 0,
});

const overview = {
  total_words: 0,
  due_words: 0,
  learned_words: 0,
  mastered_words: 0,
  total_reviews: 0,
  today_reviews: 0,
  review_queue: [],
};

beforeEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.mocked(api.getOverview).mockResolvedValue(overview);
  vi.mocked(api.getWordbooks).mockResolvedValue([
    {
      id: 1,
      code: "senior",
      title: "高中",
      description: "",
      level: "",
      difficulty: 1,
      word_count: 1,
      sort_order: 1,
      lang: "en",
    },
  ]);
  vi.mocked(api.getLearningConfig).mockResolvedValue({
    enabled: true,
    book_codes: ["senior"],
    selection_mode: "random",
    custom_word_ids: [],
    daily_total: 10,
  });
});

describe("WordsPage", () => {
  it("shows a preview notice instead of fake data outside the desktop runtime", async () => {
    vi.mocked(api.getDailyQueue).mockRejectedValue(new Error("当前为浏览器预览模式"));
    render(<WordsPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    expect(await screen.findByText(/浏览器预览模式/)).toBeInTheDocument();
    expect(screen.getByText(/复习队列已清空/)).toBeInTheDocument();
  });

  it("does not show a recorded line after rating a card known", async () => {
    vi.mocked(api.getDailyQueue).mockResolvedValue(
      queue([
        makeCard({ id: 1, word: "alpha" }),
        makeCard({ id: 2, word: "beta" }),
      ]),
    );
    vi.mocked(api.answerWord).mockResolvedValue(
      makeCard({ id: 1, word: "alpha", stage: "reviewing", interval_days: 2 }),
    );
    render(<WordsPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /认识/ }));
    await waitFor(() => expect(screen.queryByText(/已记录「认识」/)).not.toBeInTheDocument());
    expect(await screen.findByText("beta")).toBeInTheDocument();
  });

  it("removes the card from the queue when fuzzy schedules a later review (fuzzy_days > 0)", async () => {
    vi.mocked(api.getDailyQueue).mockResolvedValue(
      queue([
        makeCard({
          id: 1,
          word: "alpha",
          preview: { again_days: 0, fuzzy_days: 2, known_days: 3, known_needs_again: false },
        }),
      ]),
    );
    vi.mocked(api.answerWord).mockResolvedValue(
      makeCard({ id: 1, word: "alpha", stage: "learning", interval_days: 2 }),
    );
    render(<WordsPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /模糊/ }));
    expect(await screen.findByText(/复习队列已清空/)).toBeInTheDocument();
  });

  it("requeues the card within the session when fuzzy means same-session relearn (fuzzy_days === 0)", async () => {
    vi.mocked(api.getDailyQueue).mockResolvedValue(
      queue([
        makeCard({
          id: 1,
          word: "alpha",
          preview: { again_days: 0, fuzzy_days: 0, known_days: 3, known_needs_again: false },
        }),
      ]),
    );
    vi.mocked(api.answerWord).mockResolvedValue(
      makeCard({ id: 1, word: "alpha", stage: "learning", interval_days: 0 }),
    );
    render(<WordsPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /模糊/ }));
    expect(await screen.findByText("alpha")).toBeInTheDocument();
  });

  it("deletes the current word's learning record after confirmation", async () => {
    vi.mocked(api.getDailyQueue).mockResolvedValue(
      queue([makeCard({ id: 1, word: "alpha" }), makeCard({ id: 2, word: "beta" })]),
    );
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    const notify = vi.fn();
    render(<WordsPage onNavigate={vi.fn()} onNotify={notify} />);
    fireEvent.click(await screen.findByRole("button", { name: /删除 alpha 的学习记录/ }));
    expect(api.deleteLearningRecords).toHaveBeenCalledWith(1);
    await waitFor(() => expect(notify).toHaveBeenCalledWith(expect.stringContaining("alpha")));
    confirmSpy.mockRestore();
  });

  it("does not delete the learning record when confirmation is declined", async () => {
    vi.mocked(api.getDailyQueue).mockResolvedValue(
      queue([makeCard({ id: 1, word: "alpha" })]),
    );
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(false);
    render(<WordsPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: /删除 alpha 的学习记录/ }));
    expect(api.deleteLearningRecords).not.toHaveBeenCalled();
    confirmSpy.mockRestore();
  });
});
