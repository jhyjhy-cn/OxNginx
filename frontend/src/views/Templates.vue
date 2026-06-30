<template>
  <div class="templates">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>配置模板</span>
          <el-button type="primary" @click="showAddDialog">
            <el-icon><Plus /></el-icon>
            添加模板
          </el-button>
        </div>
      </template>

      <el-table :data="templates" style="width: 100%" v-loading="loading">
        <el-table-column prop="name" label="模板名称" width="200" />
        <el-table-column prop="description" label="描述" min-width="200" />
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column label="操作" width="250" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="previewTemplate(row)">预览</el-button>
            <el-button size="small" @click="editTemplate(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="deleteTemplate(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="isEdit ? '编辑模板' : '添加模板'"
      width="700px"
    >
      <el-form ref="formRef" :model="form" :rules="formRules" label-width="100px">
        <el-form-item label="模板名称" prop="name">
          <el-input v-model="form.name" placeholder="例如: 反向代理模板" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="form.description" placeholder="模板用途描述" />
        </el-form-item>
        <el-form-item label="配置内容" prop="config">
          <el-input
            v-model="form.config"
            type="textarea"
            :rows="15"
            placeholder="Nginx配置内容，支持 {{变量名}} 格式的变量"
          />
        </el-form-item>
        <el-form-item label="变量定义">
          <el-input
            v-model="form.variables"
            type="textarea"
            :rows="5"
            placeholder='JSON格式，例如: [{"name": "domain", "label": "域名", "required": true}]'
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" @click="submitForm">确定</el-button>
      </template>
    </el-dialog>

    <!-- 预览对话框 -->
    <el-dialog v-model="previewVisible" title="模板预览" width="600px">
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

        <el-divider>预览配置</el-divider>

        <pre class="config-preview">{{ previewConfig }}</pre>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance } from 'element-plus'
import api from '@/api'

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
  name: [{ required: true, message: '请输入模板名称', trigger: 'blur' }],
  config: [{ required: true, message: '请输入配置内容', trigger: 'blur' }],
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
      ElMessage.error('变量定义格式错误，请使用有效的JSON格式')
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
      ElMessage.success('更新成功')
    } else {
      await api.post('/api/templates', data)
      ElMessage.success('创建成功')
    }

    dialogVisible.value = false
    fetchTemplates()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
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
    await ElMessageBox.confirm(`确定要删除模板 ${template.name} 吗？`, '提示', {
      type: 'warning',
    })
    await api.delete(`/api/templates/${template.id}`)
    ElMessage.success('删除成功')
    fetchTemplates()
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
