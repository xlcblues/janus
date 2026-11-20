import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

describe("api.evaluateSpeaking IPC contract", () => {
  beforeEach(() => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    invokeMock.mockReset();
    invokeMock.mockResolvedValue({ id: 1 });
  });

  afterEach(() => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("forwards clipId, segmentId, wav bytes and durationMs to the backend command", async () => {
    const { api } = await import("./api");
    await api.evaluateSpeaking(3, 5, [1, 2, 3, 4], 2500);
    expect(invokeMock).toHaveBeenCalledWith("evaluate_speaking", {
      clipId: 3,
      segmentId: 5,
      wav: [1, 2, 3, 4],
      durationMs: 2500,
    });
  });
});