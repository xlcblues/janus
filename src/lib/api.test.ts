import { describe, expect, it } from "vitest";
import { api, PREVIEW_MESSAGE, toUserMessage } from "./api";

describe("api browser preview fallback", () => {
  it("throws the preview message for reads outside the Tauri runtime", async () => {
    await expect(api.getOverview()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getStats()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getReviewQueue()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getSentences()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getVocabulary()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getAudioCatalog()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getListeningDetail(1)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getDictationStats()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getSpeakingStatus()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getSpeakingStats()).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.getMasteredWords()).rejects.toThrow(PREVIEW_MESSAGE);
  });

  it("throws the preview message for mutations outside the Tauri runtime", async () => {
    await expect(api.answerWord(1, 3, 1000)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.completeSentence(1, true, 1000)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.addToVocabulary(1)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.recordDictation(1, 2, "hello", 5000)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.deleteSeedAudio(1)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.evaluateSpeaking(1, 2, [0, 0, 0], 1000)).rejects.toThrow(PREVIEW_MESSAGE);
    await expect(api.unmasterWord(1)).rejects.toThrow(PREVIEW_MESSAGE);
  });
});

describe("toUserMessage", () => {
  it("keeps backend string errors", () => {
    expect(toUserMessage("没有找到对应的学习项，请刷新后重试。")).toBe("没有找到对应的学习项，请刷新后重试。");
  });

  it("reads error messages from thrown Error objects", () => {
    expect(toUserMessage(new Error("本地数据操作失败"))).toBe("本地数据操作失败");
  });

  it("falls back to a generic message for unknown payloads", () => {
    expect(toUserMessage(42)).toBe("本地数据操作失败，请稍后重试。");
    expect(toUserMessage(null)).toBe("本地数据操作失败，请稍后重试。");
  });
});
