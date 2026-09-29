import type { PromptVersion } from "../../../entities/version/types";
import { diffVersionContents } from "../../../shared/lib/versioning";

interface DiffViewerProps {
  before: PromptVersion;
  after: PromptVersion;
}

/**
 * 并排 Diff 视图（version-control/ui，M3 D3）。
 * 词级差异：删除（旧侧）标红、新增（新侧）标绿，相同文本中性展示。
 */
export function DiffViewer({ before, after }: DiffViewerProps) {
  const parts = diffVersionContents(before.content, after.content);

  return (
    <div className="diff-viewer">
      <div className="diff-col">
        <div className="diff-col-header">旧 · {before.version}</div>
        <div className="diff-body">
          {parts.map((part, i) =>
            part.added ? null : (
              <span key={i} className={part.removed ? "diff-del" : undefined}>
                {part.value}
              </span>
            ),
          )}
        </div>
      </div>
      <div className="diff-col">
        <div className="diff-col-header">新 · {after.version}</div>
        <div className="diff-body">
          {parts.map((part, i) =>
            part.removed ? null : (
              <span key={i} className={part.added ? "diff-add" : undefined}>
                {part.value}
              </span>
            ),
          )}
        </div>
      </div>
    </div>
  );
}