<template>
  <div class="logs">
    <el-card>
      <!-- 搜索栏 -->
      <el-form inline class="search-bar">
        <el-form-item>
          <el-radio-group v-model="logType" @change="fetchLogs">
            <el-radio-button value="access">{{ $t('sys.logs.accessLog') }}</el-radio-button>
            <el-radio-button value="error">{{ $t('sys.logs.errorLog') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchLogs">
            <el-icon><Refresh /></el-icon>
            {{ $t('common.refresh') }}
          </el-button>
        </el-form-item>
        <el-form-item>
          <el-button type="success" @click="downloadLog">
            <el-icon><Download /></el-icon>
            {{ $t('common.download') }}
          </el-button>
        </el-form-item>
      </el-form>

      <!-- 列表 -->
      <el-table :data="tableData" v-loading="loading" :row-style="{ height: '50px' }" max-height="calc(100vh - 300px)">
        <el-table-column prop="line" label="#" width="60" />
        <el-table-column prop="content" :label="$t('sys.logs.title')" min-width="800" show-overflow-tooltip />
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Refresh, Download } from '@element-plus/icons-vue'
import api from '@/api'

const logType = ref('access')
const logs = ref<string[]>([])
const loading = ref(false)

const tableData = computed(() => logs.value.map((content, i) => ({ line: i + 1, content })))

onMounted(fetchLogs)

async function fetchLogs() {
  loading.value = true
  try {
    const response = await api.get(`/api/log/${logType.value}`)
    if (response.data.code === 0) {
      logs.value = response.data.data?.lines || []
    }
  } catch {
    // ignore
  } finally {
    loading.value = false
  }
}

function downloadLog() {
  const blob = new Blob([logs.value.join('\n')], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${logType.value}_log.txt`
  a.click()
  URL.revokeObjectURL(url)
}
</script>

<style scoped>
.search-bar {
  margin-bottom: 12px;
}
</style>
