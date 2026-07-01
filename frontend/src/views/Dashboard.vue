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
            <h2>Nginx {{ nginxStatus.running ? '运行中' : (nginxStatus.not_installed ? '未安装' : '已停止') }}</h2>
            <div class="status-meta">
              <span v-if="nginxStatus.version">版本: {{ nginxStatus.version }}</span>
              <span v-if="nginxStatus.pid">PID: {{ nginxStatus.pid }}</span>
              <span v-if="nginxStatus.uptime">运行时间: {{ nginxStatus.uptime }}</span>
            </div>
          </div>
        </div>
        <div class="status-actions">
          <el-button
            :type="nginxStatus.running ? 'danger' : 'success'"
            :loading="loading.startStop"
            @click="toggleNginx"
          >
            <el-icon v-if="!loading.startStop"><VideoPlay v-if="!nginxStatus.running" /><VideoPause v-else /></el-icon>
            {{ nginxStatus.running ? '停止' : '启动' }}
          </el-button>
          <el-button :loading="loading.restart" @click="restartNginx">
            <el-icon v-if="!loading.restart"><RefreshRight /></el-icon>
            重启
          </el-button>
          <el-button :loading="loading.reload" @click="reloadConfig">
            <el-icon v-if="!loading.reload"><Refresh /></el-icon>
            重载配置
          </el-button>
          <el-button :loading="loading.test" @click="testConfig">
            <el-icon v-if="!loading.test"><Finished /></el-icon>
            测试配置
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 统计卡片 -->
    <el-row :gutter="20" class="stats-row">
      <el-col :span="4" :xs="12" :sm="8" :md="4">
        <el-card class="stat-card" shadow="hover">
          <div class="stat-icon" style="background: #409eff22; color: #409eff">
            <el-icon :size="24"><Monitor /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.site_count }}</div>
            <div class="stat-label">站点数</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4" :xs="12" :sm="8" :md="4">
        <el-card class="stat-card" shadow="hover">
          <div class="stat-icon" style="background: #67c23a22; color: #67c23a">
            <el-icon :size="24"><Lock /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.cert_count }}</div>
            <div class="stat-label">SSL证书</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4" :xs="12" :sm="8" :md="4">
        <el-card class="stat-card" shadow="hover">
          <div class="stat-icon" style="background: #e6a23c22; color: #e6a23c">
            <el-icon :size="24"><Cpu /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stats.cpu_usage.toFixed(1) }}%</div>
            <div class="stat-label">CPU使用率</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4" :xs="12" :sm="8" :md="4">
        <el-card class="stat-card" shadow="hover">
          <div class="stat-icon" style="background: #f56c6c22; color: #f56c6c">
            <el-icon :size="24"><Coin /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ formatMemory(stats.memory_usage) }} / {{ formatMemory(stats.memory_total) }}</div>
            <div class="stat-label">系统内存</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="4" :xs="12" :sm="8" :md="4">
        <el-card class="stat-card" shadow="hover">
          <div class="stat-icon" style="background: #90939922; color: #909399">
            <el-icon :size="24"><Box /></el-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ formatMemory(stats.app_memory) }}</div>
            <div class="stat-label">面板内存</div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快捷操作 -->
    <el-card class="quick-card">
      <template #header>
        <span>快捷操作</span>
      </template>
      <el-row :gutter="20">
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/sites')">
            <el-icon :size="28"><Monitor /></el-icon>
            <span>站点管理</span>
          </el-button>
        </el-col>
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/ssl')">
            <el-icon :size="28"><Lock /></el-icon>
            <span>SSL证书</span>
          </el-button>
        </el-col>
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/logs')">
            <el-icon :size="28"><Document /></el-icon>
            <span>日志查看</span>
          </el-button>
        </el-col>
        <el-col :span="6" :xs="12">
          <el-button class="quick-btn" @click="$router.push('/settings')">
            <el-icon :size="28"><Setting /></el-icon>
            <span>系统设置</span>
          </el-button>
        </el-col>
      </el-row>
    </el-card>

    <!-- 系统信息 -->
    <el-card class="system-card">
      <template #header>
        <span>系统信息</span>
      </template>
      <el-descriptions :column="3" border>
        <el-descriptions-item label="系统版本">{{ systemInfo.os }}</el-descriptions-item>
        <el-descriptions-item label="系统架构">{{ systemInfo.arch }}</el-descriptions-item>
        <el-descriptions-item label="主机名">{{ systemInfo.hostname }}</el-descriptions-item>
        <el-descriptions-item label="CPU核心数">{{ systemInfo.cpu_cores }}</el-descriptions-item>
        <el-descriptions-item label="Nginx版本">{{ systemInfo.nginx_version || '-' }}</el-descriptions-item>
        <el-descriptions-item label="Rust版本">{{ systemInfo.rust_version }}</el-descriptions-item>
        <el-descriptions-item label="服务地址">{{ systemInfo.host }}:{{ systemInfo.port }}</el-descriptions-item>
        <el-descriptions-item label="OxNginx版本">1.0.0</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
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
} from '@element-plus/icons-vue'
import api from '@/api'

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
})

