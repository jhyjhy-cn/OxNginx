<template>
  <div class="settings">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>系统设置</span>
          <el-button type="primary" @click="saveSettings" :loading="saving">
            保存设置
          </el-button>
        </div>
      </template>

      <el-form label-width="140px">
        <el-divider>Nginx 配置</el-divider>

        <el-form-item label="Nginx 路径">
          <el-input v-model="settings.nginx.bin" placeholder="/usr/sbin/nginx" />
        </el-form-item>
        <el-form-item label="主配置文件">
          <el-input v-model="settings.nginx.config" placeholder="/etc/nginx/nginx.conf" />
        </el-form-item>
        <el-form-item label="站点配置目录">
          <el-input v-model="settings.nginx.sites_enabled" placeholder="/etc/nginx/sites-enabled" />
        </el-form-item>

        <el-divider>ACME 配置</el-divider>

        <el-form-item label="acme.sh 路径">
          <el-input v-model="settings.acme.bin" placeholder="/root/.acme.sh/acme.sh" />
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 20px">
      <template #header>
        <span>系统信息</span>
      </template>

      <el-descriptions :column="2" border>
        <el-descriptions-item label="系统版本">{{ settings.system.os }}</el-descriptions-item>
        <el-descriptions-item label="系统架构">{{ settings.system.arch }}</el-descriptions-item>
        <el-descriptions-item label="主机名">{{ settings.system.hostname }}</el-descriptions-item>
        <el-descriptions-item label="CPU核心数">{{ settings.system.cpu_cores }}</el-descriptions-item>
        <el-descriptions-item label="Nginx版本">{{ settings.system.nginx_version || '-' }}</el-descriptions-item>
        <el-descriptions-item label="Rust版本">{{ settings.system.rust_version }}</el-descriptions-item>
        <el-descriptions-item label="服务地址">{{ settings.server.host }}:{{ settings.server.port }}</el-descriptions-item>
        <el-descriptions-item label="OxNginx版本">1.0.0</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import api from '@/api'

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
      ElMessage.error(response.data.message || '保存失败')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '保存失败')
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
