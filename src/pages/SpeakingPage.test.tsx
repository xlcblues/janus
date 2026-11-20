import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SpeakingPage } from "./SpeakingPage";

describe("SpeakingPage", () => {
  it("shows a preview notice instead of fake speech data outside the desktop runtime", async () => {
    render(<SpeakingPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    expect(await screen.findByText(/浏览器预览模式/)).toBeInTheDocument();
    expect(screen.queryByText(/正在准备朗读模块/)).not.toBeInTheDocument();
  });
});
