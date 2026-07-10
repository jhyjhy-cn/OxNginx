import { ElMessage } from "element-plus";

/**
 * 触发浏览器下载 Blob
 * ponytail: 不依赖第三方库，原生 Blob + a[download] 搞定；mime 留给调用方指定
 */
export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * 通用下载：从后端 GET 一个流式响应，保存为文件
 * 默认期望 xlsx（application/vnd.openxmlformats-...），失败时尝试读 JSON 错误体
 */
export async function downloadXlsx(
  url: string,
  params: Record<string, any> = {},
  filename?: string,
) {
  const { default: api } = await import("@/api");
  try {
    const res = await api.get(url, {
      params,
      responseType: "blob",
    });
    const blob = new Blob([res.data], {
      type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    });
    const ts = new Date().toISOString().slice(0, 19).replace(/[T:-]/g, "");
    const name = filename || `export-${ts}.xlsx`;
    downloadBlob(blob, name);
    ElMessage.success("导出成功");
    return true;
  } catch (e: any) {
    // 后端返回的 JSON 错误体被 axios 当 blob 接住，要解析回来
    if (e?.response?.data instanceof Blob) {
      const text = await e.response.data.text();
      try {
        const json = JSON.parse(text);
        ElMessage.error(json?.message || "导出失败");
      } catch {
        ElMessage.error("导出失败");
      }
    } else {
      ElMessage.error(e?.message || "导出失败");
    }
    return false;
  }
}
