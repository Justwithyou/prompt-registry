import { useEffect } from "react";
import { useParams } from "react-router";
import type { PromptVersion } from "../../../entities/version/types";
import { useVersionStore } from "../stores/versionStore";
import { Icon } from "../../../shared/ui/Icon";
import { DiffViewer } from "./DiffViewer";

function formatTime(iso: string): string {
  return new Date(iso).toLocaleString();
}

/**
 * 版本历史面板（version-control/ui，M3 D2）。
 *
 * 时间轴展示版本号 / 变更摘要 / 时间戳；勾选两个版本启用并排 Diff；
 * 每个版本提供「回滚」入口（追加式，生成新版本）。
 */
export function VersionHistoryPanel() {
  const { id } = useParams();
  const promptId = id ?? "";

  const versions = useVersionStore((s) => s.versions);
  const diffSelection = useVersionStore((s) => s.diffSelection);
  const isLoading = useVersionStore((s) => s.isLoading);
  const fetchVersions = useVersionStore((s) => s.fetchVersions);
  const toggleDiffSelection = useVersionStore((s) => s.toggleDiffSelection);
  const rollback = useVersionStore((s) => s.rollback);

  useEffect(() => {
    if (promptId) void fetchVersions(promptId);
  }, [promptId, fetchVersions]);

  // 选中两个版本，按时间升序排列，旧版本在左（before）、新版本在右（after）。
  const selected = diffSelection
    .map((vid) => versions.find((v) => v.id === vid))
    .filter((v): v is PromptVersion => v != null)
    .sort((a, b) => a.createdAt.localeCompare(b.createdAt));

  const before = selected[0] ?? null;
  const after = selected[1] ?? null;

  return (
    <div className="version-panel">
      <h3 className="panel-title">
        <Icon name="history" size={18} />
        版本历史
      </h3>
      <p className="panel-subtitle">每次保存自动记录，勾选两个版本可并排对比差异</p>

      {after ? (
        <DiffViewer before={before} after={after} />
      ) : (
        <p className="panel-hint">勾选两个版本以并排对比（旧版本在左，新版本在右）。</p>
      )}

      {versions.length === 0 ? (
        <p className="panel-hint">
          {isLoading ? "加载中…" : "暂无历史版本，保存提示词后自动记录。"}
        </p>
      ) : (
        <ol className="version-timeline">
          {versions.map((v) => {
            const checked = diffSelection.includes(v.id);
            return (
              <li
                key={v.id}
                className={`version-item${checked ? " is-selected" : ""}`}
              >
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={() => toggleDiffSelection(v.id)}
                />
                <span className="version-badge">{v.version}</span>
                <div className="version-meta">
                  <span className="version-summary">
                    {v.changeSummary ?? "（无摘要）"}
                  </span>
                  <span className="version-time">{formatTime(v.createdAt)}</span>
                </div>
                <button
                  type="button"
                  className="btn-ghost version-rollback"
                  onClick={() => void rollback(promptId, v.id)}
                >
                  <Icon name="rotate-ccw" size={14} />
                  回滚
                </button>
              </li>
            );
          })}
        </ol>
      )}
    </div>
  );
}