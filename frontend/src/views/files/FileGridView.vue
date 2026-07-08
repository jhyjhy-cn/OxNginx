<template>
  <div class="card-grid">
    <div
      v-for="row in files"
      :key="row.path"
      class="file-card"
      @dblclick="$emit('dblclick', row)"
      @contextmenu.prevent="$emit('contextmenu', row, $event)"
    >
      <div class="card-icon"><OnIcon :svgName="getFileIcon(row)" :size="40" /></div>
      <div class="card-name" :title="row.name">{{ row.name }}</div>
      <div class="card-meta">
        <span v-if="!row.is_dir">{{ formatSize(row.size) }}</span>
        <span v-else-if="row._size !== undefined">{{ formatSize(row._size) }}</span>
        <span v-else class="calc-link" @click.stop="$emit('calc-size', row)">计算</span>
      </div>
    </div>
    <div v-if="files.length === 0" class="empty-tip">{{ emptyText }}</div>
  </div>
</template>

<script setup lang="ts">
import OnIcon from '@/components/OnIcon/index.vue'
import type { FileItem } from './useFileManager'

defineProps<{
  files: FileItem[]
  getFileIcon: (row: FileItem) => string
  formatSize: (bytes: number) => string
  emptyText: string
}>()

defineEmits<{
  dblclick: [row: FileItem]
  contextmenu: [row: FileItem, event: MouseEvent]
  'calc-size': [row: FileItem]
}>()
</script>

<style scoped>
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 12px;
  padding: 16px;
}
.file-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
  border: 1px solid transparent;
}
.file-card:hover {
  background: var(--el-fill-color-light);
  border-color: var(--el-border-color);
}
.card-icon {
  margin-bottom: 8px;
}
.card-name {
  font-size: 12px;
  text-align: center;
  word-break: break-all;
  line-height: 1.4;
  max-height: 2.8em;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}
.card-meta {
  font-size: 11px;
  color: var(--el-text-color-placeholder);
  margin-top: 4px;
}
.calc-link {
  color: var(--el-color-primary);
  cursor: pointer;
  font-size: 11px;
}
.calc-link:hover {
  text-decoration: underline;
}
.empty-tip {
  text-align: center;
  color: var(--el-text-color-placeholder);
  padding: 60px 0;
  grid-column: 1 / -1;
}
</style>
