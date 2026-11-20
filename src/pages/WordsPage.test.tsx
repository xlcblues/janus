import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { WordsPage } from "./WordsPage";

describe("WordsPage", () => {
  it("shows a preview notice instead of fake data outside the desktop runtime", async () => {
    render(<WordsPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    expect(await screen.findByText(/浏览器预览模式/)).toBeInTheDocument();
    expect(screen.getByText(/复习队列已清空/)).toBeInTheDocument();
  });
});
