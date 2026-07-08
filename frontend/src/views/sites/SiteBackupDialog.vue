<template>
  <OnDialog v-model="dialogVisible" :title="$t('sites.backupDialogTitle', { name: site?.name || '' })" width="800px">
    <div style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center">
      <el-button v-if="selected.length > 0" type="danger" size="small" @click="batchDeleteBackups">
        {{ $t('common.delete') }} ({{ selected.length }})
      </el-button>
      <span v-else />
      <el-button type="primary" size="small" :loading="creating" @click="createBackup">
        <el-icon><Plus /></el-icon>
        {{ $t('sites.backupSite') }}
      </el-button>
    </div>
    <el-table
      :data="list"
      v-loading="tableLoading"
      style="width: 100%; height: 400px"
      height="400"
      @selection-change="(val: BackupFile[]) => (selected = val)"
    >
      <el-table-column type="selection" width="45" />
      <el-table-column prop="filename" :label="$t('sites.backupFilename')" min-width="180" show-overflow-tooltip />
      <el-table-column :label="$t('sites.backupPath')" min-width="160" show-overflow-tooltip>
        <template #default="{ row }">
          <el-button type="primary" link @click="openFileManager(row.path)">{{ row.path }}</el-button>
        </template>
      </el-table-column>
      <el-table-column :label="$t('sites.backupSize')" width="90">
        <template #default="{ row }">{{ formatSize(row.size) }}</template>
      </el-table-column>
      <el-table-column prop="created_at" :label="$t('sites.backupTime')" width="160" />
      <el-table-column prop="remark" :label="$t('sites.remark')" width="100" show-overflow-tooltip>
        <template #default="{ row }">{{ row.remark || '-' }}</template>
      </el-table-column>
      <el-table-column :label="$t('common.action')" width="120" fixed="right">
        <template #default="{ row }">
          <el-button type="primary" link @click="downloadBackup(row.filename)">{{ $t('common.download') }}</el-button>
          <el-button type="danger" link @click="deleteBackup(row.filename)">{{ $t('common.delete') }}</el-button>
        </template>
      </el-table-column>
    </el-table>
    <div style="margin-top: 12px; display: flex; justify-content: flex-end">
      <el-pagination
        v-model:current-page="page"
        v-model:page-size="pageSize"
        :total="total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        small
        @current-change="fetchList"
        @size-change="fetchList"
      />
    </div>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'
import { useTabStore } from '@/stores/tabs'
import { useFilesStore } from '@/stores/files'
import type { Site, BackupFile } from './types'

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  visible: boolean
  site: Site | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  refresh: []
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

const list = ref<BackupFile[]>([])
const tableLoading = ref(false)
const creating = ref(false)
const selected = ref<BackupFile[]>([])
const page = ref(1)
const pageSize = ref(20)
const total = ref(0)

watch(
  () => props.visible,
  (val) => {
    if (val) {
      page.value = 1
      fetchList()
    }
  }
)

async function fetchList() {
  if (!props.site) return
  tableLoading.value = true
  try {
    const res = await api.get(`/api/sites/${props.site.id}/backups`, {
      params: { page: page.value, page_size: pageSize.value },
    })
    if (res.data.code === 0) {
      list.value = res.data.data?.items || []
      total.value = res.data.data?.total || 0
    }
  } catch {
    list.value = []
  } finally {
    tableLoading.value = false
  }
}

async function createBackup() {
  if (!props.site) return
  creating.value = true
  try {
    const res = await api.post(`/api/sites/${props.site.id}/backups`)
    if (res.data.code === 0) {
      ElMessage.success(t('sites.backupCreated'))
      fetchList()
      emit('refresh')
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('sites.backupCreateFailed'))
  } finally {
    creating.value = false
  }
}

function downloadBackup(filename: string) {
  if (!props.site) return
  const url = `/api/sites/${props.site.id}/backups/${encodeURIComponent(filename)}/download`
  const token = localStorage.getItem('token')
  const a = document.createElement('a')
  a.href = url + (token ? `?token=${token}` : '')
  a.download = filename
  a.click()
}

async function deleteBackup(filename: string) {
  if (!props.site) return
  try {
    await ElMessageBox.confirm(t('sites.confirmDeleteBackup', { name: filename }), t('common.tip'), { type: 'warning' })
    const res = await api.delete(`/api/sites/${props.site.id}/backups/${encodeURIComponent(filename)}`)
    if (res.data.code === 0) {
      ElMessage.success(t('sites.backupDeleted'))
      fetchList()
      emit('refresh')
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('sites.backupDeleteFailed'))
    }
  }
}

async function batchDeleteBackups() {
  if (!props.site || selected.value.length === 0) return
  try {
    await ElMessageBox.confirm(t('sites.confirmBatchDeleteBackup', { count: selected.value.length }), t('common.warning'), {
      type: 'warning',
    })
    const res = await api.post(`/api/sites/${props.site.id}/backups/batch-delete`, {
      filenames: selected.value.map((b) => b.filename),
    })
    if (res.data.code === 0) {
      ElMessage.success(t('sites.backupDeleted'))
      selected.value = []
      fetchList()
      emit('refresh')
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('sites.backupDeleteFailed'))
    }
  }
}

function openFileManager(path: string) {
  const tabStore = useTabStore()
  const filesStore = useFilesStore()
  tabStore.addTab({ path: '/files', title: 'menu.files', closable: true })
  const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '')
  const existing = filesStore.tabs.find((t) => t.path.replace(/\\/g, '/').replace(/\/+$/, '') === normalized)
  if (existing) {
    filesStore.setActiveTab(existing.id)
  } else {
    filesStore.addTab(path)
  }
  router.push('/files')
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}
</script>
