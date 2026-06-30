<template>
  <div class="access-control">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>访问控制</span>
          <el-button type="primary" @click="showAddDialog">
            <el-icon><Plus /></el-icon>
            添加规则
          </el-button>
        </div>
      </template>

      <el-table :data="rules" style="width: 100%" v-loading="loading">
        <el-table-column prop="rule_type" label="规则类型" width="150">
          <template #default="{ row }">
            <el-tag :type="ruleTypeColors[row.rule_type]" size="small">
              {{ ruleTypeLabels[row.rule_type] || row.rule_type }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="value" label="规则值" min-width="200" />
        <el-table-column prop="description" label="描述" min-width="150" />
        <el-table-column prop="site_id" label="适用站点" width="150">
          <template #default="{ row }">
            {{ row.site_id ? getSiteName(row.site_id) : '全局' }}
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ row.status === 'enabled' ? '启用' : '禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="editRule(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="deleteRule(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑规则' : '添加规则'"
      width="500px"
    >
      <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
        <el-form-item label="规则类型" prop="rule_type">
          <el-select v-model="form.rule_type" style="width: 100%" @change="onRuleTypeChange">
            <el-option label="IP 白名单" value="ip_allow" />
            <el-option label="IP 黑名单" value="ip_deny" />
            <el-option label="Basic Auth" value="basic_auth" />
            <el-option label="速率限制" value="rate_limit" />
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
            支持单个IP或CIDR格式，例如：192.168.1.100 或 192.168.1.0/24
          </div>
          <div class="form-tip" v-if="form.rule_type === 'basic_auth'">
            格式：用户名:密码，每行一个
          </div>
          <div class="form-tip" v-if="form.rule_type === 'rate_limit'">
            格式：请求数/时间，例如：10r/s 或 100r/m
          </div>
        </el-form-item>

        <el-form-item label="适用站点">
          <el-select v-model="form.site_id" style="width: 100%" clearable placeholder="全局">
            <el-option
              v-for="site in sites"
              :key="site.id"
              :label="site.name"
              :value="site.id"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="描述">
          <el-input v-model="form.description" placeholder="可选描述" />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitForm">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import api from '@/api'

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

const ruleTypeLabels: Record<string, string> = {
  ip_allow: 'IP白名单',
  ip_deny: 'IP黑名单',
  basic_auth: 'Basic Auth',
  rate_limit: '速率限制',
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
      return 'IP地址'
    case 'basic_auth':
      return '用户名:密码'
    case 'rate_limit':
      return '速率限制'
    default:
      return '规则值'
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

onMounted(() => {
  fetchRules()
  fetchSites()
})

function getSiteName(siteId: number): string {
  const site = sites.value.find(s => s.id === siteId)
  return site ? site.name : `站点#${siteId}`
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
      ElMessage.success('更新成功')
    } else {
      await api.post('/api/access-rules', data)
      ElMessage.success('创建成功')
    }

    dialogVisible.value = false
    fetchRules()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
  } finally {
    submitting.value = false
  }
}

async function deleteRule(rule: AccessRule) {
  try {
    await ElMessageBox.confirm('确定要删除该规则吗？', '提示', {
      type: 'warning',
    })
    await api.delete(`/api/access-rules/${rule.id}`)
    ElMessage.success('删除成功')
    fetchRules()
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

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}
</style>
