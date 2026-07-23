<template>
  <div class="sites h-full">
    <el-card class="h-full">
      <SiteTable
        :sites="pagedSites"
        :loading="loading"
        :traffic-metric="trafficMetric"
        :pagination="{ total: sites.length, currentPage: page, pageSize }"
        @page-change="onPageChange"
        @edit="onEdit"
        @open-backup="onOpenBackup"
        @toggle="toggleSite"
        @deploy-ssl="deploySSL"
        @delete="onDelete"
        @selection-change="(val: Site[]) => (selectedSites = val)"
        @open-file-manager="openFileManager"
        @update:traffic-metric="(v: string) => (trafficMetric = v as any)"
        @reload="fetchSites"
      >
        <template #toolbar-left>
          <el-button-group v-if="selectedSites.length > 0">
            <el-button size="small" @click="batchEnable">{{ $t('sys.sites.batchEnable') }} ({{ selectedSites.length }})</el-button>
            <el-button size="small" @click="batchDisable">{{ $t('sys.sites.batchDisable') }} ({{ selectedSites.length }})</el-button>
            <el-button size="small" type="danger" @click="batchDelete">
              {{ $t('sys.sites.batchDelete') }} ({{ selectedSites.length }})
            </el-button>
          </el-button-group>
          <el-button type="primary" @click="addVisible = true">
            <el-icon><Plus /></el-icon>
            {{ $t('sys.sites.addSite') }}
          </el-button>
          <NginxStatusButton />
        </template>
      </SiteTable>
    </el-card>

    <SiteAddDialog v-model:visible="addVisible" @created="fetchSites" />

    <SiteEditDialog
      v-model:visible="editVisible"
      :site-id="editTarget?.id ?? null"
      :site-name="editTarget?.name ?? ''"
      @saved="fetchSites"
    />

    <SiteDeleteDialog v-model:visible="deleteVisible" :site="deleteTarget" @deleted="fetchSites" />

    <SiteBackupDialog v-model:visible="backupVisible" :site="backupTarget" @refresh="fetchSites" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useTabStore } from '@/stores/tabs'
import { useFilesStore } from '@/stores/files'
import { useSites } from './useSites'
import type { Site } from './types'
import { deploySsl } from '@/api/sites'
import { useMessage } from '@/hooks'
import SiteTable from './SiteTable.vue'
import NginxStatusButton from './NginxStatusButton.vue'
import SiteAddDialog from './SiteAddDialog.vue'
import SiteEditDialog from './SiteEditDialog.vue'
import SiteDeleteDialog from './SiteDeleteDialog.vue'
import SiteBackupDialog from './SiteBackupDialog.vue'

const { t } = useI18n()
const { confirm } = useMessage()
const router = useRouter()

const { sites, pagedSites, page, pageSize, onPageChange, selectedSites, loading, trafficMetric, fetchSites, toggleSite, batchEnable, batchDisable, batchDelete } = useSites()

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
  const ok = await confirm({
    message: 'sys.sites.sslDeployConfirm',
    params: { domain: site.server_name },
    title: 'sys.sites.sslDeploy',
  })
  if (!ok) return
  try {
    await deploySsl(site.id)
    ElMessage.success(t('sys.sites.sslDeploySuccess'))
    fetchSites()
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.sites.deployFailed'))
  }
}

function openFileManager(path: string) {
  const tabStore = useTabStore()
  const filesStore = useFilesStore()
  tabStore.addTab({ path: '/files', title: 'sys.menu.files', closable: true })
  const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '')
  const existing = filesStore.tabs.find((t) => t.path.replace(/\\/g, '/').replace(/\/+$/, '') === normalized)
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

<style>
.el-dropdown-menu__item.active {
  color: var(--el-color-primary);
  font-weight: 600;
}
</style>
