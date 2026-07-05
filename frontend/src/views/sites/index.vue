<template>
  <div class="sites">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('sites.title') }}</span>
          <div>
            <el-button-group v-if="selectedSites.length > 0" style="margin-right: 12px">
              <el-button size="small" @click="batchEnable">{{ $t('sites.batchEnable') }} ({{ selectedSites.length }})</el-button>
              <el-button size="small" @click="batchDisable">{{ $t('sites.batchDisable') }} ({{ selectedSites.length }})</el-button>
              <el-button size="small" type="danger" @click="batchDelete">{{ $t('sites.batchDelete') }} ({{ selectedSites.length }})</el-button>
            </el-button-group>
            <el-button type="primary" @click="addVisible = true">
              <el-icon><Plus /></el-icon> {{ $t('sites.addSite') }}
            </el-button>
          </div>
        </div>
      </template>

      <SiteTable
        :sites="sites"
        :loading="loading"
        :traffic-metric="trafficMetric"
        @edit="onEdit"
        @open-backup="onOpenBackup"
        @toggle="toggleSite"
        @deploy-ssl="deploySSL"
        @delete="onDelete"
        @selection-change="(val: Site[]) => selectedSites = val"
        @open-file-manager="openFileManager"
        @update:traffic-metric="(v: string) => trafficMetric = v as any"
      />
    </el-card>

    <SiteAddDialog v-model:visible="addVisible" @created="fetchSites" />

    <SiteEditDialog
      v-model:visible="editVisible"
      :site-id="editTarget?.id ?? null"
      :site-name="editTarget?.name ?? ''"
      @saved="fetchSites"
    />

    <SiteDeleteDialog
      v-model:visible="deleteVisible"
      :site="deleteTarget"
      @deleted="fetchSites"
    />

    <SiteBackupDialog
      v-model:visible="backupVisible"
      :site="backupTarget"
      @refresh="fetchSites"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'
import { useTabStore } from '@/stores/tabs'
import { useFilesStore } from '@/stores/files'
import { useSites } from './useSites'
import type { Site } from './types'
import SiteTable from './SiteTable.vue'
import SiteAddDialog from './SiteAddDialog.vue'
import SiteEditDialog from './SiteEditDialog.vue'
import SiteDeleteDialog from './SiteDeleteDialog.vue'
import SiteBackupDialog from './SiteBackupDialog.vue'

const { t } = useI18n()
const router = useRouter()

const {
  sites, selectedSites, loading, trafficMetric,
  fetchSites, toggleSite, batchEnable, batchDisable, batchDelete,
} = useSites()

// ---- 弹窗状态 ----
const addVisible = ref(false)

const editVisible = ref(false)
const editTarget = ref<Site | null>(null)

const deleteVisible = ref(false)
const deleteTarget = ref<Site | null>(null)

const backupVisible = ref(false)
const backupTarget = ref<Site | null>(null)

// ---- 事件处理 ----
function onEdit(site: Site) {
  editTarget.value = site
  editVisible.value = true
}

function onDelete(site: Site) {
  deleteTarget.value = site
  deleteVisible.value = true
}

function onOpenBackup(site: Site) {
  backupTarget.value = site
  backupVisible.value = true
}

async function deploySSL(site: Site) {
  try {
    await ElMessageBox.confirm(
      t('sites.sslDeployConfirm', { domain: site.server_name }),
      t('sites.sslDeploy'),
      { type: 'warning' },
    )
    const response = await api.post(`/api/sites/${site.id}/deploy-ssl`)
    if (response.data.code === 0) {
      ElMessage.success(t('sites.sslDeploySuccess'))
      fetchSites()
    } else {
      ElMessage.error(response.data.message || t('sites.deployFailed'))
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(
        error.response?.data?.message || error.message || t('sites.deployFailed'),
      )
    }
  }
}

function openFileManager(path: string) {
  const tabStore = useTabStore()
  const filesStore = useFilesStore()
  tabStore.addTab({ path: '/files', title: 'menu.files', closable: true })
  const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '')
  const existing = filesStore.tabs.find(
    (t) => t.path.replace(/\\/g, '/').replace(/\/+$/, '') === normalized,
  )
  if (existing) {
    filesStore.setActiveTab(existing.id)
  } else {
    filesStore.addTab(path)
  }
  router.push('/files')
}

onMounted(() => {
  fetchSites()
})
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>

<style>
.el-dropdown-menu__item.active {
  color: var(--el-color-primary);
  font-weight: 600;
}
</style>
