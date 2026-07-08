<template>
  <div class="log-page">
    <el-card>
      <div class="toolbar">
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>
      <el-table :data="logs" v-loading="loading" max-height="calc(100vh - 280px)">
        <el-table-column prop="created_at" :label="$t('common.createdAt')" width="180" />
        <el-table-column prop="username" :label="$t('login.username')" width="120" />
        <el-table-column prop="action" label="操作" min-width="200" />
        <el-table-column prop="target" label="目标" min-width="200" />
        <el-table-column prop="ip" label="IP" width="140" />
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
  action: string
  target: string
  ip: string
}

const logs = ref<LogRow[]>([])
const loading = ref(false)

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/log/operation')
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
