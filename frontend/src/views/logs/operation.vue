<template>
  <div class="log-page">
    <el-card>
      <div class="search-bar">
        <el-button type="primary" @click="doSearch">{{ $t('common.search') }}</el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table :data="logs" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column :label="$t('common.createdAt')" width="170">
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column prop="action" label="操作" min-width="200" show-overflow-tooltip />
        <el-table-column prop="method" label="方式" width="80" />
        <el-table-column prop="uri" label="地址" min-width="200" show-overflow-tooltip />
        <el-table-column prop="username" :label="$t('login.username')" width="100" />
        <el-table-column label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.status === 'success' ? 'success' : 'danger'" size="small">
              {{ row.status === 'success' ? '成功' : '失败' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="耗时" width="80">
          <template #default="{ row }">{{ row.cost_ms != null ? row.cost_ms + 'ms' : '' }}</template>
        </el-table-column>
        <el-table-column label="详情" width="60" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" text size="small" @click="showDetail(row)">查看</el-button>
          </template>
        </el-table-column>
      </el-table>

      <OnPagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :total="total"
        @change="load"
      />
    </el-card>

    <!-- 详情弹窗 -->
    <OnDialog v-model="showDialog" title="操作详情" width="700px">
      <el-descriptions :column="1" border size="small">
        <el-descriptions-item label="操作">{{ detail?.action }}</el-descriptions-item>
        <el-descriptions-item label="方式">{{ detail?.method }}</el-descriptions-item>
        <el-descriptions-item label="地址">{{ detail?.uri }}</el-descriptions-item>
        <el-descriptions-item label="操作人">{{ detail?.username }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="detail?.status === 'success' ? 'success' : 'danger'" size="small">
            {{ detail?.status === 'success' ? '成功' : '失败' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="耗时">{{ detail?.cost_ms != null ? detail.cost_ms + 'ms' : '-' }}</el-descriptions-item>
        <el-descriptions-item label="请求参数">
          <pre class="detail-pre">{{ detail?.request_body || '-' }}</pre>
        </el-descriptions-item>
        <el-descriptions-item label="返回参数">
          <pre class="detail-pre">{{ detail?.response_body || '-' }}</pre>
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
const showDialog = ref(false)
const detail = ref<OpLog | null>(null)

function formatTime(t: string | null): string {
  if (!t) return ''
  return dayjs.utc(t).local().format('YYYY-MM-DD HH:mm:ss')
}

function showDetail(row: OpLog) {
  detail.value = row
  showDialog.value = true
}

function doSearch() {
  currentPage.value = 1
  load()
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/log/operation', { params: { page: currentPage.value, page_size: pageSize.value } })
    if (data.code === 0) {
      logs.value = data.data?.list || data.data || []
      total.value = data.data?.total || logs.value.length
    }
  } catch {
    // ignore
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }
.detail-pre { margin: 0; font-size: 12px; white-space: pre-wrap; word-break: break-all; max-height: 200px; overflow-y: auto; }
</style>
