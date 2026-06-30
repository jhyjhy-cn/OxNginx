<template>
  <div class="dashboard">
    <el-row :gutter="20">
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" style="background: #409eff">
              <el-icon size="28"><Grid /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ data.site_count }}</div>
              <div class="stat-label">站点数量</div>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" style="background: #67c23a">
              <el-icon size="28"><Lock /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ data.cert_count }}</div>
              <div class="stat-label">SSL证书</div>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" style="background: #e6a23c">
              <el-icon size="28"><Cpu /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ data.cpu_usage.toFixed(1) }}%</div>
              <div class="stat-label">CPU使用率</div>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" style="background: #f56c6c">
              <el-icon size="28"><Coin /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ data.memory_usage.toFixed(1) }}%</div>
              <div class="stat-label">内存使用率</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" style="margin-top: 20px">
      <el-col :span="12">
        <el-card>
          <template #header>
            <span>Nginx 信息</span>
          </template>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="版本">
              {{ data.nginx_version || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="Worker数量">
              {{ data.worker_count }}
            </el-descriptions-item>
            <el-descriptions-item label="活跃连接">
              {{ data.active_connections }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card>
          <template #header>
            <span>快速操作</span>
          </template>
          <div class="quick-actions">
            <el-button type="primary" @click="$router.push('/sites')">
              <el-icon><Plus /></el-icon>
              添加站点
            </el-button>
            <el-button type="success" @click="testNginx">
              <el-icon><Check /></el-icon>
              测试配置
            </el-button>
            <el-button type="warning" @click="reloadNginx">
              <el-icon><Refresh /></el-icon>
              重载Nginx
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import api from '@/api'

const data = ref({
  nginx_version: '',
  worker_count: 0,
  active_connections: 0,
  site_count: 0,
  cert_count: 0,
  cpu_usage: 0,
  memory_usage: 0,
  memory_total: 0,
})

onMounted(() => {
  fetchDashboard()
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

async function testNginx() {
  try {
    const response = await api.post('/api/nginx/test')
    if (response.data.code === 0 && response.data.data.success) {
      ElMessage.success('配置测试通过')
    } else {
      ElMessage.error(response.data.data?.message || '配置测试失败')
    }
  } catch (error) {
    ElMessage.error('测试失败')
  }
}

async function reloadNginx() {
  try {
    const response = await api.post('/api/nginx/reload')
    if (response.data.code === 0) {
      ElMessage.success('Nginx重载成功')
    } else {
      ElMessage.error(response.data.message || '重载失败')
    }
  } catch (error) {
    ElMessage.error('重载失败')
  }
}
</script>

<style scoped>
.stat-card {
  cursor: pointer;
}

.stat-content {
  display: flex;
  align-items: center;
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  margin-right: 16px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.quick-actions {
  display: flex;
  gap: 12px;
}
</style>
