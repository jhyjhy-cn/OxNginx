<template>
  <el-table
    :data="files"
    style="width: 100%"
    @row-contextmenu="(row: any, col: any, e: MouseEvent) => $emit('contextmenu', row, col, e)"
    @row-dblclick="(row: any) => $emit('dblclick', row)"
    @selection-change="(rows: any) => $emit('selection-change', rows)"
    highlight-current-row
    height="100%"
  >
    <el-table-column type="selection" width="45" />
    <el-table-column :label="t('files.name')" min-width="300">
      <template #default="{ row }">
        <div class="file-name-cell">
          <OnIcon :svgName="getFileIcon(row)" :size="18" class="file-icon" />
          <span class="file-name" :class="{ 'is-dir': row.is_dir }">{{ row.name }}</span>
          <el-tag v-if="row.note" size="small" type="info" class="note-tag">{{ row.note }}</el-tag>
        </div>
      </template>
    </el-table-column>
    <el-table-column :label="t('files.permissions') + ' / ' + t('files.owner')" width="200">
      <template #default="{ row }">
        <span v-if="row.permissions || row.owner">
          {{ row.permissions }}
          <template v-if="row.permissions && row.owner">/</template>
          {{ row.owner }}
        </span>
        <span v-else>-</span>
      </template>
    </el-table-column>
    <el-table-column :label="t('files.size')" width="120">
      <template #default="{ row }">
        <span v-if="!row.is_dir">{{ formatSize(row.size) }}</span>
        <span v-else-if="row._size !== undefined">{{ formatSize(row._size) }}</span>
        <el-button v-else link type="primary" size="small" :loading="row._calcLoading" @click="$emit('calc-size', row)">计算</el-button>
      </template>
    </el-table-column>
    <el-table-column prop="modified" :label="t('files.modified')" width="180" />
    <el-table-column :label="t('files.note')" min-width="180">
      <template #default="{ row }">
        <div class="note-cell" @mouseenter="$emit('note-enter', row)" @mouseleave="$emit('note-leave', row)">
          <template v-if="hoverNotePath === row.path">
            <el-input
              :model-value="editingNote"
              @update:model-value="$emit('note-update', $event)"
              size="small"
              :placeholder="t('files.notePlaceholder')"
              @keyup.enter="$emit('note-save', row)"
              @blur="$emit('note-save', row)"
              autofocus
            />
          </template>
          <template v-else>
            <span class="note-text" :class="{ empty: !row.note }">{{ row.note || '-' }}</span>
          </template>
        </div>
      </template>
    </el-table-column>
    <el-table-column :label="t('common.action')" width="150" fixed="right">
      <template #default="{ row }">
        <el-button link type="primary" size="small" @click="$emit('rename', row)">{{ t('files.rename') }}</el-button>
        <el-button link type="danger" size="small" @click="$emit('delete', row)">{{ t('files.delete') }}</el-button>
      </template>
    </el-table-column>
  </el-table>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import OnIcon from '@/components/OnIcon/index.vue'
import type { FileItem } from './useFileManager'

defineProps<{
  files: FileItem[]
  getFileIcon: (row: FileItem) => string
  formatSize: (bytes: number) => string
  hoverNotePath: string
  editingNote: string
}>()

defineEmits<{
  contextmenu: [row: FileItem, col: unknown, event: MouseEvent]
  dblclick: [row: FileItem]
  'selection-change': [rows: FileItem[]]
  'calc-size': [row: FileItem]
  'note-enter': [row: FileItem]
  'note-leave': [row: FileItem]
  'note-update': [value: string]
  'note-save': [row: FileItem]
  rename: [row: FileItem]
  delete: [row: FileItem]
}>()

const { t } = useI18n()
</script>

<style scoped>
.file-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}
.file-icon {
  flex-shrink: 0;
}
.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.file-name.is-dir {
  color: var(--el-color-primary);
  cursor: pointer;
}
.note-tag {
  flex-shrink: 0;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}
.note-cell {
  min-height: 24px;
  display: flex;
  align-items: center;
}
.note-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.note-text.empty {
  color: var(--el-text-color-placeholder);
}
</style>
