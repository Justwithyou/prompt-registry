import { useState, type ReactNode } from "react";
import { Icon } from "./Icon";

interface CollapsibleSectionProps {
  /** 模块标题。 */
  title: string;
  /** 模块内容（收起时不渲染）。 */
  children: ReactNode;
  /** 右侧附加操作（常显，不随收起隐藏）。 */
  headerRight?: ReactNode;
  /** 初始是否展开，默认收起。 */
  defaultOpen?: boolean;
  /** 受控展开态（传入则由父级控制，用于设置页手风琴）。 */
  open?: boolean;
  /** 受控切换回调。 */
  onToggle?: () => void;
}

/**
 * 可折叠的设置模块（shared/ui）。
 * 提供「标题 + 展开/收起箭头」的头部与可折叠内容区，用于把冗长的设置页按模块收拢（默认收起）。
 * 支持受控（open/onToggle），父组件可实现「展开一个自动收起其它」的手风琴效果。
 */
export function CollapsibleSection({
  title,
  children,
  headerRight,
  defaultOpen = false,
  open,
  onToggle,
}: CollapsibleSectionProps) {
  const [internalOpen, setInternalOpen] = useState(defaultOpen);
  const isOpen = open ?? internalOpen;

  function toggle() {
    if (onToggle) {
      onToggle();
    } else {
      setInternalOpen((v) => !v);
    }
  }

  return (
    <section className="settings-section">
      <div className="settings-section-header">
        <button
          type="button"
          className="settings-section-toggle"
          onClick={toggle}
          aria-expanded={isOpen}
        >
          <Icon name={isOpen ? "chevron-down" : "chevron-right"} size={16} />
          <span className="settings-section-title">{title}</span>
        </button>
        {headerRight}
      </div>
      {isOpen && <div className="settings-section-body">{children}</div>}
    </section>
  );
}