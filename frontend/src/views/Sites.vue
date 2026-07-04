<template>
  <div class="sites">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>站点管理</span>
          <div>
            <el-button-group v-if="selectedSites.length > 0" style="margin-right: 12px">
              <el-button size="small" @click="batchEnable">
                批量启用 ({{ selectedSites.length }})
              </el-button>
              <el-button size="small" @click="batchDisable">
                批量禁用 ({{ selectedSites.length }})
              </el-button>
              <el-button size="small" type="danger" @click="batchDelete">
                批量删除 ({{ selectedSites.length }})
              </el-button>
            </el-button-group>
            <el-button type="primary" @click="showAddDialog">
              <el-icon><Plus /></el-icon>
              添加站点
            </el-button>
          </div>
        </div>
      </template>

      <el-table :data="sites" style="width: 100%" v-loading="loading" @selection-change="handleSelectionChange">
        <el-table-column type="selection" width="55" />
        <el-table-column prop="name" label="网站名" width="150">
          <template #default="{ row }">
            <el-button type="primary" link @click="editSite(row)">{{ row.name }}</el-button>
          </template>
        </el-table-column>
        <el-table-column prop="listen" label="端口" width="80" />
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <el-switch
              :model-value="row.status === 'enabled'"
              inline-prompt
              active-text="启"
              inactive-text="停"
              @change="(val: boolean) => toggleSite(row, val)"
            />
          </template>
        </el-table-column>
        <el-table-column label="根目录" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.root_path || row.proxy_pass || '-' }}
          </template>
        </el-table-column>
        <el-table-column label="SSL证书" width="140">
          <template #default="{ row }">
            <el-tag v-if="row.ssl === 1 && row.cert_expire_days != null" :type="row.cert_expire_days > 30 ? 'success' : row.cert_expire_days > 7 ? 'warning' : 'danger'" size="small">
              剩余 {{ row.cert_expire_days }} 天
            </el-tag>
            <el-tag v-else-if="row.ssl === 1" type="success" size="small">已部署</el-tag>
            <el-tag v-else type="info" size="small">未部署</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="170">
          <template #default="{ row }">
            {{ row.created_at ? new Date(row.created_at).toLocaleString() : '-' }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" link @click="editSite(row)">编辑</el-button>
            <el-button type="primary" link @click="deploySSL(row)" :loading="row._sslLoading">SSL部署</el-button>
            <el-button type="danger" link @click="deleteSite(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <OnDialog
      v-model="dialogVisible"
      :title="isEdit ? `站点修改[${editSiteName}] - 添加时间[${editCreatedAt}]` : '添加站点'"
      width="600px"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
        <el-form-item label="名称" prop="name">
          <el-input v-model="form.name" placeholder="站点名称" />
        </el-form-item>
        <el-form-item label="域名" prop="server_name">
          <el-input v-model="form.server_name" placeholder="example.com" />
        </el-form-item>
        <el-form-item label="监听端口" prop="listen">
          <el-input v-model="form.listen" placeholder="80" />
        </el-form-item>
        <el-form-item label="启用SSL">
          <el-switch v-model="form.ssl" />
        </el-form-item>
        <template v-if="form.ssl">
          <el-form-item label="证书路径">
            <el-input v-model="form.certificate_path" placeholder="/opt/oxnginx/ssl/fullchain.cer" />
          </el-form-item>
          <el-form-item label="密钥路径">
            <el-input v-model="form.key_path" placeholder="/opt/oxnginx/ssl/private.key" />
          </el-form-item>
        </template>
        <el-form-item label="反向代理">
          <el-input v-model="form.proxy_pass" placeholder="http://127.0.0.1:8080" />
        </el-form-item>
        <el-form-item label="根目录">
          <el-input v-model="form.root_path" placeholder="/opt/oxnginx/wwwroot" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitForm">确定</el-button>
      </template>
    </OnDialog>

    <!-- 删除确认对话框 -->
    <OnDialog v-model="deleteDialogVisible" title="删除站点" width="420px" :maximizable="false">
      <div style="margin-bottom: 16px;">
        <p>确定要删除站点 <strong>{{ deleteTarget?.name }}</strong> 吗？</p>
      </div>
      <el-checkbox v-model="deleteOptions.deleteRecord">
        删除站点记录（从数据库中移除）
      </el-checkbox>
      <el-checkbox v-model="deleteOptions.deleteFiles" style="margin-top: 12px;">
        删除站点文件目录（{{ deleteTarget?.root_path || '无' }}）
      </el-checkbox>
      <template #footer>
        <el-button @click="deleteDialogVisible = false">取消</el-button>
        <el-button type="danger" @click="confirmDelete">确认删除</el-button>
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

interface Site {
  id: number
  name: string
  server_name: string
  listen: string
  ssl: number
  certificate_path: string | null
  key_path: string | null
  proxy_pass: string | null
  root_path: string | null
  status: string
  created_at?: string
  expire_time?: string
  cert_expire_days?: number
}

const sites = ref<Site[]>([])
const selectedSites = ref<Site[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editId = ref<number | null>(null)
const editSiteName = ref('')
const editCreatedAt = ref('')
const formRef = ref<FormInstance>()

const form = reactive({
  name: '',
  server_name: '',
  listen: '80',
  ssl: false,
  certificate_path: '',
  key_path: '',
  proxy_pass: '',
  root_path: '',
})

const rules = {
  name: [{ required: true, message: '请输入站点名称', trigger: 'blur' }],
  server_name: [{ required: true, message: '请输入域名', trigger: 'blur' }],
}

onMounted(() => {
  fetchSites()
})

async function fetchSites() {
  loading.value = true
  try {
    const response = await api.get('/api/sites/with-certs')
    if (response.data.code === 0) {
      sites.value = (response.data.data || []).map((s: Site) => {
        if (s.expire_time) {
          const expireDate = new Date(s.expire_time)
          const now = new Date()
          s.cert_expire_days = Math.ceil((expireDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
        }
        return s
      })
    }
  } catch (error) {
    console.error('获取站点列表失败:', error)
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

function editSite(site: Site) {
  isEdit.value = true
  editId.value = site.id
  editSiteName.value = site.name
  editCreatedAt.value = site.created_at ? new Date(site.created_at).toLocaleString() : '-'
  form.name = site.name
  form.server_name = site.server_name
  form.listen = site.listen
  form.ssl = !!site.ssl
  form.certificate_path = site.certificate_path || ''
  form.key_path = site.key_path || ''
  form.proxy_pass = site.proxy_pass || ''
  form.root_path = site.root_path || ''
  dialogVisible.value = true
}

function resetForm() {
  form.name = ''
  form.server_name = ''
  form.listen = '80'
  form.ssl = false
  form.certificate_path = ''
  form.key_path = ''
  form.proxy_pass = ''
  form.root_path = ''
}

async function submitForm() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const data = {
      name: form.name,
      server_name: form.server_name,
      listen: form.listen,
      ssl: form.ssl,
      certificate_path: form.certificate_path || null,
      key_path: form.key_path || null,
      proxy_pass: form.proxy_pass || null,
      root_path: form.root_path || null,
    }

    if (isEdit.value && editId.value) {
      await api.put(`/api/sites/${editId.value}`, data)
      ElMessage.success('更新成功')
    } else {
      await api.post('/api/sites', data)
      ElMessage.success('创建成功')
    }

    dialogVisible.value = false
    fetchSites()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
  } finally {
    submitting.value = false
  }
}

async function toggleSite(site: Site, enable?: boolean) {
  const newStatus = enable !== undefined
    ? (enable ? 'enabled' : 'disabled')
    : (site.status === 'enabled' ? 'disabled' : 'enabled')
  try {
    await api.put(`/api/sites/${site.id}`, { status: newStatus })
    ElMessage.success(newStatus === 'enabled' ? '已启用' : '已停用')
    fetchSites()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
  }
}

async function deploySSL(site: Site) {
  try {
    await ElMessageBox.confirm(
      `将为「${site.server_name}」申请 Let's Encrypt 证书并绑定到该站点，是否继续？`,
      'SSL 证书部署',
      { type: 'warning' }
    )
    const response = await api.post(`/api/sites/${site.id}/deploy-ssl`)
    if (response.data.code === 0) {
      ElMessage.success('SSL 证书部署成功！')
      fetchSites()
    } else {
      ElMessage.error(response.data.message || '部署失败')
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || error.message || '部署失败')
    }
  }
}

const deleteDialogVisible = ref(false)
const deleteTarget = ref<Site | null>(null)
const deleteOptions = reactive({
  deleteRecord: true,
  deleteFiles: false,
})

function deleteSite(site: Site) {
  deleteTarget.value = site
  deleteOptions.deleteRecord = true
  deleteOptions.deleteFiles = false
  deleteDialogVisible.value = true
}

async function confirmDelete() {
  if (!deleteTarget.value) return
  try {
    await api.delete(`/api/sites/${deleteTarget.value.id}`, {
      data: {
        delete_record: deleteOptions.deleteRecord,
        delete_files: deleteOptions.deleteFiles,
      },
    })
    ElMessage.success('删除成功')
    deleteDialogVisible.value = false
    fetchSites()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '删除失败')
  }
}

function handleSelectionChange(selection: Site[]) {
  selectedSites.value = selection
}

async function batchEnable() {
  try {
    await ElMessageBox.confirm(`确定要启用选中的 ${selectedSites.value.length} 个站点吗？`, '提示')
    const response = await api.post('/api/sites/batch/enable', {
      ids: selectedSites.value.map(s => s.id),
    })
    if (response.data.code === 0) {
      ElMessage.success(`成功启用 ${response.data.data.success} 个站点`)
      fetchSites()
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '操作失败')
    }
  }
}

async function batchDisable() {
  try {
    await ElMessageBox.confirm(`确定要禁用选中的 ${selectedSites.value.length} 个站点吗？`, '提示')
    const response = await api.post('/api/sites/batch/disable', {
      ids: selectedSites.value.map(s => s.id),
    })
    if (response.data.code === 0) {
      ElMessage.success(`成功禁用 ${response.data.data.success} 个站点`)
      fetchSites()
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '操作失败')
    }
  }
}

async function batchDelete() {
  try {
    await ElMessageBox.confirm(`确定要删除选中的 ${selectedSites.value.length} 个站点吗？此操作不可恢复！`, '警告', {
      type: 'warning',
    })
    const response = await api.post('/api/sites/batch/delete', {
      ids: selectedSites.value.map(s => s.id),
    })
    if (response.data.code === 0) {
      ElMessage.success(`成功删除 ${response.data.data.success} 个站点`)
      fetchSites()
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '操作失败')
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
</style>
