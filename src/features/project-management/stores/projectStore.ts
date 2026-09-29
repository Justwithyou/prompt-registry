import { create } from "zustand";
import type {
  Project,
  CreateProjectInput,
  UpdateProjectInput,
} from "../../../entities/project/types";
import * as projectApi from "../api";

/**
 * 项目状态（Zustand store），对应架构方案 3.2 的 projectStore。
 * 维护项目列表、当前选中项目与加载态，提供 CRUD action（M2-C2）。
 */
export interface ProjectStore {
  projects: Project[];
  selectedProjectId: string | null;
  isLoading: boolean;

  fetchProjects: () => Promise<void>;
  selectProject: (id: string | null) => void;
  createProject: (data: CreateProjectInput) => Promise<Project>;
  updateProject: (id: string, data: UpdateProjectInput) => Promise<void>;
  deleteProject: (id: string) => Promise<void>;
}

export const useProjectStore = create<ProjectStore>()((set) => ({
  projects: [],
  selectedProjectId: null,
  isLoading: false,

  async fetchProjects() {
    set({ isLoading: true });
    try {
      const projects = await projectApi.listProjects();
      set({ projects });
    } finally {
      set({ isLoading: false });
    }
  },

  selectProject(id) {
    set({ selectedProjectId: id });
  },

  async createProject(data) {
    const created = await projectApi.createProject(data);
    set((state) => ({ projects: [...state.projects, created] }));
    return created;
  },

  async updateProject(id, data) {
    const updated = await projectApi.updateProject(id, data);
    set((state) => ({
      projects: state.projects.map((p) => (p.id === id ? updated : p)),
    }));
  },

  async deleteProject(id) {
    await projectApi.deleteProject(id);
    set((state) => ({
      projects: state.projects.filter((p) => p.id !== id),
      selectedProjectId:
        state.selectedProjectId === id ? null : state.selectedProjectId,
    }));
  },
}));