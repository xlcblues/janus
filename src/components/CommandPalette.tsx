import { useEffect, useMemo, useRef, useState } from "react";
import { Search, X } from "lucide-react";
import { navItems } from "../data/navigation";
import type { NavId } from "../data/navigation";

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  onSelect: (id: NavId) => void;
}

export function CommandPalette({ open, onClose, onSelect }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const results = useMemo(() => {
    const normalized = query.trim().toLocaleLowerCase();
    if (!normalized) return navItems;
    return navItems.filter((item) =>
      `${item.label} ${item.description} ${item.stage}`.toLocaleLowerCase().includes(normalized),
    );
  }, [query]);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setActiveIndex(0);
    requestAnimationFrame(() => inputRef.current?.focus());
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  if (!open) return null;

  const select = (id: NavId) => {
    onSelect(id);
    onClose();
  };

  return (
    <div className="palette-backdrop" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <section className="command-palette" role="dialog" aria-modal="true" aria-label="搜索和快速跳转">
        <div className="palette-search">
          <Search size={17} aria-hidden="true" />
          <input
            ref={inputRef}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") onClose();
              if (event.key === "ArrowDown") {
                event.preventDefault();
                setActiveIndex((index) => Math.min(index + 1, results.length - 1));
              }
              if (event.key === "ArrowUp") {
                event.preventDefault();
                setActiveIndex((index) => Math.max(index - 1, 0));
              }
              if (event.key === "Enter" && results[activeIndex]) select(results[activeIndex].id);
            }}
            placeholder="搜索模块、学习内容或设置"
            aria-label="搜索"
            aria-activedescendant={results[activeIndex] ? `palette-${results[activeIndex].id}` : undefined}
          />
          <kbd>Ctrl K</kbd>
          <button type="button" className="palette-close" onClick={onClose} aria-label="关闭搜索"><X size={16} /></button>
        </div>

        <div className="palette-results" role="listbox" aria-label="搜索结果">
          {results.length ? results.map((item, index) => {
            const Icon = item.icon;
            return (
              <button
                type="button"
                id={`palette-${item.id}`}
                role="option"
                aria-selected={index === activeIndex}
                className="palette-result"
                data-active={index === activeIndex}
                key={item.id}
                onMouseEnter={() => setActiveIndex(index)}
                onClick={() => select(item.id)}
              >
                <span className="palette-result-icon"><Icon size={17} /></span>
                <span><b>{item.label}</b><small>{item.description}</small></span>
                <span className="palette-stage">{item.stage}</span>
              </button>
            );
          }) : <div className="palette-empty">没有匹配的模块。试试“单词”、“听力”或“主题”。</div>}
        </div>

        <div className="palette-footer"><span>↑↓ 选择</span><span>Enter 打开</span><span>Esc 关闭</span></div>
      </section>
    </div>
  );
}
