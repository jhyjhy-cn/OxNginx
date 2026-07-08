<template>
  <div class="log-page">
    <el-card>
      <div class="toolbar">
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>
      <el-table :data="logs" v-loading="loading" max-height="calc(100vh - 280px)">
        <el-table-column prop="created_at" :label="$t('common.createdAt')" width="180" />
        <el-table-column prop="username" :label="$t('login.username')" width="120" />
        <el-table-column prop="ip" label="IP" width="140" />
        <el-table-column prop="user_agent" label="User-Agent" min-width="300" />
        <el-table-column prop="status" :label="$t('common.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'success' ? 'success' : 'danger'" size="small">
              {{ row.status === 'success' ? $t('common.success') : $t('common.failed') }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-if="!loading && !logs.length" :description="$t('logs.noLog')" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import api from '@/api'

interface LogRow {
  id: number
  created_at: string
  username: string
  ip: string
  user_agent: string
  status: string
}

const logs = ref<LogRow[]>([])
const loading = ref(false)

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/log/login')
    if (data.code === 0) logs.value = data.data?.list || data.data || []
  } catch {
    // 后端接口未实现时静默处理
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.toolbar { display: flex; gap: 12px; margin-bottom: 12px; }
</style>
