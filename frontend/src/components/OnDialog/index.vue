<template>
  <el-dialog
    :model-value="modelValue"
    :width="maximized ? '100%' : width"
    :close-on-click-modal="closeOnClickModal"
    :show-close="false"
    :destroy-on-close="destroyOnClose"
    :top="maximized ? '0' : '15vh'"
    :class="['on-dialog', { 'on-dialog--maximized': maximized }]"
    :style="{ '--el-dialog-padding-primary': '0' }"
    @close="handleClose"
  >
    <!-- 自定义标题栏 -->
    <template #header>
      <div
        class="on-dialog__header"
        :style="headerStyle"
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
import { ref, watch, nextTick, computed } from 'vue'
import { FullScreen, CopyDocument, Close } from '@element-plus/icons-vue'
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()

/** 根据主题色生成标题栏渐变样式 */
const headerStyle = computed(() => {
  const hsl = hexToHsl(settingsStore.themeColor)
  // 起点：提亮 + 稍降饱和度（光泽感）
  const start = `hsl(${hsl.h}, ${Math.min(hsl.s + 5, 100)}%, ${Math.min(hsl.l + 8, 55)}%)`
  // 中间：原始主题色
  const mid = `hsl(${hsl.h}, ${hsl.s}%, ${hsl.l}%)`
  // 终点：压暗 + 稍增饱和度（深邃感）
  const end = `hsl(${(hsl.h + 8) % 360}, ${Math.min(hsl.s + 10, 100)}%, ${Math.max(hsl.l - 18, 12)}%)`
  return {
    background: `linear-gradient(135deg, ${start} 0%, ${mid} 50%, ${end} 100%)`,
  }
})

function hexToHsl(hex: string): { h: number; s: number; l: number } {
  hex = hex.replace('#', '')
  const r = parseInt(hex.substring(0, 2), 16) / 255
  const g = parseInt(hex.substring(2, 4), 16) / 255
  const b = parseInt(hex.substring(4, 6), 16) / 255
  const max = Math.max(r, g, b), min = Math.min(r, g, b)
  let h = 0, s = 0
  const l = (max + min) / 2
  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break
      case g: h = ((b - r) / d + 2) / 6; break
      case b: h = ((r - g) / d + 4) / 6; break
    }
  }
  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) }
}

const props = withDefaults(defineProps<{
  modelValue: boolean
  title?: string
  width?: string
  maximizable?: boolean
  closeOnClickModal?: boolean
  destroyOnClose?: boolean
}>(), {
  title: '',
  width: '600px',
  maximizable: true,
  closeOnClickModal: true,
  destroyOnClose: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'close': []
}>()

// ========== 最大化 ==========
const maximized = ref(false)
const savedStyles = ref('')

function toggleMaximize() {
  const el = dialogRef.value
  if (!el) return

  if (!maximized.value) {
    // 保存当前 inline 样式（拖拽产生的 position/left/top 等）
    savedStyles.value = el.style.cssText
    // 一次性设置全屏样式，inline !important 胜过一切 CSS 选择器
    el.style.cssText = `
      position: fixed !important;
      inset: 0 !important;
      width: 100vw !important;
      height: 100vh !important;
      max-height: 100vh !important;
      margin: 0 !important;
      padding: 0 !important;
      border-radius: 0 !important;
      transform: none !important;
      display: flex !important;
      flex-direction: column !important;
    `
    maximized.value = true
  } else {
    // 恢复拖拽前的 inline 样式
    el.style.cssText = savedStyles.value
    maximized.value = false
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
  overflow: hidden;
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.25);
}

/* 清除 header 外层 padding/margin，标题栏背景铺满 */
.on-dialog :deep(.el-dialog__header) {
  padding: 0 !important;
  margin: 0 !important;
}

.on-dialog :deep(.el-dialog__headerbtn) {
  display: none;
}

/* 内容区左右下 padding，标题栏通过 margin-bottom 保持间距 */
.on-dialog :deep(.el-dialog__body) {
  padding: 16px !important;
  padding-top: 0 !important;
}

.on-dialog :deep(.el-dialog__footer) {
  padding: 16px !important;
  border-top: 1px solid var(--el-border-color-lighter);
}

/* 最大化时 body 撑满剩余空间 */
.on-dialog--maximized :deep(.el-dialog__body) {
  flex: 1;
  overflow: auto;
}

/* ========== 自定义标题栏 ========== */
.on-dialog__header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 44px;
  padding: 0 12px 0 20px;
  margin-bottom: 16px;
  color: #ffffff;
  cursor: move;
  user-select: none;
  border-top-left-radius: var(--el-dialog-border-radius);
  border-top-right-radius: var(--el-dialog-border-radius);
  overflow: hidden;
}

/* 顶部高光条 —— 玻璃质感 */
.on-dialog__header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.35) 30%, rgba(255,255,255,0.5) 50%, rgba(255,255,255,0.35) 70%, transparent 100%);
}

/* 底部亮线 —— 层次感 */
.on-dialog__header::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.15) 20%, rgba(255,255,255,0.25) 50%, rgba(255,255,255,0.15) 80%, transparent 100%);
}

.on-dialog__title {
  position: relative;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.on-dialog__actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  position: relative;
}

.on-dialog__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: all 0.2s;
  backdrop-filter: blur(4px);
}

.on-dialog__btn:hover {
  background: rgba(255, 255, 255, 0.22);
  color: #ffffff;
  transform: scale(1.05);
}

.on-dialog__btn:active {
  transform: scale(0.95);
}

.on-dialog__btn--close:hover {
  background: rgba(231, 76, 60, 0.85);
  color: #ffffff;
}
</style>

<!-- 全局覆盖：el-dialog 内部元素不受 scoped 穿透限制 -->
<style>
.on-dialog.el-dialog {
  --el-dialog-border-radius: 16px;
}

.on-dialog .el-dialog__header {
  padding: 0 !important;
  margin: 0 !important;
}

.on-dialog .el-dialog__headerbtn {
  display: none;
}

.on-dialog--maximized .on-dialog__header {
  border-radius: 0;
}

.on-dialog .el-dialog__body {
  padding: 0 16px 16px 16px !important;
}

.on-dialog .el-dialog__footer {
  padding: 16px !important;
}
</style>
