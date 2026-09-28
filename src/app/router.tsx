/**
 * 路由编排（FSD app 层）。
 *
 * M0 骨架阶段：由 App.tsx 直接渲染，路由留待 M1 引入 react-router 后在此编排。
 * 目标路由表见《架构设计方案》3.3：
 *   /prompts、/prompts/:id、/prompts/:id/versions、/projects、/tags、
 *   /search、/sync、/settings/* 等。
 */
export const routes = [] as const;