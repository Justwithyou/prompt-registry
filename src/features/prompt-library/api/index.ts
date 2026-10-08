/**
 * 本切片的 IPC 调用封装。
 * 统一从 shared/api/commands 再导出提示词相关命令，供 stores/hooks 使用，
 * 避免 store 直接散落 invoke 调用、越层访问 shared 的实现细节。
 */
export {
  getPrompts,
  createPrompt,
  updatePrompt,
  deletePrompt,
  searchPrompts,
  setPromptRelations,
  getPromptRelations,
  batchUpdatePrompts,
} from "../../../shared/api/commands";