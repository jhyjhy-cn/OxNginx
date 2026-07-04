<template>
  <div v-if="visible && row" class="context-menu" :style="{ left: x + 'px', top: y + 'px' }" @click="$emit('close')">
    <!-- 文件夹菜单 -->
    <template v-if="row.is_dir">
      <div class="ctx-item" @click="$emit('open', row)"><el-icon><FolderOpened /></el-icon>打开</div>
      <div class="ctx-item" @click="$emit('chmod', row)"><el-icon><Lock /></el-icon>权限</div>
      <div class="ctx-item" @click="$emit('copy', row)"><el-icon><CopyDocument /></el-icon>复制</div>
      <div class="ctx-item" @click="$emit('move', row)"><el-icon><Rank /></el-icon>剪切</div>
      <div class="ctx-item" @click="$emit('rename', row)"><el-icon><EditPen /></el-icon>重命名</div>
      <div class="ctx-item danger" @click="$emit('delete', row)"><el-icon><Delete /></el-icon>删除</div>
      <div class="ctx-divider"></div>
      <div class="ctx-item" @click="$emit('compress', row)"><el-icon><FolderAdd /></el-icon>创建压缩</div>
      <div class="ctx-item" @click="$emit('properties', row)"><el-icon><InfoFilled /></el-icon>属性</div>
    </template>
    <!-- 文件菜单 -->
    <template v-else>
      <div class="ctx-item" @click="$emit('edit', row)"><el-icon><Edit /></el-icon>编辑</div>
      <div class="ctx-item" @click="$emit('download', row)"><el-icon><Download /></el-icon>下载</div>
      <div class="ctx-item" @click="$emit('chmod', row)"><el-icon><Lock /></el-icon>权限</div>
      <div class="ctx-item" @click="$emit('copy', row)"><el-icon><CopyDocument /></el-icon>复制</div>
      <div class="ctx-item" @click="$emit('move', row)"><el-icon><Rank /></el-icon>剪切</div>
      <div class="ctx-item" @click="$emit('rename', row)"><el-icon><EditPen /></el-icon>重命名</div>
      <div class="ctx-item danger" @click="$emit('delete', row)"><el-icon><Delete /></el-icon>删除</div>
      <div class="ctx-divider"></div>
      <div class="ctx-item" @click="$emit('compress', row)"><el-icon><FolderAdd /></el-icon>创建压缩</div>
      <div v-if="isArchive(row.name)" class="ctx-item" @click="$emit('extract', row)"><el-icon><FolderRemove /></el-icon>解压</div>
      <div class="ctx-item" @click="$emit('properties', row)"><el-icon><InfoFilled /></el-icon>属性</div>
    </template>
  </div>
</template>

<script setup lang="ts">
import type { FileItem } from './useFileManager'

defineProps<{
  visible: boolean
  x: number
  y: number
  row: FileItem | null
  isArchive: (name?: string) => boolean
}>()

defineEmits<{
  close: []
  open: [row: FileItem]
  edit: [row: FileItem]
  download: [row: FileItem]
  chmod: [row: FileItem]
  copy: [row: FileItem]
  move: [row: FileItem]
  rename: [row: FileItem]
  delete: [row: FileItem]
  compress: [row: FileItem]
  extract: [row: FileItem]
  properties: [row: FileItem]
}>()
</script>

<style scoped>
.context-menu {
  position: fixed; z-index: 9999;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px; padding: 4px 0; min-width: 160px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}
.ctx-item { display: flex; align-items: center; gap: 8px; padding: 8px 16px; font-size: 13px; cursor: pointer; color: var(--el-text-color-regular); transition: background 0.15s; }
.ctx-item:hover { background: var(--el-fill-color-light); }
.ctx-item.danger { color: var(--el-color-danger); }
.ctx-divider { height: 1px; background: var(--el-border-color-lighter); margin: 4px 0; }
</style>
