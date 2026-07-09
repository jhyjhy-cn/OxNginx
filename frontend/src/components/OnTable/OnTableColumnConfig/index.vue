<template>
  <el-popover :popper-style="{ maxHeight: '450px' }" trigger="click" placement="bottom-end" width="300" v-model:visible="showColumnConfig">
    <el-table :data="localColumns" size="small" class="config-table" height="250" stripe>
      <el-table-column type="index" width="40" label="序号" />
      <el-table-column prop="label" label="列名">
        <template #default="{ row }">
          {{ row.label ? t(row.label) : row.prop || '' }}
        </template>
      </el-table-column>
      <el-table-column prop="visible" label="显示" width="80">
        <template #default="scope">
          <el-checkbox v-model="scope.row.visible" />
        </template>
      </el-table-column>
    </el-table>
    <div class="config-actions">
      <el-button size="small" @click="resetColumns">{{ t('common.reset') }}</el-button>
      <el-button size="small" @click="closePopover">{{ t('common.cancel') }}</el-button>
      <el-button size="small" type="primary" @click="confirmChanges">{{ t('common.confirm') }}</el-button>
    </div>
    <template #reference>
      <el-button :icon="Setting" :title="t('common.columnConfig')" />
    </template>
  </el-popover>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Setting } from '@element-plus/icons-vue'
import type { TableColumn } from '../types'

const { t } = useI18n()

const props = defineProps<{
  columns: TableColumn[]
}>()

const emit = defineEmits<{
  'update:columns': [columns: TableColumn[]]
}>()

const localColumns = ref<TableColumn[]>([])
const showColumnConfig = ref(false)

watch(
  () => props.columns,
  (newColumns) => {
    localColumns.value = newColumns.map((col) => ({ ...col }))
  },
  { immediate: true }
)

function resetColumns() {
  localColumns.value.forEach((col) => {
    col.visible = true
  })
}

function closePopover() {
  showColumnConfig.value = false
}

function confirmChanges() {
  emit('update:columns', [...localColumns.value])
  closePopover()
}
</script>

<style scoped>
.config-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px;
}
</style>
