<template>
  <OnDialog
    v-model="visible"
    :title="titleText"
    width="80%"
    height="75%"
    maximizable
    :modal="false"
  >
    <FileTabPanel
      v-if="open"
      tab-id=""
      :initial-path="effectivePath"
      :picker="{ mode, accept: props.acceptExtensions, multiple }"
      class="on-fp-panel"
      @pick="onPick"
      @picker-path="onPickerPath"
    >
      <template #picker-footer>
        <span class="on-fp-hint">
          <template v-if="mode === 'file'">
            {{ picked.length ? t('dbm.picker.pickedN', { n: picked.length }) : t('dbm.picker.pickFileHint') }}
          </template>
          <template v-else-if="currentFolderPath">{{ t('dbm.picker.curDir', { path: currentFolderPath }) }}</template>
          <template v-else>{{ t('dbm.picker.pickFolderHint') }}</template>
        </span>
        <el-button @click="visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :disabled="!canConfirm" @click="confirm">
          {{ t('common.select') }}
        </el-button>
      </template>
    </FileTabPanel>
  </OnDialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import OnDialog from '@/components/OnDialog/index.vue'
import FileTabPanel from '@/views/files/FileTabPanel.vue'

type Mode = 'folder' | 'file'

const props = withDefaults(
  defineProps<{
    modelValue: boolean
    mode?: Mode
    title?: string
    initialPath?: string
    /** 多选仅 file 模式生效 */
    multiple?: boolean
    /** file 模式:限制可选文件后缀,如 ['db','sqlite','sqlite3']。目录不受影响。 */
    acceptExtensions?: string[]
  }>(),
  {
    mode: 'folder',
    title: '',
    initialPath: '',
    multiple: false,
    acceptExtensions: () => [],
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  /**
   * 选中回调。
   * folder 模式:单字符串数组(选中目录的 path,长度总是 1)
   * file 模式:根据 multiple,单字符串数组或多字符串数组
   */
  (e: 'pick', paths: string[]): void
}>()

const { t } = useI18n()

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit('update:modelValue', v),
})

const open = computed(() => props.modelValue)
const picked = ref<string[]>([])
const currentFolderPath = ref('')

const effectivePath = computed(() => props.initialPath || currentFolderPath.value || '')

const titleText = computed(() => {
  if (props.title) return props.title
  return props.mode === 'folder' ? t('common.chooseFolder') : t('common.chooseFile')
})

const canConfirm = computed(() => {
  if (props.mode === 'folder') return picked.value.length === 1
  // file 模式:多选要至少 1,单选要恰好 1
  return picked.value.length >= 1
})

function onPick(paths: string[]) {
  // ponytail: 不自动 confirm —— 只更新 picked,父 footer 提示 + 确定按钮启用,用户手动点。
  picked.value = [...paths]
}

function confirm() {
  if (!canConfirm.value) return
  emit('pick', [...picked.value])
  visible.value = false
}

function onPickerPath(path: string) {
  if (path) currentFolderPath.value = path
}

watch(visible, (v) => {
  if (v) picked.value = []
})
</script>

<style scoped>
/*
 * ponytail: 这层是 picker 在 dialog body 内的 root。
 * dialog 用了 :height 模式 → body 是 display:flex; flex-direction:column;
 * 因此 root 也必须 flex column + flex:1 + min-height:0, 否则撑爆 body。
 */
.on-fp-panel {
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}
.on-fp-hint {
  margin-right: auto;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 50%;
}
:deep(.file-tab-panel) {
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}
:deep(.fm-content) {
  flex: 1 1 0;
  min-height: 0;
  overflow: auto;
}
:deep(.fm-content .el-table) {
  max-height: 100%;
}
:deep(.fm-content .el-table__inner-wrapper) {
  max-height: 100%;
}
</style>

<!--
  全局生效: FileTabPanel 内的 path-dropdown 通过 Teleport 渲染到 <body> 顶层,
  不在 OnFilePicker scoped style 范围内,scoped :deep 选不到。
  拉到 99999 以上,确保 picker 弹层之上。
-->
<style>
body .path-dropdown-mask {
  z-index: 99999 !important;
}
body .path-dropdown {
  z-index: 100000 !important;
}
</style>
