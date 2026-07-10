<template>
  <div class="site-detail">
    <el-card v-loading="loading">
      <template #header>
        <div class="card-header">
          <span>{{ $t('sys.siteDetail.title') }}</span>
          <el-button @click="$router.back()">{{ $t('common.back') }}</el-button>
        </div>
      </template>

      <template v-if="site">
        <el-descriptions :column="2" border>
          <el-descriptions-item :label="$t('sys.siteDetail.name')">{{ site.name }}</el-descriptions-item>
          <el-descriptions-item :label="$t('sys.siteDetail.domain')">{{ site.server_name }}</el-descriptions-item>
          <el-descriptions-item :label="$t('sys.siteDetail.port')">{{ site.listen }}</el-descriptions-item>
          <el-descriptions-item label="SSL">
            <el-tag :type="site.ssl ? 'success' : 'info'" size="small">
              {{ site.ssl ? $t('common.yes') : $t('common.no') }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="$t('sys.siteDetail.proxyPass')">{{ site.proxy_pass || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="$t('sys.siteDetail.rootPath')">{{ site.root_path || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="$t('common.status')">
            <el-tag :type="site.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ site.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="$t('common.createdAt')">{{ site.created_at }}</el-descriptions-item>
        </el-descriptions>

        <div style="margin-top: 20px">
          <h3>{{ $t('sys.siteDetail.generatedConfig') }}</h3>
          <el-input v-model="configContent" type="textarea" :rows="15" readonly style="margin-top: 10px" />
        </div>

        <!-- 备份管理 -->
        <el-card style="margin-top: 20px">
          <template #header>
            <div class="card-header">
              <span>{{ $t('sys.siteDetail.backupManagement') }}</span>
              <el-button type="primary" size="small" @click="createBackup">
                {{ $t('sys.siteDetail.createBackup') }}
              </el-button>
            </div>
          </template>

          <el-table :data="backups" style="width: 100%">
            <el-table-column prop="version" :label="$t('common.version')" width="100" />
            <el-table-column prop="created_at" :label="$t('common.createdAt')" width="180" />
            <el-table-column :label="$t('common.action')" width="250">
              <template #default="{ row }">
                <el-button size="small" @click="restoreBackupAction(row)">{{ $t('sys.siteDetail.restore') }}</el-button>
                <el-button size="small" @click="viewBackup(row)">{{ $t('sys.siteDetail.view') }}</el-button>
                <el-button size="small" type="danger" @click="deleteBackupAction(row)">{{ $t('common.delete') }}</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </template>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  getSite,
  listBackups,
  createBackup as createBackupApi,
  restoreBackup as restoreBackupApi,
  deleteBackup as deleteBackupApiImpl,
} from '@/api/sites'
import { useMessage } from '@/hooks'

const { t } = useI18n()
const { confirm } = useMessage()

interface Backup {
  id: number | string
  version?: number | string
  created_at?: string
  [key: string]: unknown
}

const route = useRoute()
const loading = ref(false)
const site = ref<any>(null)
const configContent = ref('')
const backups = ref<Backup[]>([])

onMounted(() => {
  fetchSite()
  fetchBackups()
})

async function fetchSite() {
  loading.value = true
  try {
    site.value = await getSite(routeId.value)
    generateConfig()
  } catch (error) {
    console.error('获取站点详情失败:', error)
  } finally {
    loading.value = false
  }
}

const routeId = computed(() => Number(route.params.id))
async function fetchBackups() {
  try {
    backups.value = (await listBackups(routeId.value)) || []
  } catch (error) {
    console.error('获取备份列表失败:', error)
  }
}

async function createBackup() {
  try {
    await createBackupApi(routeId.value)
    ElMessage.success(t('sys.siteDetail.backupCreateSuccess'))
    fetchBackups()
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.siteDetail.createFailed'))
  }
}

async function restoreBackupAction(backup: Backup) {
  const ok = await confirm({
    message: 'sys.siteDetail.restoreConfirm',
    params: { version: backup.version },
  })
  if (!ok) return
  try {
    await restoreBackupApi(routeId.value, backup.id)
    ElMessage.success(t('sys.siteDetail.restoreSuccess'))
    fetchSite()
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.siteDetail.restoreFailed'))
  }
}

function viewBackup(_backup: Backup) {
  ElMessage.info(t('sys.siteDetail.viewDeveloping'))
}

async function deleteBackupAction(backup: Backup) {
  const ok = await confirm({
    message: 'sys.siteDetail.deleteConfirm',
    params: { version: backup.version },
    title: 'common.warning',
  })
  if (!ok) return
  try {
    await deleteBackupApiImpl(routeId.value, backup.id)
    ElMessage.success(t('sys.siteDetail.deleteSuccess'))
    fetchBackups()
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.siteDetail.deleteFailed'))
  }
}

function generateConfig() {
  if (!site.value) return

  let config = 'server {\n'

  if (site.value.ssl) {
    config += `    listen ${site.value.listen} ssl;\n`
    config += '    listen [::]:443 ssl;\n'
  } else {
    config += `    listen ${site.value.listen};\n`
    config += `    listen [::]:${site.value.listen};\n`
  }

  config += `    server_name ${site.value.server_name};\n`

  if (site.value.ssl && site.value.certificate_path) {
    config += `    ssl_certificate ${site.value.certificate_path};\n`
    config += `    ssl_certificate_key ${site.value.key_path};\n`
    config += '    ssl_protocols TLSv1.2 TLSv1.3;\n'
  }

  if (site.value.proxy_pass) {
    config += '\n    location / {\n'
    config += `        proxy_pass ${site.value.proxy_pass};\n`
    config += '        proxy_set_header Host $host;\n'
    config += '        proxy_set_header X-Real-IP $remote_addr;\n'
    config += '        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n'
    config += '    }\n'
  } else if (site.value.root_path) {
    config += `\n    root ${site.value.root_path};\n`
    config += '    index index.html;\n'
    config += '\n    location / {\n'
    config += '        try_files $uri $uri/ /index.html;\n'
    config += '    }\n'
  }

  config += '}\n'
  configContent.value = config
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
