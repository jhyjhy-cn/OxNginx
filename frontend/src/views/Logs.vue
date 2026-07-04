<template>
  <div class="logs">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('logs.title') }}</span>
          <div>
            <el-radio-group v-model="logType" @change="fetchLogs">
              <el-radio-button label="access">{{ $t('logs.accessLog') }}</el-radio-button>
              <el-radio-button label="error">{{ $t('logs.errorLog') }}</el-radio-button>
            </el-radio-group>
            <el-button style="margin-left: 12px" @click="fetchLogs">
              <el-icon><Refresh /></el-icon>
              {{ $t('common.refresh') }}
            </el-button>
          </div>
        </div>
      </template>

      <div class="log-content" v-loading="loading">
        <pre v-if="logs.length">{{ logs.join('\n') }}</pre>
        <el-empty v-else :description="$t('logs.noLog')" />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import api from '@/api'

const logType = ref('access')
const logs = ref<string[]>([])
const loading = ref(false)

onMounted(() => {
  fetchLogs()
})

async function fetchLogs() {
  loading.value = true
  try {
    const response = await api.get(`/api/log/${logType.value}`)
    if (response.data.code === 0) {
      logs.value = response.data.data?.lines || []
    }
  } catch (error) {
    console.error('获取日志失败:', error)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.log-content {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 4px;
  max-height: 600px;
  overflow-y: auto;
}

.log-content pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
