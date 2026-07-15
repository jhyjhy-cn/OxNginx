<template>
  <el-table :data="rows" v-loading="loading" style="width: 100%">
    <el-table-column :label="$t('dbm.colDbName')" prop="name" min-width="140" />
    <el-table-column :label="$t('dbm.colRootPath')" min-width="240" show-overflow-tooltip>
      <template #default="{ row }">
        <el-button type="primary" link @click="emit('open-file', row)">
          {{ row.db_name || row.db_path || '-' }}
        </el-button>
      </template>
    </el-table-column>
    <el-table-column :label="$t('dbm.colBackup')" width="100">
      <template #default>
        <el-tag type="info" size="small">{{ $t('dbm.noBackup') }}</el-tag>
      </template>
    </el-table-column>
    <el-table-column :label="$t('dbm.colSize')" width="120">
      <template #default="{ row }">
        {{ formatBytes(row._size_bytes) }}
      </template>
    </el-table-column>
    <el-table-column :label="$t('common.action')" width="200" fixed="right">
      <template #default="{ row }">
        <HasPermission code="dbm:edit">
          <el-button type="primary" link @click="emit('manage', row)">
            {{ $t('dbm.manage') }}
          </el-button>
        </HasPermission>
        <HasPermission code="dbm:delete">
          <el-button type="danger" link @click="emit('remove', row)">
            {{ $t('common.delete') }}
          </el-button>
        </HasPermission>
      </template>
    </el-table-column>
  </el-table>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import HasPermission from '@/components/HasPermission/index.vue'
import type { DatabaseConn } from '@/api/databases/type'

defineProps<{ rows: any[]; loading: boolean }>()
const emit = defineEmits<{
  manage: [row: DatabaseConn]
  remove: [row: DatabaseConn]
  'open-file': [row: DatabaseConn]
}>()
const { t: _t } = useI18n()

function formatBytes(n?: number | null) {
  if (!n || n <= 0) return '-'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let v = n
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024
    i++
  }
  return `${v.toFixed(v >= 10 || i === 0 ? 0 : 1)} ${units[i]}`
}
</script>