onMounted(() => {
  fetchNginxStatus()
  fetchDashboard()
  fetchSystemInfo()
})

function formatMemory(kb: number): string {
  if (kb >= 1048576) {
    return (kb / 1048576).toFixed(1) + ' GB'
  } else if (kb >= 1024) {
    return (kb / 1024).toFixed(0) + ' MB'
  }
  return kb + ' KB'
}

async function fetchNginxStatus() {
  try {
    const response = await api.get('/api/nginx/status')
    if (response.data.code === 0) {
      nginxStatus.value = response.data.data
    }
  } catch (error) {
    console.error('获取Nginx状态失败:', error)
  }
}

async function fetchDashboard() {
  try {
    const response = await api.get('/api/dashboard')
    if (response.data.code === 0) {
      stats.value = response.data.data
    }
  } catch (error) {
    console.error('获取仪表盘数据失败:', error)
  }
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

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

async function toggleNginx() {
  loading.startStop = true
  try {
    if (nginxStatus.value.running) {
      const response = await api.post('/api/nginx/stop')
      if (response.data.code === 0) {
        ElMessage.success('Nginx已停止')
      } else {
        ElMessage.error(response.data.message || '停止失败')
      }
    } else {
      const response = await api.post('/api/nginx/start')
      if (response.data.code === 0) {
        ElMessage.success('Nginx已启动')
      } else {
        ElMessage.error(response.data.message || '启动失败')
      }
    }
    await delay(800)
    await fetchNginxStatus()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
  } finally {
    loading.startStop = false
  }
}

async function restartNginx() {
  loading.restart = true
  try {
    const response = await api.post('/api/nginx/restart')
    if (response.data.code === 0) {
      ElMessage.success('Nginx已重启')
    } else {
      ElMessage.error(response.data.message || '重启失败')
    }
    await delay(800)
    await fetchNginxStatus()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '重启失败')
  } finally {
    loading.restart = false
  }
}

async function reloadConfig() {
  loading.reload = true
  try {
    const response = await api.post('/api/nginx/reload')
    if (response.data.code === 0) {
      ElMessage.success('配置已重载')
    } else {
      ElMessage.error(response.data.message || '重载失败')
    }
    await delay(500)
    await fetchNginxStatus()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '重载失败')
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
        ElMessage.success('配置测试通过')
      } else {
        ElMessage.error('配置测试失败: ' + result.message)
      }
    } else {
      ElMessage.error(response.data.message || '测试失败')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '测试失败')
  } finally {
    loading.test = false
  }
}
</script>

<style scoped>
.status-card {
  margin-bottom: 20px;
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
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-icon.running {
  background: #67c23a22;
  color: #67c23a;
}

.status-icon.stopped {
  background: #f56c6c22;
  color: #f56c6c;
}

.status-info h2 {
  margin: 0 0 4px 0;
  font-size: 18px;
}

.status-meta {
  display: flex;
  gap: 16px;
  color: #909399;
  font-size: 13px;
}

.status-actions {
  display: flex;
  gap: 8px;
}

.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  margin-bottom: 12px;
}

.stat-card :deep(.el-card__body) {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
}

.stat-label {
  font-size: 12px;
  color: #909399;
}

.quick-card {
  margin-bottom: 20px;
}

.quick-btn {
  width: 100%;
  height: 80px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 14px;
}

.system-card {
  margin-bottom: 20px;
}

@media (max-width: 768px) {
  .status-content {
    flex-direction: column;
    gap: 16px;
  }

  .status-actions {
    flex-wrap: wrap;
    justify-content: center;
  }
}
</style>
