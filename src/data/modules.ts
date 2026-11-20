export interface ModuleStep {
  index: string;
  title: string;
  detail: string;
  status: "done" | "next" | "later";
}

export const moduleSteps: ModuleStep[] = [
  { index: "01", title: "界面骨架", detail: "导航、布局、主题与模块入口已接入应用壳。", status: "done" },
  { index: "02", title: "本地数据层", detail: "下一阶段接入 SQLite 迁移、学习事件与恢复机制。", status: "next" },
  { index: "03", title: "学习闭环", detail: "完成一次可解释、可恢复、可复习的核心练习。", status: "later" },
];

export const moduleBoundaries = [
  "本阶段只验收桌面壳层，不提供课程内容与评分结果。",
  "AI、录音和网络能力必须通过设备与供应商技术验证后接入。",
  "所有学习数据默认本地保存，并要求可导出、可删除。",
];
