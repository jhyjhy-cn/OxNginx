<template>
  <div class="online-page h-full">
    <el-card class="h-full">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <OnFormGrid :model="searchForm" :fields="searchFields" style="flex: 1" />
        <el-button type="primary" @click="search">{{ $t('common.search') }}</el-button>
        <el-button @click="reset">{{ $t('common.reset') }}</el-button>
      </div>

      <OnTable
        :data="dataList"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto', rowKey: 'id' }"
        @page-change="onPageChange"
        @command="handleCommand"
        @reload="load"
      >
        <template #token="{ row }">
          <el-tooltip :content="row.token || ''" placement="top">
            <span class="token-prefix">{{ (row.token || '').slice(0, 8) }}…</span>
          </el-tooltip>
        </template>
      </OnTable>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, computed } from 'vue'
import OnFormGrid from '@/components/OnForm/OnFormGrid/index.vue'
import OnTable from '@/components/OnTable/index.vue'
import type { FormField } from '@/components/OnForm/types'
import type { TableColumn } from '@/components/OnTable/types'
import { listOnline, kickOnline } from '@/api/sys/online'
import type { OnlineItem } from '@/api/sys/online'
import { useAuthStore } from '@/stores/auth'
import { useCrud, useMessage } from '@/hooks'

const { success, error, confirm } = useMessage()
const authStore = useAuthStore()
const canKick = computed(() => authStore.hasPermission('sys:online:kick'))

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
  getListApi: listOnline,
  isPage: true,
  pageSize: 20,
})

// 关键字同时匹配 username 和 ip
const searchFields: FormField[] = [
  { prop: 'keyword', label: 'sys.online.searchKeyword', type: 'input', span: 8 },
]

const tableColumns: TableColumn[] = [
  { prop: 'token', label: 'sys.online.colToken', minWidth: 120, slot: 'token' },
  { prop: 'username', label: 'sys.online.colUsername', minWidth: 120 },
  { prop: 'dept_name', label: 'sys.online.colDept', minWidth: 120 },
  { prop: 'ip', label: 'sys.online.colIp', minWidth: 140 },
  { prop: 'os', label: 'sys.online.colOs', minWidth: 120 },
  { prop: 'browser', label: 'sys.online.colBrowser', minWidth: 140 },
  { prop: 'created_at', label: 'sys.online.colLoginAt', minWidth: 170 },
  {
    label: 'common.action',
    width: 100,
    fixed: 'right',
    buttons: [
      {
        name: 'sys.online.kick',
        command: 'kick',
        type: 'danger',
        size: 'small',
        disabled: () => !canKick.value,
      },
    ],
  },
]

function onPageChange(p: number) {
  page.value = p
  load()
}

async function handleCommand(command: string | number, row: OnlineItem) {
  if (command === 'kick') await doKick(row)
}

async function doKick(row: OnlineItem) {
  const ok = await confirm({ message: 'sys.online.confirmKick' })
  if (!ok) return
  try {
    await kickOnline(row.id)
    success('common.success')
    load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

// ponytail: 30s 自动刷新一次会话列表，无需手动
let timer: number | undefined
onMounted(() => {
  load()
  timer = window.setInterval(() => {
    if (page.value === 1) load()
  }, 30_000)
})
onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer)
})
</script>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 12px;
}
.token-prefix {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  color: var(--el-text-color-secondary);
}
</style>