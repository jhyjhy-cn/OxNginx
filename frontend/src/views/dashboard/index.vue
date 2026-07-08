<template>
  <div class="dashboard">
    <!-- Nginx 状态卡片 -->
    <el-card class="status-card">
      <div class="status-content">
        <div class="status-left">
          <div class="status-icon" :class="nginxStatus.running ? 'running' : 'stopped'">
            <el-icon :size="32">
              <SuccessFilled v-if="nginxStatus.running" />
              <CircleCloseFilled v-else />
            </el-icon>
          </div>
          <div class="status-info">
            <h2>
              Nginx
              {{
                nginxStatus.running
                  ? $t('dashboard.running')
                  : nginxStatus.not_installed
                    ? $t('dashboard.notInstalled')
                    : $t('dashboard.stopped')
              }}
            </h2>
            <div class="status-meta">
              <span v-if="nginxStatus.version">{{ $t('dashboard.version') }}: {{ nginxStatus.version }}</span>
              <span v-if="nginxStatus.pid">PID: {{ nginxStatus.pid }}</span>
              <span v-if="nginxStatus.uptime">{{ $t('dashboard.uptime') }}: {{ nginxStatus.uptime }}</span>
            </div>
          </div>
        </div>
        <div class="status-actions">
          <el-button v-if="nginxStatus.not_installed" type="primary" :loading="loading.install" @click="installNginx">
            <el-icon v-if="!loading.install"><Download /></el-icon>
            {{ $t('dashboard.install') }}
          </el-button>
          <template v-else>
            <el-button :type="nginxStatus.running ? 'danger' : 'success'" :loading="loading.startStop" @click="toggleNginx">
              <el-icon v-if="!loading.startStop">
                <VideoPlay v-if="!nginxStatus.running" />
                <VideoPause v-else />
              </el-icon>
              {{ nginxStatus.running ? $t('dashboard.stop') : $t('dashboard.start') }}
            </el-button>
            <el-button :loading="loading.restart" @click="restartNginx">
              <el-icon v-if="!loading.restart"><RefreshRight /></el-icon>
              {{ $t('dashboard.restart') }}
            </el-button>
            <el-button :loading="loading.reload" @click="reloadConfig">
              <el-icon v-if="!loading.reload"><Refresh /></el-icon>
              {{ $t('dashboard.reloadConfig') }}
            </el-button>
            <el-button :loading="loading.test" @click="testConfig">
              <el-icon v-if="!loading.test"><Finished /></el-icon>
              {{ $t('dashboard.testConfig') }}
            </el-button>
          </template>
        </div>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #409eff22; color: #409eff">
          <el-icon :size="24"><Monitor /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="stats.site_count" :duration="800">
            <template #title>
              <div class="stat-label">{{ $t('dashboard.sites') }}</div>
            </template>
          </el-statistic>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #67c23a22; color: #67c23a">
          <el-icon :size="24"><Lock /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="stats.cert_count" :duration="800">
            <template #title>
              <div class="stat-label">{{ $t('dashboard.certificates') }}</div>
            </template>
          </el-statistic>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #e6a23c22; color: #e6a23c">
          <el-icon :size="24"><Cpu /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="stats.cpu_usage" :precision="1" :duration="800" suffix="%">
            <template #title>
              <div class="stat-label">{{ $t('dashboard.cpuUsage') }}</div>
            </template>
          </el-statistic>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #f56c6c22; color: #f56c6c">
          <el-icon :size="24"><Coin /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="stats.memory_usage" :precision="1" :duration="800" suffix="%">
            <template #title>
              <div class="stat-label">{{ $t('dashboard.memoryUsage') }}</div>
            </template>
          </el-statistic>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #90939922; color: #909399">
          <el-icon :size="24"><Box /></el-icon>
        </div>
        <div class="stat-info">
          <el-statistic :value="stats.app_memory" :duration="800" suffix=" MB">
            <template #title>
              <div class="stat-label">{{ $t('dashboard.appMemory') }}</div>
            </template>
          </el-statistic>
        </div>
      </el-card>
    </div>

    <!-- 快捷操作 -->
    <el-card class="quick-card">
      <template #header>
        <span>{{ $t('dashboard.quickActions') }}</span>
      </template>
      <el-row :gutter="20">
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/sites')">
            <el-icon :size="28"><Monitor /></el-icon>
            <span>{{ $t('menu.sites') }}</span>
          </el-button>
        </el-col>
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/ssl')">
            <el-icon :size="28"><Lock /></el-icon>
            <span>{{ $t('dashboard.certificates') }}</span>
          </el-button>
        </el-col>
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/logs')">
            <el-icon :size="28"><Document /></el-icon>
            <span>{{ $t('dashboard.viewLogs') }}</span>
          </el-button>
        </el-col>
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/settings')">
            <el-icon :size="28"><Setting /></el-icon>
            <span>{{ $t('dashboard.sysSettings') }}</span>
          </el-button>
        </el-col>
      </el-row>
    </el-card>

    <!-- 系统信息 -->
    <el-card class="system-card">
      <template #header>
        <span>{{ $t('dashboard.systemInfo') }}</span>
      </template>
      <el-descriptions :column="3" border>
        <el-descriptions-item :label="$t('dashboard.osVersion')">{{ systemInfo.os }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.arch')">{{ systemInfo.arch }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.hostname')">{{ systemInfo.hostname }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.cpuCores')">{{ systemInfo.cpu_cores }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.nginxVersion')">{{ systemInfo.nginx_version || '-' }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.rustVersion')">{{ systemInfo.rust_version }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.serverAddress')">{{ systemInfo.host }}:{{ systemInfo.port }}</el-descriptions-item>
        <el-descriptions-item :label="$t('dashboard.oxnginxVersion')">1.0.0</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import {
  SuccessFilled,
  CircleCloseFilled,
  VideoPlay,
  VideoPause,
  RefreshRight,
  Refresh,
  Finished,
  Monitor,
  Lock,
  Cpu,
  Coin,
  Box,
  Document,
  Setting,
  Download,
} from '@element-plus/icons-vue'
import api from '@/api'
import { useAuthStore } from '@/stores/auth'

