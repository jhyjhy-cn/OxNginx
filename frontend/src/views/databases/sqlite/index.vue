<template>
  <div class="databases-page">
    <el-card>
      <div class="toolbar">
        <HasPermission code="dbm:add">
          <el-button type="primary" :icon="Plus" @click="onAdd">
            {{ $t('dbm.menu.dbAdd') }}
          </el-button>
        </HasPermission>
      </div>
      <SqliteTable
        :rows="sqliteRows"
        :loading="loadingSqlite"
        @manage="onManage"
        @remove="onRemove"
        @open-file="onOpenFile"
      />
    </el-card>

    <SqliteFormDialog v-model="addVisible" @saved="fetch" />
    <ManageDialog v-model:visible="manageVisible" :db="manageTarget" />

    <!-- 统一确认弹窗(替代 ElMessageBox,样式更稳定) -->
    <OnDialog
      v-model="confirmVisible"
      :title="confirmState.title"
      width="420px"
      :maximizable="false"
      :close-on-click-modal="false"
    >
      <div class="confirm-body">{{ confirmState.message }}</div>
      <el-checkbox
        v-if="confirmState.checkboxLabel"
        v-model="confirmState.checkboxChecked"
        style="margin-top: 8px"
      >
        {{ confirmState.checkboxLabel }}
      </el-checkbox>
      <template #footer>
        <el-button @click="onConfirmCancel">{{ $t('common.cancel') }}</el-button>
        <el-button
          :type="confirmState.danger ? 'danger' : 'primary'"
          :loading="confirmState.loading"
          @click="onConfirmOk"
        >
          {{ $t('common.delete') }}
        </el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import HasPermission from '@/components/HasPermission/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import SqliteTable from './SqliteTable.vue'
import SqliteFormDialog from './SqliteFormDialog.vue'
import ManageDialog from './ManageDialog.vue'
import { listDatabases, deleteDatabase } from '@/api/databases'
import type { DatabaseConn } from '@/api/databases/type'
import { useMessage } from '@/hooks'

const { t } = useI18n()
const { success, error, info } = useMessage()

const all = ref<any[]>([])
const loadingSqlite = ref(false)

const sqliteRows = computed(() => all.value.filter((d) => d.type === 'sqlite'))

async function fetch() {
  loadingSqlite.value = true
  try {
    const data = (await listDatabases()) || []
    all.value = data as any[]
  } catch (e: any) {
    error(e?.message || t('common.failed'))
  } finally {
    loadingSqlite.value = false
  }
}

const addVisible = ref(false)
function onAdd() {
  addVisible.value = true
}

const manageVisible = ref(false)
const manageTarget = ref<DatabaseConn | null>(null)
function onManage(row: DatabaseConn) {
  manageTarget.value = row
  manageVisible.value = true
}

// ========== 确认弹窗(本地 OnDialog) ==========
interface ConfirmState {
  title: string
  message: string
  danger?: boolean
  checkboxLabel?: string
  checkboxChecked: boolean
  loading: boolean
  resolve?: (v: boolean) => void
}
const confirmVisible = ref(false)
const confirmState = reactive<ConfirmState>({
  title: '',
  message: '',
  checkboxChecked: false,
  loading: false,
})
function showConfirm(opts: {
  title?: string
  message: string
  danger?: boolean
  checkboxLabel?: string
}): Promise<{ ok: boolean; checkboxChecked: boolean }> {
  confirmState.title = opts.title || t('common.tip')
  confirmState.message = opts.message
  confirmState.danger = opts.danger
  confirmState.checkboxLabel = opts.checkboxLabel
  confirmState.checkboxChecked = false
  confirmState.loading = false
  confirmVisible.value = true
  return new Promise((resolve) => {
    confirmState.resolve = (ok: boolean) =>
      resolve({ ok, checkboxChecked: confirmState.checkboxChecked })
  })
}
function onConfirmOk() {
  confirmState.resolve?.(true)
  confirmState.resolve = undefined
  confirmVisible.value = false
}
function onConfirmCancel() {
  confirmState.resolve?.(false)
  confirmState.resolve = undefined
  confirmVisible.value = false
}

async function onRemove(row: DatabaseConn) {
  const res = await showConfirm({
    message: t('dbm.confirmDeleteDb'),
    danger: true,
    checkboxLabel: row.type === 'sqlite' ? t('dbm.confirmDeleteFile') : undefined,
  })
  if (!res.ok) return
  try {
    await deleteDatabase(row.id, res.checkboxChecked)
    success(t('common.success'))
    fetch()
  } catch (e: any) {
    error(e?.message || t('common.failed'))
  }
}

function onOpenFile(row: DatabaseConn) {
  info(row.db_name || row.db_path || '-')
}

onMounted(fetch)
</script>

<style scoped>
.databases-page {
  padding: 0;
}
.toolbar {
  margin-bottom: 12px;
}
.confirm-body {
  font-size: 14px;
  line-height: 1.6;
}
</style>
