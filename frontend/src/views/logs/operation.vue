<template>
  <div class="log-page">
    <el-card>
      <div class="search-bar">
        <el-input v-model="params.username" :placeholder="$t('login.username')" clearable style="width: 140px" @keyup.enter="doSearch" />
        <el-select v-model="params.status" :placeholder="$t('common.status')" clearable style="width: 110px">
          <el-option label="成功" value="success" />
          <el-option label="失败" value="failed" />
        </el-select>
        <el-date-picker
          v-model="dateRange"
          type="daterange"
          start-placeholder="开始日期"
          end-placeholder="结束日期"
          value-format="YYYY-MM-DD"
          style="width: 260px"
        />
        <el-button type="primary" @click="doSearch">{{ $t('common.search') }}</el-button>
        <el-button @click="doReset">{{ $t('common.reset') }}</el-button>
        <el-button type="success" @click="doExport">{{ $t('common.download') }}</el-button>
      </div>

      <el-table :data="logs" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column prop="action" label="操作类型" min-width="180" show-overflow-tooltip />
        <el-table-column prop="method" label="请求方式" width="80" />
        <el-table-column prop="username" :label="$t('login.username')" width="100" />
        <el-table-column prop="uri" label="操作地址" min-width="200" show-overflow-tooltip />
        <el-table-column label="操作状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.status === 'success' ? 'success' : 'danger'" size="small">
              {{ row.status === 'success' ? '成功' : '失败' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作日期" width="170">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="耗时" width="70">
          <template #default="{ row }">{{ row.cost_ms != null ? row.cost_ms + 'ms' : '' }}</template>
        </el-table-column>
        <el-table-column label="详情" width="60" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" text size="small" @click="showDetail(row)">查看</el-button>
          </template>
        </el-table-column>
      </el-table>

      <OnPagination v-model:current-page="currentPage" v-model:page-size="pageSize" :total="total" @change="load" />
    </el-card>

    <!-- 详情弹窗 -->
    <OnDialog v-model="showDialog" title="操作详情" width="700px">
      <el-descriptions :column="1" border size="small">
        <el-descriptions-item label="操作类型">{{ detail?.action }}</el-descriptions-item>
        <el-descriptions-item label="请求方式">{{ detail?.method }}</el-descriptions-item>
        <el-descriptions-item label="操作人员">{{ detail?.username }}</el-descriptions-item>
        <el-descriptions-item label="操作地址">{{ detail?.uri }}</el-descriptions-item>
        <el-descriptions-item label="操作状态">
          <el-tag :type="detail?.status === 'success' ? 'success' : 'danger'" size="small">
            {{ detail?.status === 'success' ? '成功' : '失败' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="操作日期">{{ formatTime(detail?.created_at ?? null) }}</el-descriptions-item>
        <el-descriptions-item label="消耗时间">{{ detail?.cost_ms != null ? detail.cost_ms + 'ms' : '-' }}</el-descriptions-item>
        <el-descriptions-item label="请求参数">
          <pre class="detail-pre">{{ formatJson(detail?.request_body ?? null) }}</pre>
        </el-descriptions-item>
        <el-descriptions-item label="返回参数">
          <pre class="detail-pre">{{ formatJson(detail?.response_body ?? null) }}</pre>
        </el-descriptions-item>
        <el-descriptions-item v-if="detail?.error_msg" label="错误信息">
          <span style="color: var(--el-color-danger)">{{ detail.error_msg }}</span>
        </el-descriptions-item>
      </el-descriptions>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import api from '@/api'
import OnPagination from '@/components/OnPagination/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'

dayjs.extend(utc)

interface OpLog {
  id: number
  username: string
  action: string
  method: string | null
  uri: string | null
  ip: string | null
  status: string
  cost_ms: number | null
  request_body: string | null
  response_body: string | null
  error_msg: string | null
  created_at: string | null
}

const logs = ref<OpLog[]>([])
const loading = ref(false)
const total = ref(0)
const currentPage = ref(1)
const pageSize = ref(20)
const dateRange = ref<[string, string] | null>(null)
const showDialog = ref(false)
const detail = ref<OpLog | null>(null)
const params = ref({ username: '', status: '' })

function formatTime(t: string | null): string {
  if (!t) return ''
  return dayjs.utc(t).local().format('YYYY-MM-DD HH:mm:ss')
}

function formatJson(s: string | null): string {
  if (!s) return '-'
  try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s }
}

function buildParams() {
  const p: Record<string, string | number> = { page: currentPage.value, page_size: pageSize.value }
  if (params.value.username) p.username = params.value.username
  if (params.value.status) p.status = params.value.status
  if (dateRange.value) {
    p.start_time = dateRange.value[0] + ' 00:00:00'
    p.end_time = dateRange.value[1] + ' 23:59:59'
  }
  return p
}

function doSearch() { currentPage.value = 1; load() }
function doReset() { params.value = { username: '', status: '' }; dateRange.value = null; currentPage.value = 1; load() }
function showDetail(row: OpLog) { detail.value = row; showDialog.value = true }

function doExport() {
  const p = buildParams()
  const query = Object.entries(p).filter(([, v]) => v !== '' && v !== undefined).map(([k, v]) => `${k}=${encodeURIComponent(v)}`).join('&')
  window.open(`/api/log/operation/export?${query}`, '_blank')
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/log/operation', { params: buildParams() })
    if (data.code === 0) { logs.value = data.data.list; total.value = data.data.total }
  } catch { /* ignore */ } finally { loading.value = false }
}
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap; }
.detail-pre { margin: 0; font-size: 12px; white-space: pre-wrap; word-break: break-all; max-height: 200px; overflow-y: auto; }
</style>