const { t } = useI18n()
const authStore = useAuthStore()

const nginxStatus = ref({
  running: false,
  pid: null as number | null,
  version: null as string | null,
  uptime: null as string | null,
  not_installed: false,
})

const stats = ref({
  nginx_version: '',
  worker_count: 0,
  active_connections: 0,
  site_count: 0,
  cert_count: 0,
  cpu_usage: 0,
  memory_usage: 0,
  memory_total: 0,
  app_memory: 0,
})

const systemInfo = reactive({
  os: '',
  arch: '',
  hostname: '',
  cpu_cores: 0,
  nginx_version: '',
  rust_version: '',
  host: '',
  port: 0,
})

const loading = reactive({
  test: false,
  reload: false,
  startStop: false,
  restart: false,
  install: false,
})

let ws: WebSocket | null = null
let reconnectTimer: ReturnType<typeof setTimeout> | null = null
let reconnectDelay = 1000

onMounted(() => {
  fetchSystemInfo()
  connectWs()
})

onUnmounted(() => {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
  ws?.close()
})

function connectWs() {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  const token = authStore.token
  ws = new WebSocket(`${protocol}//${location.host}/api/dashboard/ws?token=${token}`)

  ws.onmessage = (e) => {
    try {
      const msg = JSON.parse(e.data)
      if (msg.nginx) nginxStatus.value = msg.nginx
      if (msg.stats) stats.value = { ...stats.value, ...msg.stats }
      reconnectDelay = 1000
    } catch (err) {
      console.error('Dashboard WS 解析失败:', err)
    }
  }

  ws.onclose = () => {
    reconnectTimer = setTimeout(connectWs, reconnectDelay)
    reconnectDelay = Math.min(reconnectDelay * 2, 30000)
  }

  ws.onerror = () => ws?.close()
}

async function fetchSystemInfo() {
  try {
    const response = await api.get('/api/settings')
    if (response.data.code === 0) {
      const data = response.data.data
      Object.assign(systemInfo, data.system)
      systemInfo.host = data.server.host
      systemInfo.port = data.server.port
    }
  } catch (error) {
    console.error('获取系统信息失败:', error)
  }
}

async function toggleNginx() {
  loading.startStop = true
  try {
    if (nginxStatus.value.running) {
      const response = await api.post('/api/nginx/stop')
      if (response.data.code === 0) {
        ElMessage.success(t('dashboard.nginxStopped'))
      } else {
        ElMessage.error(response.data.message || t('dashboard.stopFailed'))
      }
    } else {
      const response = await api.post('/api/nginx/start')
      if (response.data.code === 0) {
        ElMessage.success(t('dashboard.nginxStarted'))
      } else {
        ElMessage.error(response.data.message || t('dashboard.startFailed'))
      }
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('dashboard.operationFailed'))
  } finally {
    loading.startStop = false
  }
}

