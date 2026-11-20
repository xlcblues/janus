import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ListeningPage } from "./ListeningPage";

describe("ListeningPage", () => {
  it("shows a preview notice instead of fake audio outside the desktop runtime", async () => {
    render(<ListeningPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    expect(await screen.findByText(/浏览器预览模式/)).toBeInTheDocument();
    expect(screen.queryByText(/正在读取本地音频库/)).not.toBeInTheDocument();
  });
});
