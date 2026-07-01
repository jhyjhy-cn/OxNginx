<template>
  <div class="dashboard">
    <!-- 标题区 -->
    <div class="dashboard-header">
      <div class="header-left">
        <h1 class="dashboard-title">仪表盘</h1>
        <p class="dashboard-subtitle">实时监控系统状态</p>
      </div>
      <div class="header-right">
        <div class="time-display">{{ currentTime }}</div>
      </div>
    </div>

    <!-- 统计卡片区 -->
    <div class="stats-grid">
      <!-- 站点数量 -->
      <div class="stat-card" @click="$router.push('/sites')">
        <div class="stat-card-inner">
          <div class="stat-icon-wrapper blue">
            <el-icon size="24"><Grid /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ data.site_count }}</div>
            <div class="stat-label">站点数量</div>
          </div>
          <div class="stat-trend up">
            <el-icon><ArrowUp /></el-icon>
          </div>
        </div>
      </div>

      <!-- SSL证书 -->
      <div class="stat-card">
        <div class="stat-card-inner">
          <div class="stat-icon-wrapper green">
            <el-icon size="24"><Lock /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ data.cert_count }}</div>
            <div class="stat-label">SSL证书</div>
          </div>
          <div class="stat-badge safe">
            <el-icon><Check /></el-icon>
            安全
          </div>
        </div>
      </div>

      <!-- CPU使用率 -->
      <div class="stat-card">
        <div class="stat-card-inner">
          <div class="stat-icon-wrapper orange">
            <el-icon size="24"><Cpu /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ data.cpu_usage.toFixed(1) }}<span class="unit">%</span></div>
            <div class="stat-label">CPU使用率</div>
            <div class="progress-bar">
              <div class="progress-fill orange" :style="{ width: data.cpu_usage + '%' }"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- 内存使用率 -->
      <div class="stat-card">
        <div class="stat-card-inner">
          <div class="stat-icon-wrapper red">
            <el-icon size="24"><Coin /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ data.memory_usage.toFixed(1) }}<span class="unit">%</span></div>
            <div class="stat-label">内存使用率</div>
            <div class="progress-bar">
              <div class="progress-fill red" :style="{ width: data.memory_usage + '%' }"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- 程序内存 -->
      <div class="stat-card">
        <div class="stat-card-inner">
          <div class="stat-icon-wrapper purple">
            <el-icon size="24"><Monitor /></el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ data.app_memory }}<span class="unit">MB</span></div>
            <div class="stat-label">程序内存</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <!-- Nginx状态卡片 -->
      <div class="content-card nginx-card">
        <div class="card-header">
          <div class="card-title">
            <el-icon size="20"><Cpu /></el-icon>
            <span>Nginx 状态</span>
          </div>
          <div v-if="!nginxStatus.notInstalled" class="status-indicator" :class="nginxStatus.running ? 'running' : 'stopped'">
            <span class="status-dot"></span>
            {{ nginxStatus.running ? '运行中' : '已停止' }}
          </div>
          <div v-else class="status-indicator warning">
            <span class="status-dot"></span>
            未安装
          </div>
        </div>

        <div v-if="nginxStatus.notInstalled" class="not-installed-tip">
          <el-icon size="48" color="#94a3b8"><Warning /></el-icon>
          <p>Nginx 尚未安装</p>
          <el-button type="primary" @click="installNginx">
            <el-icon><Download /></el-icon>
            一键安装 Nginx
          </el-button>
        </div>

        <div v-else class="nginx-info-grid">
          <div class="info-item">
            <span class="info-label">版本</span>
            <span class="info-value">{{ nginxStatus.version || data.nginx_version || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">PID</span>
            <span class="info-value highlight">{{ nginxStatus.pid || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">运行时间</span>
            <span class="info-value">{{ nginxStatus.uptime || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">活跃连接</span>
            <span class="info-value highlight">{{ data.active_connections }}</span>
          </div>
        </div>
      </div>

      <!-- 快速操作卡片 -->
      <div class="content-card actions-card">
        <div class="card-header">
          <div class="card-title">
            <el-icon size="20"><Setting /></el-icon>
            <span>快速操作</span>
          </div>
        </div>
        <div class="actions-grid">
          <el-button type="primary" class="action-btn" @click="$router.push('/sites')">
            <el-icon><Plus /></el-icon>
            添加站点
          </el-button>
          <el-button type="success" class="action-btn" :loading="loading.test" @click="testNginx">
            <el-icon><Check /></el-icon>
            测试配置
          </el-button>
          <el-button type="warning" class="action-btn" :loading="loading.reload" @click="reloadNginx">
            <el-icon><Refresh /></el-icon>
            重载配置
          </el-button>
          <el-button :type="nginxStatus.running ? 'danger' : 'success'" class="action-btn" :loading="loading.startStop" @click="nginxStatus.running ? stopNginx() : startNginx()">
            <el-icon><VideoPlay /></el-icon>
            {{ nginxStatus.running ? '停止' : '启动' }}
          </el-button>
          <el-button type="info" class="action-btn" :loading="loading.restart" @click="restartNginx">
            <el-icon><RefreshRight /></el-icon>
            重启
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import api from '@/api'

const currentTime = ref('')

const data = ref({
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

const nginxStatus = reactive({
  running: false,
  pid: null as number | null,
  version: '',
  uptime: '',
  notInstalled: false,
})

const loading = reactive({
  test: false,
  reload: false,
  startStop: false,
  restart: false,
})

let refreshTimer: number | null = null
let timeTimer: number | null = null

function updateTime() {
  const now = new Date()
  currentTime.value = now.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
}

onMounted(() => {
  updateTime()
  fetchDashboard()
  fetchNginxStatus()
  refreshTimer = window.setInterval(() => {
    fetchDashboard()
  }, 1000)
  timeTimer = window.setInterval(updateTime, 1000)
})

onUnmounted(() => {
  if (refreshTimer !== null) clearInterval(refreshTimer)
  if (timeTimer !== null) clearInterval(timeTimer)
})

async function fetchDashboard() {
  try {
    const response = await api.get('/api/dashboard')
    if (response.data.code === 0) {
      data.value = response.data.data
    }
  } catch (error) {
    console.error('获取Dashboard数据失败:', error)
  }
}

async function fetchNginxStatus() {
  try {
    const response = await api.get('/api/nginx/status')
    if (response.data.code === 0) {
      // 后端返回 not_installed，前端用 notInstalled
      const status = response.data.data
      nginxStatus.running = status.running
      nginxStatus.pid = status.pid
      nginxStatus.version = status.version || ''
      nginxStatus.uptime = status.uptime || ''
      nginxStatus.notInstalled = status.not_installed || false
      console.log('[NginxStatus]', JSON.stringify(status))
    }
  } catch (error) {
    console.error('获取Nginx状态失败:', error)
  }
}

async function testNginx() {
  loading.test = true
  try {
    const response = await api.post('/api/nginx/test')
    if (response.data.code === 0 && response.data.data.success) {
      ElMessage.success('配置测试通过')
    } else {
      ElMessage.error(response.data.data?.message || '配置测试失败')
    }
  } catch (error) {
    ElMessage.error('测试失败')
  } finally {
    loading.test = false
  }
}

async function reloadNginx() {
  loading.reload = true
  try {
    const response = await api.post('/api/nginx/reload')
    if (response.data.code === 0) {
      ElMessage.success('Nginx重载成功')
    } else {
      ElMessage.error(response.data.message || '重载失败')
    }
  } catch (error) {
    ElMessage.error('重载失败')
  } finally {
    loading.reload = false
  }
}

async function startNginx() {
  loading.startStop = true
  try {
    const response = await api.post('/api/nginx/start')
    if (response.data.code === 0) {
      ElMessage.success('Nginx启动成功')
      await delay(500)
      await fetchNginxStatus()
    } else {
      ElMessage.error(response.data.message || '启动失败')
    }
  } catch (error) {
    ElMessage.error('启动失败')
  } finally {
    loading.startStop = false
  }
}

async function stopNginx() {
  loading.startStop = true
  try {
    const response = await api.post('/api/nginx/stop')
    if (response.data.code === 0) {
      ElMessage.success('Nginx已停止')
      await delay(500)
      await fetchNginxStatus()
    } else {
      ElMessage.error(response.data.message || '停止失败')
    }
  } catch (error) {
    ElMessage.error('停止失败')
  } finally {
    loading.startStop = false
  }
}

async function restartNginx() {
  loading.restart = true
  try {
    const response = await api.post('/api/nginx/restart')
    if (response.data.code === 0) {
      ElMessage.success('Nginx重启成功')
      await delay(1000)
      await fetchNginxStatus()
    } else {
      ElMessage.error(response.data.message || '重启失败')
    }
  } catch (error) {
    ElMessage.error('重启失败')
  } finally {
    loading.restart = false
  }
}

async function installNginx() {
  try {
    ElMessage.info('正在安装 Nginx，请稍候...')
    const response = await api.post('/api/nginx/install')
    if (response.data.code === 0) {
      ElMessage.success('Nginx 安装成功')
      await fetchNginxStatus()
    } else {
      ElMessage.error(response.data.message || '安装失败')
    }
  } catch (error) {
    ElMessage.error('安装失败')
  }
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}
</script>

<style scoped>
.dashboard {
  padding: 24px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e8ec 100%);
  min-height: calc(100vh - 60px);
}

/* 标题区 */
.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.dashboard-title {
  font-size: 28px;
  font-weight: 600;
  color: #1a1a2e;
  margin: 0;
}

.dashboard-subtitle {
  font-size: 14px;
  color: #64748b;
  margin: 4px 0 0 0;
}

.time-display {
  font-size: 14px;
  color: #64748b;
  background: white;
  padding: 8px 16px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

/* 统计卡片网格 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  background: white;
  border-radius: 16px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.04);
  border: 1px solid rgba(0, 0, 0, 0.04);
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
}

.stat-card-inner {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.stat-icon-wrapper {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.stat-icon-wrapper.blue { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }
.stat-icon-wrapper.green { background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%); }
.stat-icon-wrapper.orange { background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%); }
.stat-icon-wrapper.red { background: linear-gradient(135deg, #ff0844 0%, #ffb199 100%); }
.stat-icon-wrapper.purple { background: linear-gradient(135deg, #a18cd1 0%, #fbc2eb 100%); }

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 26px;
  font-weight: 700;
  color: #1a1a2e;
  line-height: 1.2;
}

.stat-value .unit {
  font-size: 14px;
  font-weight: 500;
  color: #64748b;
}

.stat-label {
  font-size: 13px;
  color: #94a3b8;
  margin-top: 4px;
}

.progress-bar {
  height: 4px;
  background: #e2e8f0;
  border-radius: 2px;
  margin-top: 10px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}

.progress-fill.orange { background: linear-gradient(90deg, #f093fb 0%, #f5576c 100%); }
.progress-fill.red { background: linear-gradient(90deg, #ff0844 0%, #ffb199 100%); }

.stat-trend {
  display: flex;
  align-items: center;
  gap: 2px;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 6px;
}

.stat-trend.up {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.stat-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 6px;
}

.stat-badge.safe {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

/* 主内容区 */
.main-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.content-card {
  background: white;
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.04);
  border: 1px solid rgba(0, 0, 0, 0.04);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f1f5f9;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
  color: #1a1a2e;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 6px 12px;
  border-radius: 20px;
}

.status-indicator.running {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.status-indicator.stopped {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.status-indicator.warning {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
}

.not-installed-tip {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px 0;
  gap: 12px;
}

.not-installed-tip p {
  color: #64748b;
  margin: 0;
  font-size: 14px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-indicator.running .status-dot {
  background: #10b981;
}

.status-indicator.stopped .status-dot {
  background: #ef4444;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.nginx-info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 12px;
  color: #94a3b8;
}

.info-value {
  font-size: 15px;
  color: #1a1a2e;
  font-weight: 500;
}

.info-value.highlight {
  color: #667eea;
}

/* 操作按钮 */
.actions-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.action-btn {
  border-radius: 10px;
  height: 42px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.action-btn:hover {
  transform: translateY(-2px);
}

/* 响应式 */
@media (max-width: 1400px) {
  .stats-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 1024px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  .main-content {
    grid-template-columns: 1fr;
  }
  .actions-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 640px) {
  .stats-grid {
    grid-template-columns: 1fr;
  }
  .dashboard {
    padding: 16px;
  }
  .actions-grid {
    grid-template-columns: 1fr;
  }
}
</style>
