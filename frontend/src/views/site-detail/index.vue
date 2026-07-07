<template>
  <div class="site-detail">
    <el-card v-loading="loading">
      <template #header>
        <div class="card-header">
          <span>{{ $t('siteDetail.title') }}</span>
          <el-button @click="$router.back()">{{ $t('common.back') }}</el-button>
        </div>
      </template>

      <template v-if="site">
        <el-descriptions :column="2" border>
          <el-descriptions-item :label="$t('siteDetail.name')">{{ site.name }}</el-descriptions-item>
          <el-descriptions-item :label="$t('siteDetail.domain')">{{ site.server_name }}</el-descriptions-item>
          <el-descriptions-item :label="$t('siteDetail.port')">{{ site.listen }}</el-descriptions-item>
          <el-descriptions-item label="SSL">
            <el-tag :type="site.ssl ? 'success' : 'info'" size="small">
              {{ site.ssl ? $t('common.yes') : $t('common.no') }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="$t('siteDetail.proxyPass')">{{ site.proxy_pass || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="$t('siteDetail.rootPath')">{{ site.root_path || '-' }}</el-descriptions-item>
          <el-descriptions-item :label="$t('common.status')">
            <el-tag :type="site.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ site.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="$t('common.createdAt')">{{ site.created_at }}</el-descriptions-item>
        </el-descriptions>

        <div style="margin-top: 20px">
          <h3>{{ $t('siteDetail.generatedConfig') }}</h3>
          <el-input
            v-model="configContent"
            type="textarea"
            :rows="15"
            readonly
            style="margin-top: 10px"
          />
        </div>

        <!-- 备份管理 -->
        <el-card style="margin-top: 20px">
          <template #header>
            <div class="card-header">
              <span>{{ $t('siteDetail.backupManagement') }}</span>
              <el-button type="primary" size="small" @click="createBackup">
                {{ $t('siteDetail.createBackup') }}
              </el-button>
            </div>
          </template>

          <el-table :data="backups" style="width: 100%">
            <el-table-column prop="version" :label="$t('common.version')" width="100" />
            <el-table-column prop="created_at" :label="$t('common.createdAt')" width="180" />
            <el-table-column :label="$t('common.action')" width="250">
              <template #default="{ row }">
                <el-button size="small" @click="restoreBackup(row)">{{ $t('siteDetail.restore') }}</el-button>
                <el-button size="small" @click="viewBackup(row)">{{ $t('siteDetail.view') }}</el-button>
                <el-button size="small" type="danger" @click="deleteBackup(row)">{{ $t('common.delete') }}</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </template>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'

const { t } = useI18n()

interface Backup {
  id: number
  version: number
  created_at: string
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
    const response = await api.get(`/api/sites/${route.params.id}`)
    if (response.data.code === 0) {
      site.value = response.data.data
      generateConfig()
    }
  } catch (error) {
    console.error('获取站点详情失败:', error)
  } finally {
    loading.value = false
  }
}

async function fetchBackups() {
  try {
    const response = await api.get(`/api/backups/${route.params.id}`)
    if (response.data.code === 0) {
      backups.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取备份列表失败:', error)
  }
}

async function createBackup() {
  try {
    const response = await api.post(`/api/backups/${route.params.id}`)
    if (response.data.code === 0) {
      ElMessage.success(t('siteDetail.backupCreateSuccess'))
      fetchBackups()
    } else {
      ElMessage.error(response.data.message || t('siteDetail.createFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('siteDetail.createFailed'))
  }
}

async function restoreBackup(backup: Backup) {
  try {
    await ElMessageBox.confirm(t('siteDetail.restoreConfirm', { version: backup.version }), t('common.tip'))
    const response = await api.post(`/api/backups/restore/${backup.id}`)
    if (response.data.code === 0) {
      ElMessage.success(t('siteDetail.restoreSuccess'))
      fetchSite()
    } else {
      ElMessage.error(response.data.message || t('siteDetail.restoreFailed'))
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('siteDetail.restoreFailed'))
    }
  }
}

function viewBackup(_backup: Backup) {
  // TODO: 实现查看备份内容
  ElMessage.info(t('siteDetail.viewDeveloping'))
}

async function deleteBackup(backup: Backup) {
  try {
    await ElMessageBox.confirm(t('siteDetail.deleteConfirm', { version: backup.version }), t('common.warning'), {
      type: 'warning',
    })
    const response = await api.delete(`/api/backups/${backup.id}`)
    if (response.data.code === 0) {
      ElMessage.success(t('siteDetail.deleteSuccess'))
      fetchBackups()
    } else {
      ElMessage.error(response.data.message || t('siteDetail.deleteFailed'))
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('siteDetail.deleteFailed'))
    }
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
