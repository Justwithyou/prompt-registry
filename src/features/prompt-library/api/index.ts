/**
 * 本切片的 IPC 调用封装。
 * M1 起在此封装 shared/api/commands 的命令（get_prompts / create_prompt 等），
 * 供 stores/hooks 使用，避免 store 直接散落 invoke 调用。
 */
export {};