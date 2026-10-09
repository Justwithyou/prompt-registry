import { useEffect, useState } from "react";
import type { Project } from "../../../entities/project/types";
import { useProjectStore } from "../stores/projectStore";
import { getPrompts } from "../../../shared/api/commands";
import { Icon } from "../../../shared/ui/Icon";
import { ExportDialog } from "../../import-export/index";

/**
 * 项目管理器（D2 / project-management/ui）。
 * 项目 CRUD + 元数据（名称/描述/颜色）编辑，并展示每个项目下关联的提示词数量。
 * M2-I1 起项目提示词计数来自真实后端（按项目过滤计数），而非 mock 的 projectIds 字段。
 */
export function ProjectManager() {
  const projects = useProjectStore((s) => s.projects);
  const createProject = useProjectStore((s) => s.createProject);
  const updateProject = useProjectStore((s) => s.updateProject);
  const deleteProject = useProjectStore((s) => s.deleteProject);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState("#2f6fed");
  const [showExport, setShowExport] = useState(false);

  const [counts, setCounts] = useState<Record<string, number>>({});

  useEffect(() => {
    let cancelled = false;
    (async () => {
      const map: Record<string, number> = {};
      await Promise.all(
        projects.map(async (p) => {
          map[p.id] = (await getPrompts({ projectId: p.id })).length;
        }),
      );
      if (!cancelled) setCounts(map);
    })();
    return () => {
      cancelled = true;
    };
  }, [projects]);

  function countFor(projectId: string): number {
    return counts[projectId] ?? 0;
  }

  function resetForm() {
    setEditingId(null);
    setCreating(false);
    setName("");
    setDescription("");
    setColor("#2f6fed");
  }

  function startEdit(p: Project) {
    setEditingId(p.id);
    setCreating(false);
    setName(p.name);
    setDescription(p.description ?? "");
    setColor(p.color ?? "#2f6fed");
  }

  function submit() {
    const trimmed = name.trim();
    if (!trimmed) return;
    if (editingId) {
      updateProject(editingId, {
        name: trimmed,
        description: description || null,
        color,
      });
    } else {
      createProject({ name: trimmed, description: description || null, color });
    }
    resetForm();
  }

  return (
    <div className="manager-page">
      <header className="manager-header">
        <div className="page-identity">
          <span className="page-identity-icon">
            <Icon name="briefcase" size={18} />
          </span>
          <div className="page-identity-text">
            <span className="page-title">项目管理</span>
            <span className="page-subtitle">共 {projects.length} 个项目</span>
          </div>
        </div>
        <button
          type="button"
          className="btn-ghost"
          onClick={() => setShowExport(true)}
        >
          <Icon name="download" size={16} />
          批量导出
        </button>
        <button
          type="button"
          className="btn-primary"
          onClick={() => {
            resetForm();
            setCreating(true);
          }}
        >
          <Icon name="plus" size={16} />
          新建项目
        </button>
      </header>

      {creating || editingId ? (
        <div className="project-form">
          <input
            className="project-form-name"
            value={name}
            placeholder="项目名称"
            autoFocus
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && submit()}
          />
          <input
            className="project-form-desc"
            value={description}
            placeholder="项目描述（可选）"
            onChange={(e) => setDescription(e.target.value)}
          />
          <label className="project-form-color">
            <span>颜色</span>
            <input type="color" value={color} onChange={(e) => setColor(e.target.value)} />
          </label>
          <button type="button" className="btn-primary" onClick={submit}>
            {editingId ? "保存" : "创建"}
          </button>
          <button type="button" className="btn-ghost" onClick={resetForm}>
            取消
          </button>
        </div>
      ) : null}

      {projects.length === 0 ? (
        <div className="project-empty">
          <div className="manager-empty">暂无项目，点击「新建项目」开始。</div>
        </div>
      ) : (
        <div className="project-grid">
          {projects.map((project) => (
            <div key={project.id} className="project-card">
              <div className="project-card-top">
                <span
                  className="project-swatch"
                  style={{ background: project.color ?? "var(--accent)" }}
                />
                <span className="project-card-name">{project.name}</span>
                <span className="project-card-actions">
                  <button
                    type="button"
                    className="icon-btn"
                    title="编辑"
                    onClick={() => startEdit(project)}
                  >
                    <Icon name="edit" size={14} />
                  </button>
                  <button
                    type="button"
                    className="icon-btn is-danger"
                    title="删除"
                    onClick={() => deleteProject(project.id)}
                  >
                    <Icon name="trash" size={14} />
                  </button>
                </span>
              </div>
              <p className="project-card-desc">
                {project.description || "暂无描述"}
              </p>
              <div className="project-card-footer">
                <span className="muted">{countFor(project.id)} 条提示词</span>
              </div>
            </div>
          ))}
        </div>
      )}

      {showExport && (
        <ExportDialog
          title="按项目批量导出"
          dimension="project"
          options={projects.map((p) => ({ id: p.id, name: p.name }))}
          onClose={() => setShowExport(false)}
        />
      )}
    </div>
  );
}