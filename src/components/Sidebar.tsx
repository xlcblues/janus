import { CircleHelp, Palette, Settings } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { BrandMark } from "./BrandMark";
import { learningNav, resourceNav } from "../data/navigation";
import type { NavId } from "../data/navigation";

interface SidebarProps {
  activeId: NavId;
  onNavigate: (id: NavId) => void;
}

function NavButton({ id, label, icon: Icon, count, shortcut, active, onNavigate }: {
  id: NavId;
  label: string;
  icon: LucideIcon;
  count?: number;
  shortcut?: string;
  active: boolean;
  onNavigate: (id: NavId) => void;
}) {
  return (
    <button
      type="button"
      className="nav-item"
      data-active={active}
      onClick={() => onNavigate(id)}
      title={shortcut ? `${label}  Ctrl+${shortcut}` : label}
      aria-current={active ? "page" : undefined}
    >
      <Icon size={18} aria-hidden="true" />
      <span className="nav-copy">{label}</span>
      {count ? <span className="nav-count">{count}</span> : null}
    </button>
  );
}

export function Sidebar({ activeId, onNavigate }: SidebarProps) {
  return (
    <aside className="sidebar" aria-label="主导航">
      <div className="sidebar-group">
        <div className="sidebar-group-label">学习</div>
        {learningNav.map((item) => (
          <NavButton key={item.id} {...item} active={item.id === activeId} onNavigate={onNavigate} />
        ))}
      </div>

      <div className="sidebar-rule" aria-hidden="true" />

      <div className="sidebar-group">
        <div className="sidebar-group-label">资料</div>
        {resourceNav.map((item) => (
          <NavButton key={item.id} {...item} active={item.id === activeId} onNavigate={onNavigate} />
        ))}
      </div>

      <div className="sidebar-bottom">
        <div className="streak-card">
          <div className="streak-heading">
            <span>本周连续学习</span>
            <b>5 / 7 天</b>
          </div>
          <div className="streak-track" aria-hidden="true"><i /></div>
        </div>

        <div className="profile-row">
          <div className="avatar" aria-hidden="true">L</div>
          <span className="profile-name">Lin</span>
          <button type="button" className="profile-action" aria-label="设置" title="设置" onClick={() => onNavigate("settings")}>
            <Settings size={16} />
          </button>
        </div>

        <div className="sidebar-signature">
          <BrandMark className="sidebar-mark" />
          <span>越过边界，开口即世界。</span>
        </div>

        <div className="sidebar-small-actions">
          <button type="button" title="界面主题由 Janus Iris 推导" aria-label="品牌说明"><Palette size={15} /></button>
          <button type="button" title="帮助中心将在后续接入" aria-label="帮助"><CircleHelp size={15} /></button>
        </div>
      </div>
    </aside>
  );
}
