<template>
  <div ref="el" class="sql-editor" :style="{ height }"></div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { monaco } from '@/utils/monaco-env'

const props = withDefaults(
  defineProps<{
    modelValue: string
    height?: string
    readonly?: boolean
  }>(),
  { height: '180px', readonly: false },
)

const emit = defineEmits<{ 'update:modelValue': [v: string] }>()

const el = ref<HTMLElement | null>(null)
let editor: monaco.editor.IStandaloneCodeEditor | null = null

onMounted(() => {
  if (!el.value) return
  editor = monaco.editor.create(el.value, {
    value: props.modelValue,
    language: 'sql',
    theme: 'vs',
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: 'on',
    scrollBeyondLastLine: false,
    folding: true,
    tabSize: 2,
    readOnly: props.readonly,
    wordWrap: 'on',
  })
  editor.onDidChangeModelContent(() => {
    const v = editor!.getValue()
    if (v !== props.modelValue) emit('update:modelValue', v)
  })
})

onBeforeUnmount(() => {
  editor?.dispose()
  editor = null
})

watch(
  () => props.modelValue,
  (v) => {
    if (editor && v !== editor.getValue()) {
      editor.setValue(v)
    }
  }
)
</script>

<style scoped>
.sql-editor {
  width: 100%;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  overflow: hidden;
}
</style>
