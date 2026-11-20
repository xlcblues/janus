import type { LucideIcon } from "lucide-react";
import {
  BookOpen,
  CalendarDays,
  ChartNoAxesCombined,
  Headphones,
  MessagesSquare,
  Mic,
  NotebookPen,
  Settings,
  Sparkles,
} from "lucide-react";

export type NavId =
  | "today"
  | "words"
  | "sentences"
  | "listening"
  | "speaking"
  | "conversation"
  | "notebook"
  | "analytics"
  | "settings";

export type NavGroup = "learning" | "resources";

export interface NavItem {
  id: NavId;
  label: string;
  group: NavGroup;
  icon: LucideIcon;
  count?: number;
  shortcut?: string;
  description: string;
  stage: string;
}

export const navItems: NavItem[] = [
  { id: "today", label: "今日学习", group: "learning", icon: CalendarDays, shortcut: "1", description: "今日路线、到期复习与学习节奏。", stage: "工作台" },
  { id: "words", label: "单词", group: "learning", icon: BookOpen, count: 24, shortcut: "2", description: "间隔重复、词义、语境与生词收集。", stage: "M2" },
  { id: "sentences", label: "句子", group: "learning", icon: MessagesSquare, shortcut: "3", description: "语块拆解、理解、跟读与主动回忆。", stage: "M2" },
  { id: "listening", label: "听力", group: "learning", icon: Headphones, count: 12, shortcut: "4", description: "精听、逐句定位、变速与听写。", stage: "M3" },
  { id: "speaking", label: "朗读", group: "learning", icon: Mic, shortcut: "5", description: "录音、回放、发音提示与表达校准。", stage: "M3" },
  { id: "conversation", label: "AI 对话", group: "learning", icon: Sparkles, shortcut: "6", description: "实时语音、纠错反馈与场景对话。", stage: "M4" },
  { id: "notebook", label: "生词本", group: "resources", icon: NotebookPen, count: 24, description: "集中管理生词、错句与待复习内容。", stage: "M2" },
  { id: "analytics", label: "学习数据", group: "resources", icon: ChartNoAxesCombined, description: "学习时间、掌握度、错误趋势与会话记录。", stage: "M2" },
  { id: "settings", label: "设置", group: "resources", icon: Settings, description: "主题、学习计划、语音设备与隐私设置。", stage: "M1" },
];

export const learningNav = navItems.filter((item) => item.group === "learning");
export const resourceNav = navItems.filter((item) => item.group === "resources");

export function findNavItem(id: NavId): NavItem {
  return navItems.find((item) => item.id === id) ?? navItems[0];
}
