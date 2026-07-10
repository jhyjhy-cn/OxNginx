<template>
  <el-upload
    :action="url"
    :headers="headers"
    :file-list="fileList"
    :limit="limit"
    :accept="accept"
    :drag="drag"
    :show-file-list="showFileList"
    :before-upload="beforeUpload"
    :on-success="onSuccess"
    :on-error="onError"
    :on-remove="onRemove"
    :on-exceed="onExceed"
    name="file"
  >
    <template v-if="drag">
      <el-icon class="on-upload-drag-icon"><UploadFilled /></el-icon>
      <div class="on-upload-drag-text">
        {{ $t('common.uploadDragText') }}
        <em class="on-upload-drag-link">{{ $t('common.uploadClickText') }}</em>
      </div>
    </template>
    <slot v-else>
      <el-button type="primary" :loading="loading">{{ $t('common.upload') }}</el-button>
    </slot>
  </el-upload>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { UploadFilled } from '@element-plus/icons-vue'
import { useAuthStore } from '@/stores/auth'
import { useMessage } from '@/hooks'

interface UploadResult {
  url: string
  name?: string
}

const props = withDefaults(
  defineProps<{
    /** 上传地址，由 axios 实例代理 */
    url?: string
    /** v-model：单文件绑定 URL，多文件（limit>1）绑定 URL 数组 */
    modelValue?: string | string[]
    /** 接收的 MIME / 扩展名 */
    accept?: string
    /** 最大文件数，>1 时启用多文件模式（v-model 接数组） */
    limit?: number
    /** 拖拽上传样式 */
    drag?: boolean
    /** 是否显示文件列表 */
    showFileList?: boolean
  }>(),
  {
    url: '/api/rbac/files/upload',
    modelValue: '',
    accept: '*/*',
    limit: 1,
    drag: false,
    showFileList: true,
  }
)

const emit = defineEmits<{
  (e: 'update:modelValue', v: string | string[]): void
  (e: 'change', payload: { url: string; name?: string; response: UploadResult }): void
}>()

const { success, error, warning } = useMessage()
const authStore = useAuthStore()

// ponytail: el-upload 走原生 XHR，不走 axios interceptor，token 必须显式注入
const headers = computed(() => ({
  Authorization: `Bearer ${authStore.token || ''}`,
}))

const loading = ref(false)
const fileList = ref<{ name: string; url: string; status: string }[]>([])

const isMulti = computed(() => props.limit > 1)

const syncFileList = (val: string | string[] | undefined) => {
  if (!val || (Array.isArray(val) && !val.length)) {
    fileList.value = []
    return
  }
  const urls = Array.isArray(val) ? val : [val]
  fileList.value = urls.map((u) => ({ name: u.split('/').pop() ?? u, url: u, status: 'success' }))
}

watch(
  () => props.modelValue,
  (v) => syncFileList(v),
  { immediate: true, deep: true }
)

const beforeUpload = (_file: File) => {
  loading.value = true
  return true
}

const onSuccess = (res: { code: number; message?: string; data: UploadResult }, file: { name: string }) => {
  loading.value = false
  if (res?.code !== 0 || !res.data?.url) {
    error(res?.message ? String(res.message) : 'common.uploadFailed')
    fileList.value = fileList.value.filter((f) => f.name !== file.name)
    return
  }
  success('common.uploadSuccess')
  emit('change', { url: res.data.url, name: res.data.name, response: res.data })
  if (isMulti.value) {
    const current = Array.isArray(props.modelValue) ? [...props.modelValue] : []
    current.push(res.data.url)
    emit('update:modelValue', current)
  } else {
    emit('update:modelValue', res.data.url)
  }
}

const onError = () => {
  loading.value = false
  error('common.uploadFailed')
}

const onRemove = (file: { url?: string }) => {
  if (isMulti.value) {
    const current = Array.isArray(props.modelValue) ? [...props.modelValue] : []
    if (file.url) emit('update:modelValue', current.filter((u) => u !== file.url))
  } else {
    emit('update:modelValue', '')
  }
}

const onExceed = () => {
  warning('common.uploadLimitExceeded')
}

defineExpose({ loading })
</script>

<style scoped>
.on-upload-drag-icon {
  font-size: 40px;
  color: var(--el-color-primary);
  margin-bottom: 12px;
}
.on-upload-drag-text {
  font-size: 14px;
  color: var(--el-text-color-regular);
}
.on-upload-drag-link {
  color: var(--el-color-primary);
  font-style: normal;
  cursor: pointer;
}
:deep(.el-upload-dragger) {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}
</style>
