<template>
  <div class="log-page h-full">
    <el-card class="h-full">
      <!-- ponytail: daterange/module 无 OnFormGrid 字段类型，搜索栏保留手写，绑定 useCrud.searchForm -->
      <div class="search-bar">
        <el-input v-model="searchForm.username" :placeholder="$t('login.username')" clearable style="width: 140px" @keyup.enter="doSearch" />
        <el-select v-model="searchForm.module" placeholder="操作模块" clearable style="width: 130px">
          <el-option v-for="m in MODULE_OPTIONS" :key="m.value" :label="m.label" :value="m.value" />
        </el-select>
        <el-input v-model="searchForm.trace_id" placeholder="TraceID" clearable style="width: 220px" @keyup.enter="doSearch" />
        <el-select v-model="searchForm.status" :placeholder="$t('common.status')" clearable style="width: 110px">
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

      <OnTable
        :data="dataList"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto' }"
        @page-change="onPageChange"
        @command="handleCommand"
        @reload="load"
      >
        <template #module="{ row }">{{ moduleLabel(row.module) }}</template>
        <template #status="{ row }">
          <el-tag :type="row.status === 'success' ? 'success' : 'danger'" size="small">
            {{ row.status === 'success' ? '成功' : '失败' }}
          </el-tag>
        </template>
        <template #created="{ row }">{{ formatTime(row.created_at) }}</template>
        <template #cost="{ row }">{{ durationMs(row) }}</template>
      </OnTable>
    </el-card>

    <!-- 详情弹窗 -->
    <OnDialog v-model="showDialog" title="操作详情" width="700px">
      <el-descriptions :column="1" border size="small">
        <el-descriptions-item label="操作模块">{{ moduleLabel(detail?.module) }}</el-descriptions-item>
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
        <el-descriptions-item label="消耗时间">{{ durationMs(detail) }}</el-descriptions-item>
        <el-descriptions-item v-if="detail?.trace_id" label="TraceID">
          <span style="font-family: monospace; font-size: 12px">{{ detail.trace_id }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="请求参数">
          <pre class="detail-pre">{{ formatJson(detail?.request_body ?? null) }}</pre>
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
import type { TableColumn } from '@/components/OnTable/types'
import OnTable from '@/components/OnTable/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import { useCrud } from '@/hooks'
import { listOperationLogs } from '@/api/logs'

dayjs.extend(utc)

// ponytail: module 英文 key → 中文显示。前端做 i18n 翻译
const MODULE_OPTIONS = [
  { value: 'site', label: '站点管理' },
  { value: 'rbac', label: '权限管理' },
  { value: 'nginx', label: 'Nginx' },
  { value: 'file', label: '文件管理' },
  { value: 'config', label: '配置管理' },
  { value: 'access', label: '访问控制' },
  { value: 'backup', label: '备份管理' },
  { value: 'template', label: '模板管理' },
  { value: 'upstream', label: '上游服务' },
  { value: 'proxy', label: '反向代理' },
  { value: 'system', label: '系统设置' },
]
const MODULE_MAP: Record<string, string> = Object.fromEntries(MODULE_OPTIONS.map(m => [m.value, m.label]))
function moduleLabel(key: string | null | undefined): string {
  if (!key) return ''
  return MODULE_MAP[key] || key
}

interface OpLog {
  id: number
  trace_id: string | null
  username: string
  module: string | null
  action: string
  method: string | null
  uri: string | null
  ip: string | null
  status: string
  cost_ms: number | null
  duration_ms: number | null
  request_body: string | null
  response_body: string | null
  error_msg: string | null
  created_at: string | null
}

function durationMs(row: OpLog | null | undefined): string {
  if (!row) return '-'
  const ms = row.duration_ms ?? row.cost_ms
  return ms != null ? ms + 'ms' : '-'
}

const dateRange = ref<[string, string] | null>(null)
const showDialog = ref(false)
const detail = ref<OpLog | null>(null)

const {
  loading,
  dataList,
  total,
  page,
  pageSize,
  searchForm,
  load,
  search,
  reset,
} = useCrud({
  getListApi: listOperationLogs,
  isPage: true,
  pageSize: 20,
  searchForm: { username: '', status: '', module: '', trace_id: '', start_time: '', end_time: '' },
})

const tableColumns: TableColumn[] = [
  { prop: 'module', label: '操作模块', minWidth: 100, showOverflowTooltip: true, slot: 'module' },
  { prop: 'action', label: '操作类型', minWidth: 180, showOverflowTooltip: true },
  { prop: 'method', label: '请求方式', width: 80 },
  { prop: 'username', label: 'login.username', width: 100 },
  { prop: 'uri', label: '操作地址', minWidth: 200, showOverflowTooltip: true },
  { prop: 'status', label: '操作状态', width: 80, slot: 'status' },
  { prop: 'created_at', label: '操作日期', width: 170, slot: 'created' },
  { prop: 'cost', label: '耗时', width: 80, slot: 'cost' },
  {
    label: '详情',
    width: 60,
    fixed: 'right',
    buttons: [{ name: { zh: '查看', en: 'View' }, command: 'detail', size: 'small' }],
  },
]

function formatTime(t: string | null): string {
  if (!t) return ''
  return dayjs.utc(t).local().format('YYYY-MM-DD HH:mm:ss')
}

function formatJson(s: string | null): string {
  if (!s) return '-'
  try { return JSON.stringify(JSON.parse(s), null, 2) } catch { return s }
}

function syncDates() {
  if (dateRange.value) {
    searchForm.start_time = dateRange.value[0] + ' 00:00:00'
    searchForm.end_time = dateRange.value[1] + ' 23:59:59'
  } else {
    searchForm.start_time = ''
    searchForm.end_time = ''
  }
}

function doSearch() { syncDates(); search() }
function doReset() { dateRange.value = null; reset() }

function onPageChange(p: number) { page.value = p; load() }

function handleCommand(command: string | number, row: OpLog) {
  if (command === 'detail') { detail.value = row; showDialog.value = true }
}

function doExport() {
  syncDates()
  const query = Object.entries({ ...searchForm })
    .filter(([, v]) => v !== '' && v !== undefined && v !== null)
    .map(([k, v]) => `${k}=${encodeURIComponent(v as string)}`)
    .join('&')
  window.open(`/api/log/operation/export?${query}`, '_blank')
}

onMounted(load)
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap; }
.detail-pre { margin: 0; font-size: 12px; white-space: pre-wrap; word-break: break-all; max-height: 200px; overflow-y: auto; }
</style>
