import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AnalyticsPage } from "./AnalyticsPage";
import { SettingsPage } from "./SettingsPage";

describe("M6 data & settings pages", () => {
  it("AnalyticsPage renders weekly trend, mastery and mistake book sections", async () => {
    render(<AnalyticsPage onNotify={vi.fn()} />);
    expect(await screen.findByRole("heading", { name: "学习数据" })).toBeInTheDocument();
    expect(screen.getByText("周趋势")).toBeInTheDocument();
    expect(screen.getByText("mastery")).toBeInTheDocument();
    expect(screen.getByText("错题本")).toBeInTheDocument();
  });

  it("SettingsPage renders wordbook, reminder and voice device panels", async () => {
    render(<SettingsPage onNotify={vi.fn()} />);
    expect(await screen.findByRole("heading", { name: "设置" })).toBeInTheDocument();
    expect(screen.getByText("wordbook")).toBeInTheDocument();
    expect(screen.getByText("reminder")).toBeInTheDocument();
    expect(screen.getByText("microphone")).toBeInTheDocument();
    expect(screen.getAllByText(/桌面通知/).length).toBeGreaterThan(0);
    expect(screen.getByText(/系统默认麦克风/)).toBeInTheDocument();
    expect(screen.getByText("版本与更新")).toBeInTheDocument();
    expect(screen.getByText("检查更新")).toBeInTheDocument();
    expect(screen.getByText(/云同步接口将在后续里程碑接入/)).toBeInTheDocument();
  });
});
