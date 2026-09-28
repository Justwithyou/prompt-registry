import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 期望 dev 服务器端口固定为 1420（与 tauri.conf.json 的 devUrl 对齐）
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tauri 桌面窗口不需要监听 src-tauri 变更，避免额外重启
      ignored: ["**/src-tauri/**"],
    },
  },
});