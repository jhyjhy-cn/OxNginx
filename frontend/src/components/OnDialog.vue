<template>
  <el-dialog
    :model-value="modelValue"
    :width="maximized ? '100%' : width"
    :close-on-click-modal="closeOnClickModal"
    :show-close="false"
    :top="maximized ? '0' : '15vh'"
    :class="['on-dialog', { 'on-dialog--maximized': maximized }]"
    @close="handleClose"
  >
    <!-- 自定义标题栏 -->
    <template #header>
      <div
        class="on-dialog__header"
        @mousedown="onDragStart"
        @dblclick="toggleMaximize"
      >
        <span class="on-dialog__title">{{ title }}</span>
        <div class="on-dialog__actions">
          <button
            v-if="maximizable"
            class="on-dialog__btn"
            :title="maximized ? '还原' : '最大化'"
            @click.stop="toggleMaximize"
          >
            <el-icon :size="14">
              <FullScreen v-if="!maximized" />
              <CopyDocument v-else />
            </el-icon>
          </button>
          <button class="on-dialog__btn on-dialog__btn--close" title="关闭" @click.stop="handleClose">
            <el-icon :size="14"><Close /></el-icon>
          </button>
        </div>
      </div>
    </template>

    <!-- 内容 -->
    <slot />

    <!-- 底部按钮 -->
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { FullScreen, CopyDocument, Close } from '@element-plus/icons-vue'

const props = withDefaults(defineProps<{
  modelValue: boolean
  title?: string
  width?: string
  maximizable?: boolean
  closeOnClickModal?: boolean
}>(), {
  title: '',
  width: '600px',
  maximizable: true,
  closeOnClickModal: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'close': []
}>()

// ========== 最大化 ==========
const maximized = ref(false)
const savedPos = ref({ left: '', top: '' })

function toggleMaximize() {
  if (!maximized.value) {
    const el = dialogRef.value
    if (el) {
      savedPos.value = { left: el.style.left || '', top: el.style.top || '' }
    }
    maximized.value = true
  } else {
    maximized.value = false
    nextTick(() => {
      const el = dialogRef.value
      if (el && savedPos.value.left) {
        el.style.left = savedPos.value.left
        el.style.top = savedPos.value.top
      }
    })
  }
}

// ========== 拖拽 ==========
const dragging = ref(false)
const dragOffset = ref({ x: 0, y: 0 })
const dialogRef = ref<HTMLElement | null>(null)

function onDragStart(e: MouseEvent) {
  if (maximized.value) return
  const header = e.currentTarget as HTMLElement
  const dialog = header.closest('.el-dialog') as HTMLElement
  if (!dialog) return
  dialogRef.value = dialog

  // 读取当前视觉位置（此时 transform 还在）
  const rect = dialog.getBoundingClientRect()

  // 先切到 fixed + 清 transform，再设 left/top → 无跳动
  dialog.style.position = 'fixed'
  dialog.style.transform = 'none'
  dialog.style.left = rect.left + 'px'
  dialog.style.top = rect.top + 'px'
  dialog.style.margin = '0'

  dragging.value = true
  dragOffset.value = {
    x: e.clientX - rect.left,
    y: e.clientY - rect.top,
  }

  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
  e.preventDefault()
}

function onDragMove(e: MouseEvent) {
  if (!dragging.value || !dialogRef.value) return
  const dialog = dialogRef.value
  const maxX = window.innerWidth - dialog.offsetWidth
  const maxY = window.innerHeight - dialog.offsetHeight

  let x = e.clientX - dragOffset.value.x
  let y = e.clientY - dragOffset.value.y
  x = Math.max(0, Math.min(x, maxX))
  y = Math.max(0, Math.min(y, maxY))

  dialog.style.left = x + 'px'
  dialog.style.top = y + 'px'
}

function onDragEnd() {
  dragging.value = false
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
}

// ========== 关闭 ==========
function handleClose() {
  emit('update:modelValue', false)
  emit('close')
  // 重置定位样式，下次打开时 el-dialog 重新居中
  nextTick(() => {
    const el = dialogRef.value
    if (el) {
      el.style.position = ''
      el.style.transform = ''
      el.style.left = ''
      el.style.top = ''
      el.style.margin = ''
    }
    maximized.value = false
  })
}

// 打开时重置
watch(() => props.modelValue, (val) => {
  if (val) {
    maximized.value = false
    nextTick(() => {
      const el = dialogRef.value
      if (el) {
        el.style.position = ''
        el.style.transform = ''
        el.style.left = ''
        el.style.top = ''
        el.style.margin = ''
      }
    })
  }
})
</script>

<style scoped>
/* 覆盖 el-dialog 默认样式 */
.on-dialog :deep(.el-dialog) {
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.25);
}

/* 清除 header 外层 padding，让标题栏背景填满 */
.on-dialog :deep(.el-dialog__header) {
  padding: 0 !important;
  margin: 0 !important;
}

.on-dialog :deep(.el-dialog__headerbtn) {
  display: none;
}

.on-dialog :deep(.el-dialog__body) {
  padding: 20px;
}

.on-dialog :deep(.el-dialog__footer) {
  padding: 12px 20px;
  border-top: 1px solid var(--el-border-color-lighter);
}

/* 最大化 */
.on-dialog--maximized :deep(.el-dialog) {
  border-radius: 0;
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-height: 100vh;
}

.on-dialog--maximized :deep(.el-dialog__body) {
  flex: 1;
  overflow: auto;
}

/* ========== 自定义标题栏 ========== */
.on-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 12px 0 20px;
  background: linear-gradient(135deg, #001529 0%, #002140 100%);
  color: #ffffff;
  cursor: move;
  user-select: none;
}

.on-dialog__title {
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.on-dialog__actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.on-dialog__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.75);
  cursor: pointer;
  transition: all 0.2s;
}

.on-dialog__btn:hover {
  background: rgba(255, 255, 255, 0.18);
  color: #ffffff;
}

.on-dialog__btn--close:hover {
  background: #e74c3c;
  color: #ffffff;
}
</style>
