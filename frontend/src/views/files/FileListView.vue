<template>
  <el-table
    :data="files"
    style="width: 100%"
    @row-contextmenu="(row: any, col: any, e: MouseEvent) => $emit('contextmenu', row, col, e)"
    @row-click="onRowClick"
    @row-dblclick="(row: any) => $emit('dblclick', row)"
    @selection-change="(rows: any) => $emit('selection-change', rows)"
    highlight-current-row
    :style="tableStyle"
  >
    <!-- ponytail: picker 单选模式用 el-radio 自定义一列。
         v-model 绑 pickedPath,哪个 label === pickedPath 就被勾,原生单选语义。 -->
    <el-table-column v-if="radioMode && !noSelection" width="48" align="center">
      <template #default="{ row }">
        <el-radio
          v-model="pickedPath"
          :label="row.path"
          :disabled="disableDirs && row.is_dir"
          @change="onRadioChange(row)"
        >
          <span />
        </el-radio>
      </template>
    </el-table-column>
    <el-table-column v-else-if="!noSelection" type="selection" width="48" />
    <el-table-column :label="t('sys.files.name')" min-width="300">
      <template #default="{ row }">
        <div class="file-name-cell">
          <OnIcon :svgName="getFileIcon(row)" :size="18" class="file-icon" />
          <span class="file-name" :class="{ 'is-dir': row.is_dir }">{{ row.name }}</span>
          <el-tag v-if="row.note" size="small" type="info" class="note-tag">{{ row.note }}</el-tag>
        </div>
      </template>
    </el-table-column>
    <el-table-column :label="t('sys.files.permissions') + ' / ' + t('sys.files.owner')" width="200">
      <template #default="{ row }">
        <span v-if="row.permissions || row.owner">
          {{ row.permissions }}
          <template v-if="row.permissions && row.owner">/</template>
          {{ row.owner }}
        </span>
        <span v-else>-</span>
      </template>
    </el-table-column>
    <el-table-column :label="t('sys.files.size')" width="120">
      <template #default="{ row }">
        <span v-if="!row.is_dir">{{ formatSize(row.size) }}</span>
        <span v-else-if="row._size !== undefined">{{ formatSize(row._size) }}</span>
        <el-button v-else link type="primary" size="small" :loading="row._calcLoading" @click="$emit('calc-size', row)">计算</el-button>
      </template>
    </el-table-column>
    <el-table-column prop="modified" :label="t('sys.files.modified')" width="180" />
    <el-table-column :label="t('sys.files.note')" min-width="180">
      <template #default="{ row }">
        <div class="note-cell" @mouseenter="$emit('note-enter', row)" @mouseleave="$emit('note-leave', row)">
          <template v-if="hoverNotePath === row.path">
            <el-input
              :model-value="editingNote"
              @update:model-value="$emit('note-update', $event)"
              size="small"
              :placeholder="t('sys.files.notePlaceholder')"
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
        <el-button link type="primary" size="small" @click="$emit('rename', row)">{{ t('sys.files.rename') }}</el-button>
        <el-button link type="danger" size="small" @click="$emit('delete', row)">{{ t('sys.files.delete') }}</el-button>
      </template>
    </el-table-column>
  </el-table>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import OnIcon from '@/components/OnIcon/index.vue'
import type { FileItem } from './useFileManager'

const { t } = useI18n()

// ponytail: 真正防溢出靠的是 max-height —— el-table 自带 scroll,父容器约束高度即可。
const props = withDefaults(
  defineProps<{
    files: FileItem[]
    getFileIcon: (row: FileItem) => string
    formatSize: (bytes: number) => string
    hoverNotePath: string
    editingNote: string
    noSelection?: boolean
    /** picker 用:单选用 radio 列,多选用 checkbox */
    radioMode?: boolean
    /** radio 模式下:目录行是否禁用(file 模式应禁用,只让文件可勾) */
    disableDirs?: boolean
  }>(),
  { radioMode: false, disableDirs: false }
)

const tableStyle = computed(() => ({
  height: '100%',
  maxHeight: '100%',
  overflow: 'auto',
}))

// ponytail: radio 模式本地 ref 维护"当前 path",el-radio v-model 绑这个;
// radio 列的勾选态天然由 v-model 联动,selectable 由 :disabled 控制(目录行禁用)。
const pickedPath = ref<string>('')

// 点行 = 选中(file 模式 + radio 模式:目录行被 disableDirs 禁用,跳过)
function onRowClick(row: FileItem) {
  if (!props.radioMode) return
  if (props.disableDirs && row.is_dir) return
  pickedPath.value = row.path
  emit('selection-change', [row])
}

// 直接点 radio 圆圈也会触发 v-model,@change 只用于通知 FileTabPanel
function onRadioChange(row: FileItem) {
  if (props.disableDirs && row.is_dir) return
  emit('selection-change', [row])
}

// ponytail: assign defineEmits return to bind template helpers + script helpers
const emit = defineEmits<{
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
