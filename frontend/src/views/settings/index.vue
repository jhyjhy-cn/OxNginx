<template>
  <div class="settings">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('sys.settings.title') }}</span>
          <el-button type="primary" @click="saveSettings" :loading="saving">
            {{ $t('sys.settings.saveSettings') }}
          </el-button>
        </div>
      </template>

      <el-form label-width="140px">
        <el-divider>{{ $t('sys.settings.nginxConfig') }}</el-divider>

        <el-form-item :label="$t('sys.settings.nginxPath')">
          <el-input v-model="settings.nginx.bin" placeholder="/usr/sbin/nginx" />
        </el-form-item>
        <el-form-item :label="$t('sys.settings.mainConfig')">
          <el-input v-model="settings.nginx.config" placeholder="/etc/nginx/nginx.conf" />
        </el-form-item>
        <el-form-item :label="$t('sys.settings.sitesDir')">
          <el-input v-model="settings.nginx.sites_enabled" placeholder="/etc/nginx/sites-enabled" />
        </el-form-item>

        <el-divider>{{ $t('sys.settings.acmeConfig') }}</el-divider>

        <el-form-item :label="$t('sys.settings.acmePath')">
          <el-input v-model="settings.acme.bin" placeholder="/root/.acme.sh/acme.sh" />
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 20px">
      <template #header>
        <span>{{ $t('sys.settings.systemInfo') }}</span>
      </template>

      <el-descriptions :column="2" border>
        <el-descriptions-item :label="$t('sys.settings.osVersion')">{{ settings.system.os }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.arch')">{{ settings.system.arch }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.hostname')">{{ settings.system.hostname }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.cpuCores')">{{ settings.system.cpu_cores }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.nginxVersion')">{{ settings.system.nginx_version || '-' }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.rustVersion')">{{ settings.system.rust_version }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.serverAddress')">
          {{ settings.server.host }}:{{ settings.server.port }}
        </el-descriptions-item>
        <el-descriptions-item :label="$t('sys.settings.oxnginxVersion')">1.0.0</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import api from '@/api'

const { t } = useI18n()

const saving = ref(false)

const settings = reactive({
  server: {
    host: '0.0.0.0',
    port: 3000,
  },
  nginx: {
    bin: '',
    config: '',
    sites_enabled: '',
  },
  acme: {
    bin: '',
  },
  system: {
    os: '',
    arch: '',
    hostname: '',
    cpu_cores: 0,
    nginx_version: '',
    rust_version: '',
  },
})

onMounted(() => {
  fetchSettings()
})

async function fetchSettings() {
  try {
    const response = await api.get('/api/settings')
    if (response.data.code === 0) {
      const data = response.data.data
      Object.assign(settings.server, data.server)
      Object.assign(settings.nginx, data.nginx)
      Object.assign(settings.acme, data.acme)
      Object.assign(settings.system, data.system)
    }
  } catch (error) {
    console.error('获取设置失败:', error)
  }
}

async function saveSettings() {
  saving.value = true
  try {
    const response = await api.put('/api/settings', {
      nginx_bin: settings.nginx.bin,
      nginx_config: settings.nginx.config,
      nginx_sites_enabled: settings.nginx.sites_enabled,
      acme_bin: settings.acme.bin,
    })

    if (response.data.code === 0) {
      ElMessage.success(response.data.data)
    } else {
      ElMessage.error(response.data.message || t('sys.settings.saveFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('sys.settings.saveFailed'))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
