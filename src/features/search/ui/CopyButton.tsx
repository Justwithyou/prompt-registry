import { useEffect, useRef, useState } from "react";
import { copyText } from "../../../shared/lib/clipboard";
import { Icon } from "../../../shared/ui/Icon";

interface CopyButtonProps {
  /** 要复制到剪贴板的「原始内容」。 */
  text: string;
  label?: string;
}

/**
 * 一键复制按钮（features/search）——「复制原始」入口。
 * 复制结果 1.5s 内以按钮文案反馈，降级方案见 shared/lib/clipboard。
 */
export function CopyButton({ text, label = "复制原始" }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    },
    [],
  );

  async function handleClick() {
    const ok = await copyText(text);
    if (timerRef.current) clearTimeout(timerRef.current);
    setCopied(ok);
    if (ok) {
      timerRef.current = setTimeout(() => setCopied(false), 1500);
    }
  }

  return (
    <button
      type="button"
      className={`copy-button${copied ? " is-copied" : ""}`}
      onClick={handleClick}
    >
      <Icon name={copied ? "check" : "copy"} size={16} />
      {copied ? "已复制" : label}
    </button>
  );
}