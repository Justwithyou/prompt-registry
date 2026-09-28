import { useState } from "react";
import { ping } from "../shared/api/commands";

/**
 * M0 骨架根组件：验证「React 按钮 → Tauri invoke → Rust 命令 → 回传渲染」最小链路。
 * M1 起由 app/router.tsx 接管路由，本组件仅作连通性占位。
 */
export default function App() {
  const [message, setMessage] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handlePing() {
    setLoading(true);
    setMessage(null);
    try {
      const result = await ping();
      setMessage(result);
    } catch (err) {
      setMessage(`调用失败: ${String(err)}`);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>个人提示词管理平台</h1>
        <p>M0 骨架 · 打通 React → Tauri IPC → Rust 最小链路</p>
      </header>
      <main className="app-main">
        <button onClick={handlePing} disabled={loading}>
          {loading ? "调用中…" : "ping 后端"}
        </button>
        {message && <p className="ping-result">后端返回：{message}</p>}
      </main>
    </div>
  );
}