/**
 * template-engine 功能切片：模板变量编辑与使用。
 * 对应架构方案 3.2 的 templateStore。
 *
 * 公开 API：templateStore（M3 C2）、TemplateUsePanel（M3 D1 模板使用 UI，
 * 供 app 层组合注入编辑器）。
 */
export { useTemplateStore } from "./stores/templateStore";
export type { TemplateStore } from "./stores/templateStore";
export { TemplateUsePanel } from "./ui/TemplateUsePanel";