import { create } from "zustand";
import * as nunjucks from "nunjucks";
import type { VariableDefinition } from "../../../entities/template/types";
import * as templateApi from "../api";

/** 关闭 HTML 自动转义与未定义变量报错：模板渲染的是纯文本/Markdown 预览。 */
const renderEnv = nunjucks.configure({
  autoescape: false,
  throwOnUndefined: false,
});

/**
 * 模板状态（Zustand store），对应架构方案 3.2 的 templateStore。
 * 职责：变量定义、变量值、预览结果（M3-C2）。
 * 渲染使用前端 Nunjucks（M3 决策点 3.1：渲染在后端之外的「前端 Nunjucks」）。
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
    set({ preview: renderEnv.renderString(template, get().values) });
  },

  clear() {
    set({ variables: [], values: {}, preview: "" });
  },
}));