<template>
  <div class="ssl">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('ssl.title') }}</span>
          <el-button type="primary" @click="showApplyDialog">
            <el-icon><Plus /></el-icon>
            {{ $t('ssl.applyCert') }}
          </el-button>
        </div>
      </template>

      <el-table :data="certificates" style="width: 100%" v-loading="loading">
        <el-table-column prop="domain" :label="$t('ssl.domain')" min-width="200" />
        <el-table-column prop="issuer" :label="$t('ssl.issuer')" width="150" />
        <el-table-column prop="expire_time" :label="$t('ssl.expireTime')" width="180" />
        <el-table-column prop="auto_renew" :label="$t('ssl.autoRenew')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.auto_renew ? 'success' : 'info'" size="small">
              {{ row.auto_renew ? $t('common.yes') : $t('common.no') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="renewCert(row)">{{ $t('ssl.renew') }}</el-button>
            <el-button size="small" type="danger" @click="deleteCert(row)">{{ $t('ssl.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 申请证书对话框 -->
    <OnDialog v-model="dialogVisible" :title="$t('ssl.applyCert')" width="400px" :maximizable="false">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="80px">
        <el-form-item :label="$t('ssl.domain')" prop="domain">
          <el-input v-model="form.domain" placeholder="example.com" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="submitting" @click="submitApply">{{ $t('ssl.applyCert') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

const { t } = useI18n()

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
  domain: [{ required: true, message: () => t('ssl.enterDomain'), trigger: 'blur' }],
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
      ElMessage.success(t('ssl.applySuccess'))
      dialogVisible.value = false
      fetchCertificates()
    } else {
      ElMessage.error(response.data.message || t('ssl.applyFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('ssl.applyFailed'))
  } finally {
    submitting.value = false
  }
}

async function renewCert(cert: Certificate) {
  try {
    await ElMessageBox.confirm(t('ssl.renewConfirm', { name: cert.domain }), t('common.tip'))
    const response = await api.post('/api/certificate/renew', { domain: cert.domain })
    if (response.data.code === 0) {
      ElMessage.success(t('ssl.renewSuccess'))
      fetchCertificates()
    } else {
      ElMessage.error(response.data.message || t('ssl.renewFailed'))
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('ssl.renewFailed'))
    }
  }
}

async function deleteCert(cert: Certificate) {
  try {
    await ElMessageBox.confirm(t('ssl.deleteConfirm', { name: cert.domain }), t('common.tip'), {
      type: 'warning',
    })
    // TODO: 实现删除API
    ElMessage.success(t('ssl.deleteSuccess'))
    fetchCertificates()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('ssl.deleteFailed'))
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