async function restartNginx() {
  loading.restart = true
  try {
    const response = await api.post('/api/nginx/restart')
    if (response.data.code === 0) {
      ElMessage.success(t('dashboard.nginxRestarted'))
    } else {
      ElMessage.error(response.data.message || t('dashboard.restartFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('dashboard.restartFailed'))
  } finally {
    loading.restart = false
  }
}

async function reloadConfig() {
  loading.reload = true
  try {
    const response = await api.post('/api/nginx/reload')
    if (response.data.code === 0) {
      ElMessage.success(t('dashboard.configReloaded'))
    } else {
      ElMessage.error(response.data.message || t('dashboard.reloadFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('dashboard.reloadFailed'))
  } finally {
    loading.reload = false
  }
}

async function testConfig() {
  loading.test = true
  try {
    const response = await api.post('/api/nginx/test')
    if (response.data.code === 0) {
      const result = response.data.data
      if (result.success) {
        ElMessage.success(t('dashboard.configTestPassed'))
      } else {
        ElMessage.error(t('dashboard.configTestFailed') + ': ' + result.message)
      }
    } else {
      ElMessage.error(response.data.message || t('dashboard.testFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('dashboard.testFailed'))
  } finally {
    loading.test = false
  }
}

async function installNginx() {
  loading.install = true
  try {
    ElMessage.info(t('dashboard.installing'))
    const response = await api.post('/api/nginx/install', null, { timeout: 300000 })
    if (response.data.code === 0) {
      ElMessage.success(t('dashboard.installSuccess'))
    } else {
      ElMessage.error(response.data.message || t('dashboard.installFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('dashboard.installFailed'))
  } finally {
    loading.install = false
  }
}
</script>

<style scoped>
.dashboard {
  padding: 20px;
  background: var(--el-bg-color-page);
  min-height: 100vh;
}

.status-card {
  margin-bottom: 20px;
  border-radius: 12px;
  border: none;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
}

.status-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.status-icon.running {
  background: linear-gradient(135deg, #67c23a22 0%, #67c23a11 100%);
  color: #67c23a;
  box-shadow: 0 4px 12px rgba(103, 194, 58, 0.2);
}

.status-icon.stopped {
  background: linear-gradient(135deg, #f56c6c22 0%, #f56c6c11 100%);
  color: #f56c6c;
  box-shadow: 0 4px 12px rgba(245, 108, 108, 0.2);
}

.status-info h2 {
  margin: 0 0 8px 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.status-meta {
  display: flex;
  gap: 20px;
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.status-meta span {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-actions {
  display: flex;
  gap: 12px;
}

.status-actions .el-button {
  border-radius: 8px;
  font-weight: 500;
  padding: 10px 20px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

@media (max-width: 1200px) {
  .stats-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 480px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
}

.stat-card {
  border-radius: 12px;
  border: none;
  transition: all 0.3s ease;
  cursor: default;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.stat-card :deep(.el-card__body) {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
}

.stat-icon {
  width: 52px;
  height: 52px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-info {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--el-text-color-primary);
  line-height: 1.2;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.quick-card {
  margin-bottom: 20px;
  border-radius: 12px;
  border: none;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
}

.quick-card :deep(.el-card__header) {
  padding: 16px 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.quick-btn {
  width: 100%;
  height: 90px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  font-size: 14px;
  font-weight: 500;
  border-radius: 12px;
  border: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
  transition: all 0.3s ease;
  color: var(--el-text-color-regular);
}

.quick-btn:hover {
  border-color: #409eff;
  color: #409eff;
  background: #ecf5ff;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.15);
}

.quick-btn .el-icon {
  transition: transform 0.3s ease;
}

.quick-btn:hover .el-icon {
  transform: scale(1.1);
}

.system-card {
  margin-bottom: 20px;
  border-radius: 12px;
  border: none;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
}

.system-card :deep(.el-card__header) {
  padding: 16px 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.system-card :deep(.el-descriptions) {
  border-radius: 8px;
}

.system-card :deep(.el-descriptions__label) {
  font-weight: 500;
  color: var(--el-text-color-regular);
  background: var(--el-fill-color-light);
}

.system-card :deep(.el-descriptions__content) {
  color: var(--el-text-color-primary);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .dashboard {
    padding: 12px;
  }

  .status-content {
    flex-direction: column;
    gap: 20px;
    align-items: flex-start;
  }

  .status-actions {
    width: 100%;
    flex-wrap: wrap;
  }

  .status-actions .el-button {
    flex: 1;
    min-width: calc(50% - 6px);
  }

  .stat-card :deep(.el-card__body) {
    padding: 16px;
  }

  .stat-value {
    font-size: 16px;
  }

  .quick-btn {
    height: 72px;
  }
}

@media (max-width: 480px) {
  .status-icon {
    width: 48px;
    height: 48px;
  }

  .status-info h2 {
    font-size: 16px;
  }

  .status-meta {
    flex-direction: column;
    gap: 4px;
  }
}

/* 动画效果 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.status-card,
.stat-card,
.quick-card,
.system-card {
  animation: fadeIn 0.4s ease-out;
}

.stat-card:nth-child(1) {
  animation-delay: 0.05s;
}
.stat-card:nth-child(2) {
  animation-delay: 0.1s;
}
.stat-card:nth-child(3) {
  animation-delay: 0.15s;
}
.stat-card:nth-child(4) {
  animation-delay: 0.2s;
}
.stat-card:nth-child(5) {
  animation-delay: 0.25s;
}
</style>
