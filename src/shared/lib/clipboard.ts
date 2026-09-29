/**
 * 剪贴板工具（shared/lib）—— 一键复制「原始内容」的底层逻辑。
 * M1 阶段「搜索 / 复制」切片（features/search）与编辑器均复用此单一实现。
 *
 * 说明：Tauri v2 桌面窗口默认属于安全上下文，优先走 `navigator.clipboard`；
 * 若被拒绝（如未授予 clipboard 能力），降级为临时 textarea + execCommand。
 */
export async function copyText(text: string): Promise<boolean> {
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // 落入降级路径
    }
  }

  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.top = "0";
    ta.style.left = "0";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.focus();
    ta.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}