/**
 * 轻量 Markdown 渲染器（prompt-editor/lib）。
 *
 * M1 仅做常见语法的预览，覆盖：标题、段落、无序/有序列表、引用、围栏代码块、
 * 分隔线、行内代码、粗体、斜体、链接。所有文本先做 HTML 转义，避免注入。
 * 后续里程碑可替换为 `react-markdown` 等成熟实现。
 */
export function renderMarkdown(md: string): string {
  const escapeHtml = (s: string): string =>
    s
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");

  const inline = (s: string): string => {
    let out = escapeHtml(s);
    // 行内代码
    out = out.replace(/`([^`]+)`/g, "<code>$1</code>");
    // 链接 [text](url)
    out = out.replace(
      /\[([^\]]+)\]\(([^)\s]+)\)/g,
      '<a href="$2" target="_blank" rel="noreferrer">$1</a>',
    );
    // 粗体 **x**
    out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
    // 斜体 *x*
    out = out.replace(/(^|[^*])\*([^*]+)\*(?!\*)/g, "$1<em>$2</em>");
    return out;
  };

  const lines = md.split(/\r?\n/);
  const html: string[] = [];
  let paragraph: string[] = [];
  let i = 0;

  const flushParagraph = () => {
    if (paragraph.length > 0) {
      html.push(`<p>${inline(paragraph.join(" "))}</p>`);
      paragraph = [];
    }
  };

  while (i < lines.length) {
    const line = lines[i];

    // 围栏代码块 ```lang ... ```
    const fence = line.match(/^\s*```(.*)$/);
    if (fence) {
      flushParagraph();
      const lang = fence[1].trim();
      const code: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].trim().startsWith("```")) {
        code.push(lines[i]);
        i += 1;
      }
      i += 1; // 跳过结尾 ```
      html.push(
        `<pre><code class="language-${escapeHtml(lang)}">${escapeHtml(
          code.join("\n"),
        )}</code></pre>`,
      );
      continue;
    }

    // 标题
    const heading = line.match(/^\s*(#{1,6})\s+(.*)$/);
    if (heading) {
      flushParagraph();
      const level = heading[1].length;
      html.push(`<h${level}>${inline(heading[2])}</h${level}>`);
      i += 1;
      continue;
    }

    // 分隔线
    if (/^\s*(-{3,}|\*{3,})\s*$/.test(line)) {
      flushParagraph();
      html.push("<hr/>");
      i += 1;
      continue;
    }

    // 无序列表
    if (/^\s*[-*+]\s+/.test(line)) {
      flushParagraph();
      const items: string[] = [];
      while (i < lines.length) {
        const m = lines[i].match(/^\s*[-*+]\s+(.*)$/);
        if (!m) break;
        items.push(`<li>${inline(m[1])}</li>`);
        i += 1;
      }
      html.push(`<ul>${items.join("")}</ul>`);
      continue;
    }

    // 有序列表
    if (/^\s*\d+[.)]\s+/.test(line)) {
      flushParagraph();
      const items: string[] = [];
      while (i < lines.length) {
        const m = lines[i].match(/^\s*\d+[.)]\s+(.*)$/);
        if (!m) break;
        items.push(`<li>${inline(m[1])}</li>`);
        i += 1;
      }
      html.push(`<ol>${items.join("")}</ol>`);
      continue;
    }

    // 引用
    if (/^\s*>\s?/.test(line)) {
      flushParagraph();
      const items: string[] = [];
      while (i < lines.length) {
        const m = lines[i].match(/^\s*>\s?(.*)$/);
        if (!m) break;
        items.push(inline(m[1]));
        i += 1;
      }
      html.push(`<blockquote>${items.join("<br/>")}</blockquote>`);
      continue;
    }

    // 空行结束段落
    if (line.trim() === "") {
      flushParagraph();
      i += 1;
      continue;
    }

    paragraph.push(line.trim());
    i += 1;
  }
  flushParagraph();

  return html.join("");
}