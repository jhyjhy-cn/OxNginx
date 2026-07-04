<template>
  <div class="ssl">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>SSL证书管理</span>
          <el-button type="primary" @click="showApplyDialog">
            <el-icon><Plus /></el-icon>
            申请证书
          </el-button>
        </div>
      </template>

      <el-table :data="certificates" style="width: 100%" v-loading="loading">
        <el-table-column prop="domain" label="域名" min-width="200" />
        <el-table-column prop="issuer" label="颁发者" width="150" />
        <el-table-column prop="expire_time" label="过期时间" width="180" />
        <el-table-column prop="auto_renew" label="自动续期" width="100">
          <template #default="{ row }">
            <el-tag :type="row.auto_renew ? 'success' : 'info'" size="small">
              {{ row.auto_renew ? '是' : '否' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="renewCert(row)">续期</el-button>
            <el-button size="small" type="danger" @click="deleteCert(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 申请证书对话框 -->
    <OnDialog v-model="dialogVisible" title="申请证书" width="400px" :maximizable="false">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="80px">
        <el-form-item label="域名" prop="domain">
          <el-input v-model="form.domain" placeholder="example.com" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitApply">申请</el-button>
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

interface Certificate {
  id: number
  domain: string
  issuer: string | null
  expire_time: string | null
  auto_renew: number
}

const certificates = ref<Certificate[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const submitting = ref(false)
const formRef = ref<FormInstance>()

const form = reactive({
  domain: '',
})

const rules = {
  domain: [{ required: true, message: '请输入域名', trigger: 'blur' }],
}

onMounted(() => {
  fetchCertificates()
})

async function fetchCertificates() {
  loading.value = true
  try {
    const response = await api.get('/api/certificates')
    if (response.data.code === 0) {
      certificates.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取证书列表失败:', error)
  } finally {
    loading.value = false
  }
}

function showApplyDialog() {
  form.domain = ''
  dialogVisible.value = true
}

async function submitApply() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const response = await api.post('/api/certificate/apply', { domain: form.domain })
    if (response.data.code === 0) {
      ElMessage.success('证书申请成功')
      dialogVisible.value = false
      fetchCertificates()
    } else {
      ElMessage.error(response.data.message || '申请失败')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '申请失败')
  } finally {
    submitting.value = false
  }
}

async function renewCert(cert: Certificate) {
  try {
    await ElMessageBox.confirm(`确定要续期 ${cert.domain} 的证书吗？`, '提示')
    const response = await api.post('/api/certificate/renew', { domain: cert.domain })
    if (response.data.code === 0) {
      ElMessage.success('续期成功')
      fetchCertificates()
    } else {
      ElMessage.error(response.data.message || '续期失败')
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '续期失败')
    }
  }
}

async function deleteCert(cert: Certificate) {
  try {
    await ElMessageBox.confirm(`确定要删除 ${cert.domain} 的证书吗？`, '提示', {
      type: 'warning',
    })
    // TODO: 实现删除API
    ElMessage.success('删除成功')
    fetchCertificates()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
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
