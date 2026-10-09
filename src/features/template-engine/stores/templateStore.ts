import { create } from "zustand";
import type { VariableDefinition } from "../../../entities/template/types";
import * as templateApi from "../api";

/** 与后端 `template::render` 保持一致的变量占位正则（`{{name}}`，`[^{}\s]+`）。 */
const VAR_RE = /\{\{([^{}\s]+)\}\}/g;

/**
 * 渲染模板：把正文中的 `{{name}}` 回填为 `values` 中同名值；未覆盖的变量保留原占位。
 * 采用与后端一致的正则替换，而非模板引擎，规避 `{{...}}` 被误解析（中文名/多变量/特殊字符）。
 */
function renderTemplate(template: string, values: Record<string, string>): string {
  return template.replace(VAR_RE, (whole, name: string) =>
    Object.prototype.hasOwnProperty.call(values, name) ? values[name] : whole,
  );
}

/**
 * 模板状态（Zustand store），对应架构方案 3.2 的 templateStore。
 * 职责：变量定义、变量值、预览结果（M3-C2）。
 * 渲染与后端复用同一 `{{name}}` 正则契约，避免引入额外模板语义。
 */
export interface TemplateStore {
  /** `extract_variables` 提取出的变量定义。 */
  variables: VariableDefinition[];
  /** 变量名 → 当前填写值。 */
  values: Record<string, string>;
  /** Nunjucks 渲染后的预览结果。 */
  preview: string;
  isLoading: boolean;

  extractVariables: (content: string) => Promise<VariableDefinition[]>;
  setValue: (name: string, value: string) => void;
  setValues: (values: Record<string, string>) => void;
  renderPreview: (template: string) => void;
  clear: () => void;
}

export const useTemplateStore = create<TemplateStore>()((set, get) => ({
  variables: [],
  values: {},
  preview: "",
  isLoading: false,

  async extractVariables(content) {
    set({ isLoading: true });
    try {
      const variables = await templateApi.extractVariables(content);
      const values: Record<string, string> = {};
      for (const v of variables) {
        values[v.name] = v.defaultValue ?? "";
      }
      set({ variables, values });
      return variables;
    } finally {
      set({ isLoading: false });
    }
  },

  setValue(name, value) {
    set((state) => ({ values: { ...state.values, [name]: value } }));
  },

  setValues(values) {
    set({ values });
  },

  renderPreview(template) {
    set({ preview: renderTemplate(template, get().values) });
  },

  clear() {
    set({ variables: [], values: {}, preview: "" });
  },
}));