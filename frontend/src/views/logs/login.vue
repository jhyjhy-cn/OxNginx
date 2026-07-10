<template>
  <div class="log-page h-full">
    <el-card class="h-full">
      <!-- ponytail: daterange 无 OnFormGrid 字段类型，搜索栏保留手写，绑定 useCrud.searchForm -->
      <div class="search-bar">
        <el-input v-model="searchForm.username" :placeholder="$t('login.username')" clearable style="width: 160px" @keyup.enter="doSearch" />
        <el-input v-model="searchForm.ip" placeholder="IP" clearable style="width: 140px" @keyup.enter="doSearch" />
        <el-select v-model="searchForm.status" :placeholder="$t('common.status')" clearable style="width: 120px">
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

      <OnTable
        :data="dataList"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto' }"
        @page-change="onPageChange"
        @reload="load"
      >
        <template #type="{ row }">
          <el-tag :type="row.type === 1 ? 'primary' : 'info'" size="small">
            {{ row.type === 1 ? $t('sys.log.login') : $t('sys.log.logout') }}
          </el-tag>
        </template>
        <template #status="{ row }">
          <el-tag :type="row.status === 'success' ? 'success' : 'danger'" size="small">
            {{ row.status === 'success' ? $t('sys.log.loginSuccess') : $t('sys.log.loginFailed') }}
          </el-tag>
        </template>
        <template #created="{ row }">{{ formatTime(row.created_at) }}</template>
      </OnTable>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import dayjs from 'dayjs'
import utc from 'dayjs/plugin/utc'
import type { TableColumn } from '@/components/OnTable/types'
import OnTable from '@/components/OnTable/index.vue'
import { useCrud } from '@/hooks'
import { listLoginLogs } from '@/api/logs'

dayjs.extend(utc)

const dateRange = ref<[string, string] | null>(null)

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
  getListApi: listLoginLogs,
  isPage: true,
  pageSize: 20,
  searchForm: { username: '', ip: '', status: '', start_time: '', end_time: '' },
})

const tableColumns: TableColumn[] = [
  { prop: 'username', label: 'login.username' },
  { prop: 'ip', label: 'sys.log.ip' },
  { prop: 'os', label: 'sys.log.os' },
  { prop: 'browser', label: 'sys.log.browser' },
  { prop: 'type', label: 'sys.log.type', slot: 'type' },
  { prop: 'status', label: 'common.status', slot: 'status' },
  { prop: 'created_at', label: 'common.createdAt', slot: 'created' },
]

function formatTime(t: string | null): string {
  if (!t) return ''
  return dayjs.utc(t).local().format('YYYY-MM-DD HH:mm:ss')
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

function doExport() {
  syncDates()
  const query = Object.entries({ ...searchForm })
    .filter(([, v]) => v !== '' && v !== undefined && v !== null)
    .map(([k, v]) => `${k}=${encodeURIComponent(v as string)}`)
    .join('&')
  window.open(`/api/log/login/export?${query}`, '_blank')
}

onMounted(load)
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap; }
</style>
