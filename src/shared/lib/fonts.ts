/**
 * 全局字体配置（shared/lib）。
 * 提供 5 种常用字体 + 本地字体导入（经 IndexedDB 持久化，FontFace 注册）。
 */

export const IMPORTED_FONT_FAMILY = "PromptHubLocalFont";
export const IMPORTED_FONT_KEY = "local";

export interface FontOption {
  key: string;
  label: string;
  family: string;
}

export const FONT_OPTIONS: FontOption[] = [
  {
    key: "default",
    label: "系统默认",
    family: `"PingFang SC", "Microsoft YaHei", "Segoe UI", system-ui, -apple-system, sans-serif`,
  },
  { key: "yahei", label: "微软雅黑", family: `"Microsoft YaHei", "微软雅黑", sans-serif` },
  { key: "simhei", label: "黑体", family: `"SimHei", "黑体", sans-serif` },
  { key: "simsun", label: "宋体", family: `"SimSun", "宋体", serif` },
  { key: "kaiti", label: "楷体", family: `"KaiTi", "楷体", serif` },
  { key: "fangsong", label: "仿宋", family: `"FangSong", "仿宋", serif` },
];

export const DEFAULT_FONT_KEY = "default";

/** 根据字体 key 解析出实际的 CSS font-family 值。 */
export function resolveFontFamily(key: string): string {
  if (key === IMPORTED_FONT_KEY) {
    return `"${IMPORTED_FONT_FAMILY}", "Microsoft YaHei", sans-serif`;
  }
  return FONT_OPTIONS.find((f) => f.key === key)?.family ?? FONT_OPTIONS[0].family;
}

/** 根据字体 key 解析出展示名。 */
export function fontLabel(key: string): string {
  if (key === IMPORTED_FONT_KEY) {
    return localStorage.getItem("prompt-manager.imported-font-name") || "本地字体";
  }
  return FONT_OPTIONS.find((f) => f.key === key)?.label ?? FONT_OPTIONS[0].label;
}

const DB_NAME = "prompt-manager-fonts";
const STORE = "fonts";
const FONT_KEY = "font";

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE)) db.createObjectStore(STORE);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function idbGet(key: string): Promise<ArrayBuffer | undefined> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readonly");
    const req = tx.objectStore(STORE).get(key);
    req.onsuccess = () => resolve(req.result as ArrayBuffer | undefined);
    req.onerror = () => reject(req.error);
  });
}

async function idbSet(key: string, value: ArrayBuffer): Promise<void> {
  const db = await openDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, "readwrite");
    tx.objectStore(STORE).put(value, key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

/** 注册某个字体面的资源到文档（供 FontFace 使用）。 */
async function registerFace(source: ArrayBuffer): Promise<void> {
  const face = new FontFace(IMPORTED_FONT_FAMILY, source);
  await face.load();
  document.fonts.add(face);
}

/** 导入本地字体文件：注册到当前会话并持久化到 IndexedDB。 */
export async function saveImportedFont(file: File): Promise<void> {
  const buf = await file.arrayBuffer();
  await registerFace(buf);
  await idbSet(FONT_KEY, buf);
}

/** 应用启动时加载已持久化的本地字体。返回是否成功注册。 */
export async function loadImportedFont(): Promise<boolean> {
  try {
    const buf = await idbGet(FONT_KEY);
    if (!buf) return false;
    await registerFace(buf);
    return true;
  } catch {
    return false;
  }
}