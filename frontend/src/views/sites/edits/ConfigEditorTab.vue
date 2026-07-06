<template>
  <div class="config-tab-content">
    <div class="config-hint">
      <p>提示：Ctrl+F 搜索关键字，Ctrl+S 保存，Ctrl+H 查找替换</p>
      <p>此处为站点主配置文件，若您不了解配置规则，请勿随意修改</p>
    </div>
    <div ref="editorRef" class="config-editor-box" />
    <div style="display: flex; gap: 8px; margin-top: 8px">
      <el-button
        type="primary"
        size="small"
        :loading="saving"
        @click="save"
        >{{ $t("common.save") }}</el-button
      >
      <el-button size="small" @click="loadContent">{{
        $t("common.refresh")
      }}</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage, ElMessageBox } from "element-plus";
import api from "@/api";
import { monaco } from "@/utils/monaco-env";

const { t } = useI18n();

const props = defineProps<{
  siteId: number;
  siteName: string;
}>();

const emit = defineEmits<{
  saved: [];
}>();

const editorRef = ref<HTMLElement>();
const saving = ref(false);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let errorDecorations: string[] = [];

onMounted(() => {
  if (!editorRef.value) return;
  editor = monaco.editor.create(editorRef.value, {
    value: "",
    language: "nginx",
    theme: "vs-dark",
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: "on",
    glyphMargin: true,
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 4,
  });
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    save();
  });
  loadContent();
});

onUnmounted(() => {
  editor?.dispose();
  editor = null;
});

async function loadContent() {
  if (!editor) return;
  try {
    const res = await api.get(`/api/config/file/${props.siteName}`);
    if (res.data.code === 0) {
      editor.setValue(res.data.data?.content || "");
    }
  } catch {
    /* ignore */
  }
}

async function save() {
  if (!editor) return;
  saving.value = true;
  try {
    const res = await api.put(`/api/config/file/${props.siteName}`, {
      content: editor.getValue(),
    });
    if (res.data.code === 0) {
      ElMessage.success(t("common.success"));
      errorDecorations = editor.deltaDecorations(errorDecorations, []);
      emit("saved");
      // 保存成功后重载 nginx
      try {
        await api.post("/api/nginx/reload");
      } catch {
        /* 静默 */
      }
    } else {
      showConfigError(res.data.message);
    }
  } catch (error: any) {
    showConfigError(error.response?.data?.message || t("common.failed"));
  } finally {
    saving.value = false;
  }
}

function showConfigError(msg: string) {
  if (!editor) return;
  errorDecorations = editor.deltaDecorations(errorDecorations, []);

  // 解析 nginx 错误中的行号
  const lineMatch = msg.match(/:(\d+)\r?\n/);
  if (lineMatch) {
    const line = Math.max(1, parseInt(lineMatch[1], 10) - 1);
    errorDecorations = editor.deltaDecorations(
      [],
      [
        {
          range: {
            startLineNumber: line,
            startColumn: 1,
            endLineNumber: line,
            endColumn: 1,
          },
          options: {
            isWholeLine: true,
            className: "config-error-line",
            glyphMarginClassName: "config-error-glyph",
            hoverMessage: { value: `**语法错误**: ${msg.split("\n")[0]}` },
          },
        },
      ],
    );
    editor.revealLineInCenter(line);
    editor.setPosition({ lineNumber: line, column: 1 });
    editor.focus();
  }

  ElMessageBox.alert(msg, "配置语法错误", {
    type: "error",
    confirmButtonText: "确定",
    customStyle: { maxWidth: "600px", wordBreak: "break-all" },
  });
}

defineExpose({ loadContent });
</script>

<style scoped>
.config-tab-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.config-editor-box {
  width: 100%;
  flex: 1;
  min-height: 300px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 4px;
}
.config-hint {
  margin-bottom: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.8;
}
</style>

<style>
.config-error-line {
  background: rgba(255, 0, 0, 0.15);
  border-left: 3px solid #e74c3c;
}
.config-error-glyph {
  background: #e74c3c;
  border-radius: 50%;
  width: 10px !important;
  height: 10px !important;
  margin-left: 4px;
}
</style>
