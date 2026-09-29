import { diffWords } from "diff";

/**
 * 版本控制纯函数（shared/lib）。
 *
 * 语义化版本号递增由后端 `VersionRepository` 固化（M3 §3.1），前端只做词级 Diff。
 */

/** 词级差异片段（对齐 diff 库 Change 对象，仅保留渲染所需字段）。 */
export interface WordDiffPart {
  value: string;
  added?: boolean;
  removed?: boolean;
}

/** 计算两个文本的词级差异。 */
export function diffVersionContents(
  before: string,
  after: string,
): WordDiffPart[] {
  return diffWords(before, after).map((part) => ({
    value: part.value,
    added: part.added,
    removed: part.removed,
  }));
}