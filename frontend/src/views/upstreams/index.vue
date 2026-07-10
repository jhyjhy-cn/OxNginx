<template>
  <div class="upstreams">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('sys.upstreams.title') }}</span>
          <el-button type="primary" @click="showAddDialog">
            <el-icon><Plus /></el-icon>
            {{ $t('sys.upstreams.addUpstream') }}
          </el-button>
        </div>
      </template>

      <OnTable
        :data="dataList"
        :columns="tableColumns"
        :loading="loading"
        :pagination="false"
        @command="handleCommand"
        @reload="load"
      >
        <template #method="{ row }">
          <el-tag size="small">{{ getMethodLabel(row.method) }}</el-tag>
        </template>
        <template #status="{ row }">
          <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
            {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
          </el-tag>
        </template>
      </OnTable>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <OnDialog v-model="dialogVisible" :title="isEdit ? $t('sys.upstreams.editUpstream') : $t('sys.upstreams.addUpstream')" width="700px">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="120px">
        <el-form-item :label="$t('sys.upstreams.name')" prop="name">
          <el-input v-model="form.name" :placeholder="$t('sys.upstreams.namePlaceholder')" :disabled="isEdit" />
        </el-form-item>
        <el-form-item :label="$t('sys.upstreams.method')" prop="method">
          <el-select v-model="form.method" style="width: 100%">
            <el-option :label="$t('sys.upstreams.roundRobin')" value="round_robin" />
            <el-option label="IP Hash" value="ip_hash" />
            <el-option :label="$t('sys.upstreams.leastConn')" value="least_conn" />
            <el-option label="URL Hash" value="hash" />
          </el-select>
        </el-form-item>
        <el-form-item label="Keepalive">
          <el-input-number v-model="form.keepalive" :min="0" :max="1024" />
        </el-form-item>

        <el-divider>{{ $t('sys.upstreams.serverNodes') }}</el-divider>

        <div v-for="(server, index) in form.servers" :key="index" class="server-item">
          <el-row :gutter="12">
            <el-col :span="8">
              <el-form-item
                :label="$t('sys.upstreams.address') + ' ' + (index + 1)"
                :prop="'servers.' + index + '.address'"
                :rules="{ required: true, message: () => t('sys.upstreams.enterAddress'), trigger: 'blur' }"
              >
                <el-input v-model="server.address" placeholder="127.0.0.1:8080" />
              </el-form-item>
            </el-col>
            <el-col :span="4">
              <el-form-item :label="$t('sys.upstreams.weight')">
                <el-input-number v-model="server.weight" :min="1" :max="100" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="4">
              <el-form-item :label="$t('sys.upstreams.maxFails')">
                <el-input-number v-model="server.max_fails" :min="1" :max="10" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="4">
              <el-form-item :label="$t('sys.upstreams.failTimeout')">
                <el-input v-model="server.fail_timeout" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="2">
              <el-form-item :label="$t('sys.upstreams.backup')">
                <el-switch v-model="server.backup" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="2">
              <el-button type="danger" size="small" @click="removeServer(index)" :disabled="form.servers.length <= 1">
                <el-icon><Delete /></el-icon>
              </el-button>
            </el-col>
          </el-row>
        </div>

        <el-form-item>
          <el-button type="primary" plain @click="addServer">
            <el-icon><Plus /></el-icon>
            {{ $t('sys.upstreams.addServer') }}
          </el-button>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="submitting" @click="submitForm">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import type { FormInstance } from 'element-plus'
import type { TableColumn } from '@/components/OnTable/types'
import OnTable from '@/components/OnTable/index.vue'
import {
  listUpstreams,
  getUpstream,
  createUpstream,
  updateUpstream,
  deleteUpstream as deleteUpstreamApi,
} from '@/api/sites'
import { useCrud, useMessage } from '@/hooks'

const { confirm } = useMessage()
import OnDialog from '@/components/OnDialog/index.vue'

const { t } = useI18n()

interface Upstream {
  id: number
  name: string
  method: string
  keepalive: number
  status: string
}

interface UpstreamServer {
  address: string
  weight: number
  max_fails: number
  fail_timeout: string
  backup: boolean
}

function getMethodLabel(method: string): string {
  const labels: Record<string, string> = {
    round_robin: t('sys.upstreams.roundRobin'),
    ip_hash: 'IP Hash',
    least_conn: t('sys.upstreams.leastConn'),
    hash: 'URL Hash',
  }
  return labels[method] || method
}

// ponytail: 上游接口返回整表，无分页，用 isPage:false
const { loading, dataList, load } = useCrud({
  getListApi: listUpstreams,
  isPage: false,
})

const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editId = ref<number | null>(null)
const formRef = ref<FormInstance>()

const tableColumns: TableColumn[] = [
  { prop: 'name', label: 'sys.upstreams.name', width: 200 },
  { prop: 'method', label: 'sys.upstreams.method', width: 150, slot: 'method' },
  { prop: 'keepalive', label: 'Keepalive', width: 100 },
  { prop: 'status', label: 'common.status', width: 100, slot: 'status' },
  {
    label: 'common.action',
    width: 200,
    fixed: 'right',
    buttons: [
      { name: 'common.edit', command: 'edit', size: 'small' },
      { name: 'common.delete', command: 'delete', type: 'danger', size: 'small' },
    ],
  },
]

const form = reactive({
  name: '',
  method: 'round_robin',
  keepalive: 32,
  servers: [
    {
      address: '',
      weight: 1,
      max_fails: 3,
      fail_timeout: '30s',
      backup: false,
    },
  ] as UpstreamServer[],
})

const rules = {
  name: [{ required: true, message: () => t('sys.upstreams.enterName'), trigger: 'blur' }],
  method: [{ required: true, message: () => t('sys.upstreams.selectMethod'), trigger: 'change' }],
}

onMounted(() => {
  load()
})

function handleCommand(command: string | number, row: Upstream) {
  if (command === 'edit') editUpstream(row)
  else if (command === 'delete') deleteUpstream(row)
}

function showAddDialog() {
  isEdit.value = false
  editId.value = null
  resetForm()
  dialogVisible.value = true
}

async function editUpstream(upstream: Upstream) {
  isEdit.value = true
  editId.value = upstream.id

  try {
    const data: any = await getUpstream(upstream.id)
    form.name = data.upstream.name
    form.method = data.upstream.method
    form.keepalive = data.upstream.keepalive
    form.servers = data.servers.map((s: any) => ({
      address: s.address,
      weight: s.weight,
      max_fails: s.max_fails,
      fail_timeout: s.fail_timeout,
      backup: !!s.backup,
    }))
  } catch (error) {
    console.error('获取上游服务器详情失败:', error)
  }

  dialogVisible.value = true
}

function resetForm() {
  form.name = ''
  form.method = 'round_robin'
  form.keepalive = 32
  form.servers = [
    {
      address: '',
      weight: 1,
      max_fails: 3,
      fail_timeout: '30s',
      backup: false,
    },
  ]
}

function addServer() {
  form.servers.push({
    address: '',
    weight: 1,
    max_fails: 3,
    fail_timeout: '30s',
    backup: false,
  })
}

function removeServer(index: number) {
  if (form.servers.length > 1) {
    form.servers.splice(index, 1)
  }
}

async function submitForm() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const data = {
      name: form.name,
      method: form.method,
      keepalive: form.keepalive,
      servers: form.servers,
    }

    if (isEdit.value && editId.value) {
      await updateUpstream(editId.value, data)
      ElMessage.success(t('sys.upstreams.updateSuccess'))
    } else {
      await createUpstream(data)
      ElMessage.success(t('sys.upstreams.createSuccess'))
    }

    dialogVisible.value = false
    load()
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.upstreams.operationFailed'))
  } finally {
    submitting.value = false
  }
}

async function deleteUpstream(upstream: Upstream) {
  const ok = await confirm({
    message: 'sys.upstreams.deleteConfirm',
    params: { name: upstream.name },
  })
  if (!ok) return
  try {
    await deleteUpstreamApi(upstream.id)
    ElMessage.success(t('sys.upstreams.deleteSuccess'))
    load()
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.upstreams.deleteFailed'))
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.server-item {
  background: #f5f7fa;
  padding: 12px;
  border-radius: 4px;
  margin-bottom: 12px;
}
</style>
