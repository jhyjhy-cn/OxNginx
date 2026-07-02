// ponytail: monaco worker 注册。main.ts 顶层副作用导入一次，保证 ConfigEditor.vue 懒加载前环境就绪。
// ponytail: 0.45 走主线程 fallback 是历史遗留，0.52+ 没显式 worker 会丢高亮/智能提示性能。
import * as monaco from 'monaco-editor'
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'

;(self as any).MonacoEnvironment = {
  getWorker(_workerId: string, label: string) {
    // ponytail: 仅 editor worker 足够 —— 项目只用 nginx 文本，没语言服务需要
    if (label === 'json') return new editorWorker()
    return new editorWorker()
  },
}

// ponytail: 命名空间再导出，ConfigEditor.vue 用 `monaco.editor.create(...)` 形式调用
export { monaco }
