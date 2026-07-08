<template>
  <div class="config-tab-content">
    <div class="config-hint">
      <p>提示：直接编写 nginx 伪静态规则，支持 rewrite、if、location 等指令</p>
      <p>此处规则将插入到 server 块内，若您不了解规则语法，请勿随意修改</p>
    </div>
    <div ref="editorRef" class="config-editor-box" />
    <div style="display: flex; gap: 8px; margin-top: 8px">
      <el-button type="primary" size="small" :loading="saving" @click="save">{{ $t('common.save') }}</el-button>
      <el-button size="small" @click="loadContent">{{ $t('common.refresh') }}</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import api from '@/api'
import { monaco } from '@/utils/monaco-env'

const { t } = useI18n()

const props = defineProps<{
  siteId: number
  rewriteRules: string
}>()

const emit = defineEmits<{
  saved: []
  'update:rewriteRules': [value: string]
}>()

const editorRef = ref<HTMLElement>()
const saving = ref(false)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

onMounted(() => {
  if (!editorRef.value) return
  editor = monaco.editor.create(editorRef.value, {
    value: props.rewriteRules || '',
    language: 'nginx',
    theme: 'vs-dark',
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 4,
  })
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    save()
  })
})

onUnmounted(() => {
  editor?.dispose()
  editor = null
})

// 外部刷新内容时同步
watch(
  () => props.rewriteRules,
  (val) => {
    if (editor && editor.getValue() !== val) {
      editor.setValue(val || '')
    }
  }
)

function loadContent() {
  if (editor) {
    editor.setValue(props.rewriteRules || '')
  }
}

async function save() {
  if (!editor) return
  saving.value = true
  try {
    const content = editor.getValue()
    await api.put(`/api/sites/${props.siteId}`, {
      rewrite_rules: content || null,
    })
    emit('update:rewriteRules', content)
    ElMessage.success(t('common.success'))
    emit('saved')
  } catch (e: any) {
    ElMessage.error(e.response?.data?.message || t('sites.operationFailed'))
  } finally {
    saving.value = false
  }
}

defineExpose({ loadContent })
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
