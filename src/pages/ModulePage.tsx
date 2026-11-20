import { ArrowLeft, CircleCheck, Clock3, Layers3, ShieldCheck } from "lucide-react";
import { findNavItem } from "../data/navigation";
import { moduleBoundaries, moduleSteps } from "../data/modules";
import type { NavId } from "../data/navigation";

interface ModulePageProps {
  id: NavId;
  onNavigate: (id: NavId) => void;
  onNotify: (message: string) => void;
}

export function ModulePage({ id, onNavigate, onNotify }: ModulePageProps) {
  const item = findNavItem(id);
  const Icon = item.icon;

  return (
    <div className="module-page">
      <section className="page-heading module-heading">
        <div>
          <p className="eyebrow">模块底座 · {item.stage}</p>
          <h1>{item.label}</h1>
          <p>{item.description}</p>
        </div>
        <span className="stage-badge">{item.stage === "M1" ? "应用壳已接入" : "等待数据层"}</span>
      </section>

      <section className="module-hero">
        <div className="module-icon"><Icon size={28} /></div>
        <div className="module-hero-copy">
          <span className="module-kicker">M1 / FOUNDATION</span>
          <h2>入口已就位，学习闭环将在下一阶段接入。</h2>
          <p>当前版本优先验证窗口、导航、主题与响应式布局。这里不会用静态评分或模拟对话掩盖尚未完成的真实能力。</p>
          <div className="module-actions">
            <button type="button" className="button-primary" onClick={() => onNotify(`${item.label}的本地学习闭环将在 M2–M4 分阶段接入。`)}>
              查看阶段计划
            </button>
            <button type="button" className="button-secondary" onClick={() => onNavigate("today")}>
              <ArrowLeft size={14} /> 返回工作台
            </button>
          </div>
        </div>
        <div className="module-gate" aria-hidden="true">
          <span />
          <i />
        </div>
      </section>

      <section className="module-milestones" aria-label="模块里程碑">
        {moduleSteps.map((step) => (
          <article className={`milestone-card ${step.status}`} key={step.index}>
            <div className="milestone-top">
              <span>{step.index}</span>
              {step.status === "done" ? <CircleCheck size={16} /> : step.status === "next" ? <Clock3 size={16} /> : <Layers3 size={16} />}
            </div>
            <h3>{step.title}</h3>
            <p>{step.detail}</p>
          </article>
        ))}
      </section>

      <section className="boundary-panel">
        <div className="boundary-title"><ShieldCheck size={18} /><b>当前阶段边界</b></div>
        <ul>
          {moduleBoundaries.map((boundary) => <li key={boundary}>{boundary}</li>)}
        </ul>
      </section>
    </div>
  );
}
