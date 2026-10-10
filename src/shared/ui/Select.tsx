import { useEffect, useRef, useState } from "react";
import { Icon } from "./Icon";
import { useEscapeKey } from "../hooks/useEscapeKey";

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

interface SelectProps {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  /** 当前值可能为空或未匹配时，触发器显示的占位文案（如「全部分类」「打标签…」）。 */
  placeholder?: string;
  /** 额外作用到触发器上的类名（沿用原 select 的场景类，如 filter-select / tag-form-parent）。 */
  className?: string;
  disabled?: boolean;
}

/**
 * 自绘下拉选择器（shared/ui）。
 * 替代原生 <select>：原生下拉面板是系统渲染，无法用 CSS 控制悬停/选中高亮色；
 * 自绘为 DOM 弹层后，悬停与选中项可统一为品牌蓝（天空蓝），风格与整体主题一致。
 */
export function Select({
  options,
  value,
  onChange,
  placeholder,
  className,
  disabled,
}: SelectProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);

  const selected = options.find((o) => o.value === value);
  const display = selected?.label ?? placeholder ?? "";

  const close = () => setOpen(false);

  useEscapeKey(close, open);

  useEffect(() => {
    if (!open) return;
    function onDocPointerDown(e: MouseEvent) {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", onDocPointerDown);
    return () => document.removeEventListener("mousedown", onDocPointerDown);
  }, [open]);

  return (
    <div className="select" ref={rootRef}>
      <button
        type="button"
        className={`select-trigger${className ? " " + className : ""}`}
        onClick={() => setOpen((v) => !v)}
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={open}
      >
        <span className="select-value">{display}</span>
        <Icon name="chevron-down" size={14} className="select-caret" />
      </button>

      {open && (
        <div className="select-menu" role="listbox">
          {options.map((o) => (
            <button
              key={o.value}
              type="button"
              role="option"
              aria-selected={o.value === value}
              disabled={o.disabled}
              className={`select-option${o.value === value ? " is-selected" : ""}`}
              onClick={() => {
                onChange(o.value);
                close();
              }}
            >
              <span className="select-option-label">{o.label}</span>
              {o.value === value && <Icon name="check" size={14} />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}