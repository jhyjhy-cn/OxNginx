<template>
  <div class="access-control">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('access.title') }}</span>
          <el-button type="primary" @click="showAddDialog">
            <el-icon><Plus /></el-icon>
            {{ $t('access.addRule') }}
          </el-button>
        </div>
      </template>

      <el-table :data="rules" style="width: 100%" v-loading="loading">
        <el-table-column prop="rule_type" :label="$t('access.ruleType')" width="150">
          <template #default="{ row }">
            <el-tag :type="ruleTypeColors[row.rule_type]" size="small">
              {{ getRuleTypeLabel(row.rule_type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="value" :label="$t('access.ruleValue')" min-width="200" />
        <el-table-column prop="description" :label="$t('access.description')" min-width="150" />
        <el-table-column prop="site_id" :label="$t('access.site')" width="150">
          <template #default="{ row }">
            {{ row.site_id ? getSiteName(row.site_id) : $t('access.global') }}
          </template>
        </el-table-column>
        <el-table-column prop="status" :label="$t('common.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="editRule(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" type="danger" @click="deleteRule(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <OnDialog v-model="dialogVisible" :title="isEdit ? $t('access.editRule') : $t('access.addRule')" width="500px">
      <el-form ref="formRef" :model="form" :rules="validationRules" label-width="100px">
        <el-form-item :label="$t('access.ruleType')" prop="rule_type">
          <el-select v-model="form.rule_type" style="width: 100%" @change="onRuleTypeChange">
            <el-option :label="$t('access.ipAllow')" value="ip_allow" />
            <el-option :label="$t('access.ipDeny')" value="ip_deny" />
            <el-option :label="$t('access.basicAuth')" value="basic_auth" />
            <el-option :label="$t('access.rateLimit')" value="rate_limit" />
          </el-select>
        </el-form-item>

        <el-form-item :label="valueLabel" prop="value">
          <el-input
            v-model="form.value"
            :placeholder="valuePlaceholder"
            :type="form.rule_type === 'basic_auth' ? 'textarea' : 'text'"
            :rows="form.rule_type === 'basic_auth' ? 3 : 1"
          />
          <div class="form-tip" v-if="form.rule_type === 'ip_allow' || form.rule_type === 'ip_deny'">
            {{ $t('access.ipFormatTip') }}
          </div>
          <div class="form-tip" v-if="form.rule_type === 'basic_auth'">
            {{ $t('access.basicAuthFormatTip') }}
          </div>
          <div class="form-tip" v-if="form.rule_type === 'rate_limit'">
            {{ $t('access.rateLimitFormatTip') }}
          </div>
        </el-form-item>

        <el-form-item :label="$t('access.site')">
          <el-select v-model="form.site_id" style="width: 100%" clearable :placeholder="$t('access.global')">
            <el-option v-for="site in sites" :key="site.id" :label="site.name" :value="site.id" />
          </el-select>
        </el-form-item>

        <el-form-item :label="$t('access.description')">
          <el-input v-model="form.description" :placeholder="$t('access.optionalDesc')" />
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
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

const { t } = useI18n()

interface AccessRule {
  id: number
  site_id: number | null
  rule_type: string
  value: string
  description: string | null
  status: string
}

interface Site {
  id: number
  name: string
}

function getRuleTypeLabel(type: string): string {
  const labels: Record<string, string> = {
    ip_allow: t('access.ipAllow'),
    ip_deny: t('access.ipDeny'),
    basic_auth: t('access.basicAuth'),
    rate_limit: t('access.rateLimit'),
  }
  return labels[type] || type
}

const ruleTypeColors: Record<string, string> = {
  ip_allow: 'success',
  ip_deny: 'danger',
  basic_auth: 'warning',
  rate_limit: 'info',
}

const rules = ref<AccessRule[]>([])
const sites = ref<Site[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editId = ref<number | null>(null)
const formRef = ref<FormInstance>()

const form = reactive({
  rule_type: 'ip_allow',
  value: '',
  site_id: null as number | null,
  description: '',
})

const valueLabel = computed(() => {
  switch (form.rule_type) {
    case 'ip_allow':
    case 'ip_deny':
      return t('access.ipAddress')
    case 'basic_auth':
      return t('access.userPass')
    case 'rate_limit':
      return t('access.rateLimit')
    default:
      return t('access.ruleValue')
  }
})

const valuePlaceholder = computed(() => {
  switch (form.rule_type) {
    case 'ip_allow':
    case 'ip_deny':
      return '192.168.1.100 或 192.168.1.0/24'
    case 'basic_auth':
      return 'admin:password123'
    case 'rate_limit':
      return '10r/s'
    default:
      return ''
  }
})

const validationRules = {
  value: [{ required: true, message: () => t('access.enterRuleValue'), trigger: 'blur' }],
}

onMounted(() => {
  fetchRules()
  fetchSites()
})

function getSiteName(siteId: number): string {
  const site = sites.value.find((s) => s.id === siteId)
  return site ? site.name : `#${siteId}`
}

function onRuleTypeChange() {
  form.value = ''
}

async function fetchRules() {
  loading.value = true
  try {
    const response = await api.get('/api/access-rules')
    if (response.data.code === 0) {
      rules.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取规则列表失败:', error)
  } finally {
    loading.value = false
  }
}

async function fetchSites() {
  try {
    const response = await api.get('/api/sites')
    if (response.data.code === 0) {
      sites.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取站点列表失败:', error)
  }
}

function showAddDialog() {
  isEdit.value = false
  editId.value = null
  resetForm()
  dialogVisible.value = true
}

function editRule(rule: AccessRule) {
  isEdit.value = true
  editId.value = rule.id
  form.rule_type = rule.rule_type
  form.value = rule.value
  form.site_id = rule.site_id
  form.description = rule.description || ''
  dialogVisible.value = true
}

function resetForm() {
  form.rule_type = 'ip_allow'
  form.value = ''
  form.site_id = null
  form.description = ''
}

async function submitForm() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const data = {
      rule_type: form.rule_type,
      value: form.value,
      site_id: form.site_id,
      description: form.description || null,
    }

    if (isEdit.value && editId.value) {
      await api.put(`/api/access-rules/${editId.value}`, data)
      ElMessage.success(t('access.updateSuccess'))
    } else {
      await api.post('/api/access-rules', data)
      ElMessage.success(t('access.createSuccess'))
    }

    dialogVisible.value = false
    fetchRules()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('common.operationFailed'))
  } finally {
    submitting.value = false
  }
}

async function deleteRule(rule: AccessRule) {
  try {
    await ElMessageBox.confirm(t('access.deleteConfirm'), t('common.tip'), {
      type: 'warning',
    })
    await api.delete(`/api/access-rules/${rule.id}`)
    ElMessage.success(t('access.deleteSuccess'))
    fetchRules()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('access.deleteFailed'))
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

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}
</style>
