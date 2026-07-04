<template>
  <div class="upstreams">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>负载均衡管理</span>
          <el-button type="primary" @click="showAddDialog">
            <el-icon><Plus /></el-icon>
            添加上游服务器
          </el-button>
        </div>
      </template>

      <el-table :data="upstreams" style="width: 100%" v-loading="loading">
        <el-table-column prop="name" label="名称" width="200" />
        <el-table-column prop="method" label="负载策略" width="150">
          <template #default="{ row }">
            <el-tag size="small">{{ methodLabels[row.method] || row.method }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="keepalive" label="Keepalive" width="100" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ row.status === 'enabled' ? '启用' : '禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="editUpstream(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="deleteUpstream(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <OnDialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑上游服务器' : '添加上游服务器'"
      width="700px"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="120px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="例如: backend" :disabled="isEdit" />
        </el-form-item>
        <el-form-item label="负载策略" prop="method">
          <el-select v-model="form.method" style="width: 100%">
            <el-option label="轮询 (Round Robin)" value="round_robin" />
            <el-option label="IP Hash" value="ip_hash" />
            <el-option label="最少连接 (Least Conn)" value="least_conn" />
            <el-option label="URL Hash" value="hash" />
          </el-select>
        </el-form-item>
        <el-form-item label="Keepalive">
          <el-input-number v-model="form.keepalive" :min="0" :max="1024" />
        </el-form-item>

        <el-divider>服务器节点</el-divider>

        <div v-for="(server, index) in form.servers" :key="index" class="server-item">
          <el-row :gutter="12">
            <el-col :span="8">
              <el-form-item :label="'地址 ' + (index + 1)" :prop="'servers.' + index + '.address'" :rules="{ required: true, message: '请输入地址', trigger: 'blur' }">
                <el-input v-model="server.address" placeholder="127.0.0.1:8080" />
              </el-form-item>
            </el-col>
            <el-col :span="4">
              <el-form-item label="权重">
                <el-input-number v-model="server.weight" :min="1" :max="100" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="4">
              <el-form-item label="最大失败">
                <el-input-number v-model="server.max_fails" :min="1" :max="10" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="4">
              <el-form-item label="失败超时">
                <el-input v-model="server.fail_timeout" size="small" />
              </el-form-item>
            </el-col>
            <el-col :span="2">
              <el-form-item label="备份">
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
            添加服务器
          </el-button>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitForm">确定</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

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

const methodLabels: Record<string, string> = {
  round_robin: '轮询',
  ip_hash: 'IP Hash',
  least_conn: '最少连接',
  hash: 'URL Hash',
}

const upstreams = ref<Upstream[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editId = ref<number | null>(null)
const formRef = ref<FormInstance>()

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
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  method: [{ required: true, message: '请选择负载策略', trigger: 'change' }],
}

onMounted(() => {
  fetchUpstreams()
})

async function fetchUpstreams() {
  loading.value = true
  try {
    const response = await api.get('/api/upstreams')
    if (response.data.code === 0) {
      upstreams.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取上游服务器列表失败:', error)
  } finally {
    loading.value = false
  }
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
    const response = await api.get(`/api/upstreams/${upstream.id}`)
    if (response.data.code === 0) {
      const data = response.data.data
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
    }
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
      await api.put(`/api/upstreams/${editId.value}`, data)
      ElMessage.success('更新成功')
    } else {
      await api.post('/api/upstreams', data)
      ElMessage.success('创建成功')
    }

    dialogVisible.value = false
    fetchUpstreams()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
  } finally {
    submitting.value = false
  }
}

async function deleteUpstream(upstream: Upstream) {
  try {
    await ElMessageBox.confirm(`确定要删除上游服务器 ${upstream.name} 吗？`, '提示', {
      type: 'warning',
    })
    await api.delete(`/api/upstreams/${upstream.id}`)
    ElMessage.success('删除成功')
    fetchUpstreams()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '删除失败')
    }
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
