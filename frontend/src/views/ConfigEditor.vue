<template>
  <div class="config-editor">
    <el-row :gutter="20">
      <!-- 左侧文件列表 -->
      <el-col :span="6">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>配置文件</span>
              <el-button size="small" @click="refreshFiles">
                <el-icon><Refresh /></el-icon>
              </el-button>
            </div>
          </template>

          <div class="file-list">
            <div
              class="file-item"
              :class="{ active: currentFile === 'main' }"
              @click="loadMainConfig"
            >
              <el-icon><Document /></el-icon>
              <span>nginx.conf</span>
            </div>

            <el-divider />

            <div class="file-group-title">站点配置</div>
            <div
              v-for="file in configFiles"
              :key="file.name"
              class="file-item"
              :class="{ active: currentFile === file.name }"
              @click="loadSiteConfig(file.name)"
            >
              <el-icon><Document /></el-icon>
              <span class="file-name">{{ file.name }}</span>
              <el-tag
                :type="file.enabled ? 'success' : 'info'"
                size="small"
                class="file-status"
              >
                {{ file.enabled ? '启用' : '禁用' }}
              </el-tag>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 右侧编辑器 -->
      <el-col :span="18">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>{{ currentFileName }}</span>
              <div class="editor-actions">
                <el-button
                  v-if="currentFile && currentFile !== 'main'"
                  size="small"
                  @click="toggleConfig"
                >
                  {{ currentFileEnabled ? '禁用' : '启用' }}
                </el-button>
                <el-button type="primary" size="small" @click="saveConfig" :loading="saving">
                  保存
                </el-button>
                <el-button
                  v-if="currentFile && currentFile !== 'main'"
                  type="danger"
                  size="small"
                  @click="deleteConfig"
                >
                  删除
                </el-button>
              </div>
            </div>
          </template>

          <div class="editor-container" v-loading="loading">
            <div ref="editorRef" class="monaco-editor"></div>
          </div>

          <div class="editor-footer" v-if="currentFile">
            <el-text type="info" size="small">
              修改后点击保存，配置将自动测试。如果测试失败，将自动回滚到上一个版本。
            </el-text>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { monaco } from '@/utils/monaco-env'
import api from '@/api'

interface ConfigFile {
  name: string
  path: string
  size: number
  modified: string
  enabled: boolean
}

const configFiles = ref<ConfigFile[]>([])
const currentFile = ref<string>('')
const currentFileEnabled = ref(false)
const loading = ref(false)
const saving = ref(false)
const editorRef = ref<HTMLElement>()

let editor: monaco.editor.IStandaloneCodeEditor | null = null

const currentFileName = ref('选择一个配置文件')

onMounted(() => {
  refreshFiles()
  initEditor()
})

onUnmounted(() => {
  if (editor) {
    editor.dispose()
  }
})

function initEditor() {
  nextTick(() => {
    if (editorRef.value) {
      editor = monaco.editor.create(editorRef.value, {
        value: '',
        language: 'nginx',
        theme: 'vs-dark',
        minimap: { enabled: false },
        fontSize: 14,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        automaticLayout: true,
        tabSize: 4,
        wordWrap: 'on',
      })
    }
  })
}

async function refreshFiles() {
  try {
    const response = await api.get('/api/config/files')
    if (response.data.code === 0) {
      configFiles.value = response.data.data || []
    }
  } catch (error) {
    console.error('获取配置文件列表失败:', error)
  }
}

async function loadMainConfig() {
  loading.value = true
  currentFile.value = 'main'
  currentFileName.value = 'nginx.conf'
  currentFileEnabled.value = true

  try {
    const response = await api.get('/api/config/main')
    if (response.data.code === 0 && editor) {
      editor.setValue(response.data.data.content)
      monaco.editor.setModelLanguage(editor.getModel()!, 'nginx')
    }
  } catch (error) {
    ElMessage.error('读取配置文件失败')
  } finally {
    loading.value = false
  }
}

async function loadSiteConfig(name: string) {
  loading.value = true
  currentFile.value = name
  currentFileName.value = name
  currentFileEnabled.value = configFiles.value.find(f => f.name === name)?.enabled || false

  try {
    const response = await api.get(`/api/config/file/${name}`)
    if (response.data.code === 0 && editor) {
      editor.setValue(response.data.data.content)
      monaco.editor.setModelLanguage(editor.getModel()!, 'nginx')
    }
  } catch (error) {
    ElMessage.error('读取配置文件失败')
  } finally {
    loading.value = false
  }
}

async function saveConfig() {
  if (!editor || !currentFile.value) return

  saving.value = true
  const content = editor.getValue()

  try {
    let response
    if (currentFile.value === 'main') {
      response = await api.put('/api/config/main', { content })
    } else {
      response = await api.put(`/api/config/file/${currentFile.value}`, { content })
    }

    if (response.data.code === 0) {
      ElMessage.success('配置保存成功')
    } else {
      ElMessage.error(response.data.message || '保存失败')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function toggleConfig() {
  if (!currentFile.value || currentFile.value === 'main') return

  try {
    const response = await api.post(`/api/config/file/${currentFile.value}/toggle`)
    if (response.data.code === 0) {
      ElMessage.success(response.data.data)
      currentFileEnabled.value = !currentFileEnabled.value
      refreshFiles()
    } else {
      ElMessage.error(response.data.message || '操作失败')
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '操作失败')
  }
}

async function deleteConfig() {
  if (!currentFile.value || currentFile.value === 'main') return

  try {
    await ElMessageBox.confirm(`确定要删除配置文件 ${currentFile.value} 吗？`, '提示', {
      type: 'warning',
    })

    const response = await api.delete(`/api/config/file/${currentFile.value}`)
    if (response.data.code === 0) {
      ElMessage.success('配置文件已删除')
      currentFile.value = ''
      currentFileName.value = '选择一个配置文件'
      if (editor) {
        editor.setValue('')
      }
      refreshFiles()
    } else {
      ElMessage.error(response.data.message || '删除失败')
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || '删除失败')
    }
  }
}
</script>

<style scoped>
.config-editor {
  height: calc(100vh - 120px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-list {
  max-height: calc(100vh - 240px);
  overflow-y: auto;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.file-item:hover {
  background-color: #f5f7fa;
}

.file-item.active {
  background-color: #ecf5ff;
  color: #409eff;
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-status {
  margin-left: auto;
}

.file-group-title {
  font-size: 12px;
  color: #909399;
  padding: 8px 12px 4px;
}

.editor-container {
  height: calc(100vh - 280px);
}

.monaco-editor {
  width: 100%;
  height: 100%;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
}

.editor-actions {
  display: flex;
  gap: 8px;
}

.editor-footer {
  margin-top: 12px;
}
</style>
