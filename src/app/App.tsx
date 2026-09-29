import { RouterProvider } from "react-router";
import { router } from "./router";
import "../styles/app.css";

/**
 * 应用根组件（FSD app 层）。
 * M1 起由路由接管，渲染 RouterProvider；M0 的连通性占位已移除。
 */
export default function App() {
  return <RouterProvider router={router} />;
}