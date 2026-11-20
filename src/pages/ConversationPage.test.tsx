import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ConversationPage } from "./ConversationPage";

describe("ConversationPage", () => {
  it("shows a preview notice instead of fake conversation data outside the desktop runtime", async () => {
    render(<ConversationPage onNavigate={vi.fn()} onNotify={vi.fn()} />);
    expect(await screen.findByText(/浏览器预览模式/)).toBeInTheDocument();
    expect(screen.queryByText(/正在加载对话模块/)).not.toBeInTheDocument();
    expect(screen.queryByText(/开始对话/)).not.toBeInTheDocument();
  });
});
