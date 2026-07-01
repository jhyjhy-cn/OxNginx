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
            v-if="nginxStatus.not_installed"
            type="primary"
            :loading="loading.install"
            @click="installNginx"
          >
            <el-icon v-if="!loading.install"><Download /></el-icon>
            一键安装
          </el-button>
          <template v-else>
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
          <div class="stat-value">{{ stats.site_count }}</div>
          <div class="stat-label">站点数</div>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #67c23a22; color: #67c23a">
          <el-icon :size="24"><Lock /></el-icon>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.cert_count }}</div>
          <div class="stat-label">SSL证书</div>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #e6a23c22; color: #e6a23c">
          <el-icon :size="24"><Cpu /></el-icon>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.cpu_usage.toFixed(1) }}%</div>
          <div class="stat-label">CPU使用率</div>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #f56c6c22; color: #f56c6c">
          <el-icon :size="24"><Coin /></el-icon>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.memory_usage.toFixed(1) }}%</div>
          <div class="stat-label">系统内存</div>
        </div>
      </el-card>
      <el-card class="stat-card" shadow="hover">
        <div class="stat-icon" style="background: #90939922; color: #909399">
          <el-icon :size="24"><Box /></el-icon>
        </div>
        <div class="stat-info">
          <div class="stat-value">{{ stats.app_memory }} MB</div>
          <div class="stat-label">面板内存</div>
        </div>
      </el-card>
    </div>

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
import { ref, reactive, onMounted, onUnmounted } from 'vue'
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

let refreshTimer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  fetchNginxStatus()
  fetchDashboard()
  fetchSystemInfo()
  // 每 10 秒轮询更新状态
  refreshTimer = setInterval(() => {
    fetchNginxStatus()
    fetchDashboard()
  }, 10000)
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})


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

async function installNginx() {
  loading.install = true
  try {
    ElMessage.info('正在安装 Nginx，请稍候...')
    const response = await api.post('/api/nginx/install')
    if (response.data.code === 0) {
      ElMessage.success('Nginx 安装成功')
      await delay(1000)
      await fetchNginxStatus()
      await fetchDashboard()
    } else {
      ElMessage.error(response.data.message || '安装失败')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '安装失败')
  } finally {
    loading.install = false
  }
}
</script>

<style scoped>
.dashboard {
  padding: 20px;
  background: #f5f7fa;
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
  color: #303133;
}

.status-meta {
  display: flex;
  gap: 20px;
  color: #909399;
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
  color: #303133;
  line-height: 1.2;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: #909399;
}

.quick-card {
  margin-bottom: 20px;
  border-radius: 12px;
  border: none;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
}

.quick-card :deep(.el-card__header) {
  padding: 16px 20px;
  border-bottom: 1px solid #ebeef5;
  font-weight: 600;
  color: #303133;
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
  border: 1px solid #e4e7ed;
  background: #fff;
  transition: all 0.3s ease;
  color: #606266;
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
  border-bottom: 1px solid #ebeef5;
  font-weight: 600;
  color: #303133;
}

.system-card :deep(.el-descriptions) {
  border-radius: 8px;
}

.system-card :deep(.el-descriptions__label) {
  font-weight: 500;
  color: #606266;
  background: #fafafa;
}

.system-card :deep(.el-descriptions__content) {
  color: #303133;
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

.stat-card:nth-child(1) { animation-delay: 0.05s; }
.stat-card:nth-child(2) { animation-delay: 0.1s; }
.stat-card:nth-child(3) { animation-delay: 0.15s; }
.stat-card:nth-child(4) { animation-delay: 0.2s; }
.stat-card:nth-child(5) { animation-delay: 0.25s; }
</style>
