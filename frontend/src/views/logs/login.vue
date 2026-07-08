<template>
  <div class="log-page">
    <el-card>
      <div class="search-bar">
        <el-input v-model="params.username" :placeholder="$t('login.username')" clearable style="width: 160px" @keyup.enter="doSearch" />
        <el-input v-model="params.ip" placeholder="IP" clearable style="width: 140px" @keyup.enter="doSearch" />
        <el-select v-model="params.status" :placeholder="$t('common.status')" clearable style="width: 120px">
          <el-option :label="$t('common.success')" value="success" />
          <el-option :label="$t('common.failed')" value="failed" />
        </el-select>
        <el-date-picker
          v-model="dateRange"
          type="daterange"
          :start-placeholder="$t('common.createdAt')"
          :end-placeholder="$t('common.createdAt')"
          value-format="YYYY-MM-DD"
          style="width: 260px"
        />
        <el-button type="primary" @click="doSearch">{{ $t('common.search') }}</el-button>
        <el-button @click="doReset">{{ $t('common.reset') }}</el-button>
        <el-button type="success" @click="doExport">{{ $t('common.download') }}</el-button>
      </div>

      <el-table :data="logs" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column prop="username" :label="$t('login.username')" />
        <el-table-column prop="ip" :label="$t('log.ip')" />
        <el-table-column prop="os" :label="$t('log.os')" />
        <el-table-column prop="browser" :label="$t('log.browser')" />
        <el-table-column :label="$t('log.type')">
          <template #default="{ row }">
            <el-tag :type="row.type === 'login' ? 'primary' : 'info'" size="small">
              {{ row.type === 'login' ? $t('log.login') : $t('log.logout') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.status')">
          <template #default="{ row }">
            <el-tag :type="row.status === 'success' ? 'success' : 'danger'" size="small">
              {{ row.status === 'success' ? $t('log.loginSuccess') : $t('log.loginFailed') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.createdAt')">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
      </el-table>

      <OnPagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :total="total"
        @change="load"
      />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'

dayjs.extend(utc)
import api from '@/api'
import OnPagination from '@/components/OnPagination/index.vue'

interface LoginLog {
  id: number
  username: string
  ip: string | null
  os: string | null
  browser: string | null
  type: string
  status: string
  created_at: string | null
}

const logs = ref<LoginLog[]>([])
const loading = ref(false)
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)
const dateRange = ref<[string, string] | null>(null)

const params = ref({ username: '', ip: '', status: '' })

function formatTime(t: string | null): string {
  if (!t) return ''
  return dayjs.utc(t).local().format('YYYY-MM-DD HH:mm:ss')
}

function buildParams() {
  const p: Record<string, string | number> = {
    page: currentPage.value,
    page_size: pageSize.value,
  }
  if (params.value.username) p.username = params.value.username
  if (params.value.ip) p.ip = params.value.ip
  if (params.value.status) p.status = params.value.status
  if (dateRange.value) {
    p.start_time = dateRange.value[0] + ' 00:00:00'
    p.end_time = dateRange.value[1] + ' 23:59:59'
  }
  return p
}

function doSearch() {
  currentPage.value = 1
  load()
}

function doReset() {
  params.value = { username: '', ip: '', status: '' }
  dateRange.value = null
  currentPage.value = 1
  load()
}

function doExport() {
  const p = buildParams()
  const query = Object.entries(p)
    .filter(([, v]) => v !== '' && v !== undefined)
    .map(([k, v]) => `${k}=${encodeURIComponent(v)}`)
    .join('&')
  window.open(`/api/log/login/export?${query}`, '_blank')
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/log/login', { params: buildParams() })
    if (data.code === 0) {
      logs.value = data.data.list
      total.value = data.data.total
    }
  } catch {
    // 后端接口未实现时静默处理
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap; }
</style>
