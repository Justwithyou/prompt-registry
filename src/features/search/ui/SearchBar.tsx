import { useEffect, useState } from "react";
import { usePromptStore } from "../../prompt-library/index";
import { Icon } from "../../../shared/ui/Icon";

/**
 * 搜索框（features/search）。
 *
 * 输入关键词（250ms 防抖）后调用真实 promptStore：
 *  - 非空关键词走 `searchPrompts`（后端 FTS 全文搜索）
 *  - 空关键词走 `fetchPrompts`（恢复列表，保留分类/置顶筛选）
 * 搜索框与列表经 store 联动，不再直接互相 import。
 */
export function SearchBar() {
  const searchPrompts = usePromptStore((s) => s.searchPrompts);
  const fetchPrompts = usePromptStore((s) => s.fetchPrompts);
  const [value, setValue] = useState("");

  useEffect(() => {
    const timer = setTimeout(() => {
      const keyword = value.trim();
      if (keyword) void searchPrompts(keyword);
      else void fetchPrompts();
    }, 250);
    return () => clearTimeout(timer);
  }, [value, searchPrompts, fetchPrompts]);

  return (
    <div className="search-bar">
      <Icon name="search" size={16} className="search-bar-icon" />
      <input
        type="text"
        value={value}
        placeholder="搜索标题 / 内容 / 描述…"
        onChange={(e) => setValue(e.target.value)}
      />
    </div>
  );
}