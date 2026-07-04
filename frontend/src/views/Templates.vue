<template>
  <div class="templates">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('templates.title') }}</span>
          <el-button type="primary" @click="showAddDialog">
            <el-icon><Plus /></el-icon>
            {{ $t('templates.addTemplate') }}
          </el-button>
        </div>
      </template>

      <el-table :data="templates" style="width: 100%" v-loading="loading">
        <el-table-column prop="name" :label="$t('templates.templateName')" width="200" />
        <el-table-column prop="description" :label="$t('templates.templateDesc')" min-width="200" />
        <el-table-column prop="created_at" :label="$t('common.createdAt')" width="180" />
        <el-table-column :label="$t('common.action')" width="250" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="previewTemplate(row)">{{ $t('templates.preview') }}</el-button>
            <el-button size="small" @click="editTemplate(row)">{{ $t('common.edit') }}</el-button>
            <el-button size="small" type="danger" @click="deleteTemplate(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <OnDialog
      v-model="dialogVisible"
      :title="isEdit ? $t('templates.editTemplate') : $t('templates.addTemplate')"
      width="700px"
    >
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="100px">
        <el-form-item :label="$t('templates.templateName')" prop="name">
          <el-input v-model="form.name" :placeholder="$t('templates.namePlaceholder')" />
        </el-form-item>
        <el-form-item :label="$t('templates.templateDesc')">
          <el-input v-model="form.description" :placeholder="$t('templates.descPlaceholder')" />
        </el-form-item>
        <el-form-item :label="$t('templates.configContent')" prop="config">
          <el-input
            v-model="form.config"
            type="textarea"
            :rows="15"
            :placeholder="$t('templates.configPlaceholder')"
          />
        </el-form-item>
        <el-form-item :label="$t('templates.variableDef')">
          <el-input
            v-model="form.variables"
            type="textarea"
            :rows="5"
            :placeholder="$t('templates.variablePlaceholder')"
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="submitting" @click="submitForm">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 预览对话框 -->
    <OnDialog v-model="previewVisible" :title="$t('templates.templatePreview')" width="600px" :maximizable="false">
      <div v-if="previewTemplate_">
        <el-form label-width="100px">
          <el-form-item
            v-for="variable in templateVariables"
            :key="variable.name"
            :label="variable.label || variable.name"
          >
            <el-input v-model="previewVars[variable.name]" :placeholder="variable.name" />
          </el-form-item>
        </el-form>

        <el-divider>{{ $t('templates.previewConfig') }}</el-divider>

        <pre class="config-preview">{{ previewConfig }}</pre>
      </div>
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

interface Template {
  id: number
  name: string
  description: string | null
  config: string
  variables: string | null
  created_at: string
}

interface TemplateVariable {
  name: string
  label?: string
  required?: boolean
}

const templates = ref<Template[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const previewVisible = ref(false)
const isEdit = ref(false)
const submitting = ref(false)
const editId = ref<number | null>(null)
const formRef = ref<FormInstance>()
const previewTemplate_ = ref<Template | null>(null)
const previewVars = ref<Record<string, string>>({})
const previewConfig = ref('')

const form = reactive({
  name: '',
  description: '',
  config: '',
  variables: '',
})

const formRules = {
  name: [{ required: true, message: () => t('templates.enterName'), trigger: 'blur' }],
  config: [{ required: true, message: () => t('templates.enterConfig'), trigger: 'blur' }],
}

const templateVariables = computed<TemplateVariable[]>(() => {
  if (!previewTemplate_?.value?.variables) return []
  try {
    return JSON.parse(previewTemplate_!.value!.variables!)
  } catch {
    return []
  }
})

onMounted(() => {
  fetchTemplates()
})

async function fetchTemplates() {
  loading.value = true
  try {
    const response = await api.get('/api/templates')
    if (response.data.code === 0) {
      templates.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取模板列表失败:', error)
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

function editTemplate(template: Template) {
  isEdit.value = true
  editId.value = template.id
  form.name = template.name
  form.description = template.description || ''
  form.config = template.config
  form.variables = template.variables || ''
  dialogVisible.value = true
}

function resetForm() {
  form.name = ''
  form.description = ''
  form.config = ''
  form.variables = ''
}

async function submitForm() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  // 验证变量JSON格式
  if (form.variables) {
    try {
      JSON.parse(form.variables)
    } catch {
      ElMessage.error(t('templates.variableFormatError'))
      return
    }
  }

  submitting.value = true
  try {
    const data = {
      name: form.name,
      description: form.description || null,
      config: form.config,
      variables: form.variables || null,
    }

    if (isEdit.value && editId.value) {
      await api.put(`/api/templates/${editId.value}`, data)
      ElMessage.success(t('templates.updateSuccess'))
    } else {
      await api.post('/api/templates', data)
      ElMessage.success(t('templates.createSuccess'))
    }

    dialogVisible.value = false
    fetchTemplates()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('templates.operationFailed'))
  } finally {
    submitting.value = false
  }
}

async function previewTemplate(template: Template) {
  previewTemplate_.value = template
  previewVars.value = {}
  previewConfig.value = ''

  // 初始化变量
  if (template.variables) {
    try {
      const vars: TemplateVariable[] = JSON.parse(template.variables)
      vars.forEach(v => {
        previewVars.value[v.name] = ''
      })
    } catch {}
  }

  previewVisible.value = true
  updatePreview()
}

async function updatePreview() {
  if (!previewTemplate_?.value) return

  try {
    const response = await api.post(`/api/templates/${previewTemplate_!.value!.id}/preview`, previewVars.value)
    if (response.data.code === 0) {
      previewConfig.value = response.data.data.config
    }
  } catch (error) {
    console.error('预览失败:', error)
  }
}

async function deleteTemplate(template: Template) {
  try {
    await ElMessageBox.confirm(t('templates.deleteConfirm', { name: template.name }), t('common.tip'), {
      type: 'warning',
    })
    await api.delete(`/api/templates/${template.id}`)
    ElMessage.success(t('templates.deleteSuccess'))
    fetchTemplates()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('templates.deleteFailed'))
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

.config-preview {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 4px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
  max-height: 400px;
  overflow-y: auto;
}
</style>
