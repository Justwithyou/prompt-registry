import { Icon, type IconName } from "./Icon";

interface EmptyStateProps {
  /** 空状态图标名称（shared/ui/Icon）。 */
  icon: IconName;
  /** 主标题。 */
  title: string;
  /** 辅助说明文字。 */
  description?: string;
  /** 主行动按钮文案；不传则无按钮。 */
  actionLabel?: string;
  /** 点击主行动按钮回调。 */
  onAction?: () => void;
}

/**
 * 统一空状态（shared/ui EmptyState）。
 * 结构固定为「图标 + 标题 + 说明 + 主 CTA」，替换各处「暂无 xxx」的散落文案（对应 UI 优化 E1）。
 */
export function EmptyState({
  icon,
  title,
  description,
  actionLabel,
  onAction,
}: EmptyStateProps) {
  return (
    <div className="empty-state">
      <span className="empty-state-icon">
        <Icon name={icon} size={30} />
      </span>
      <span className="empty-state-title">{title}</span>
      {description && <span className="empty-state-desc">{description}</span>}
      {actionLabel && onAction && (
        <button type="button" className="btn-primary" onClick={onAction}>
          <Icon name="plus" size={16} />
          {actionLabel}
        </button>
      )}
    </div>
  );
}