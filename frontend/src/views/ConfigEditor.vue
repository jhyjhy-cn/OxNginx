<template>
  <div class="config-editor">
    <el-row :gutter="20">
      <!-- 左侧文件列表 -->
      <el-col :span="6">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>{{ $t('config.configFiles') }}</span>
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

            <div class="file-group-title">{{ $t('config.siteConfigs') }}</div>
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
                {{ file.enabled ? $t('common.enabled') : $t('common.disabled') }}
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
              <span>{{ currentFileKey ? $t(currentFileKey) : currentFileName }}</span>
              <div class="editor-actions">
                <el-button
                  v-if="currentFile && currentFile !== 'main'"
                  size="small"
                  @click="toggleConfig"
                >
                  {{ currentFileEnabled ? $t('common.disabled') : $t('common.enabled') }}
                </el-button>
                <el-button type="primary" size="small" @click="saveConfig" :loading="saving">
                  {{ $t('config.save') }}
                </el-button>
                <el-button
                  v-if="currentFile && currentFile !== 'main'"
                  type="danger"
                  size="small"
                  @click="deleteConfig"
                >
                  {{ $t('config.delete') }}
                </el-button>
              </div>
            </div>
          </template>

          <div class="editor-container" v-loading="loading">
            <div ref="editorRef" class="monaco-editor"></div>
          </div>

          <div class="editor-footer" v-if="currentFile">
            <el-text type="info" size="small">
              {{ $t('config.autoTestTip') }}
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
import { useI18n } from 'vue-i18n'
import { monaco } from '@/utils/monaco-env'
import api from '@/api'

const { t } = useI18n()

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

// 用于标记当前文件名是否为 i18n key
const currentFileKey = ref<string>('config.selectFile')
const currentFileName = ref('')

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
  currentFileKey.value = '' // nginx.conf 不需要翻译
  currentFileEnabled.value = true

  try {
    const response = await api.get('/api/config/main')
    if (response.data.code === 0 && editor) {
      editor.setValue(response.data.data.content)
      monaco.editor.setModelLanguage(editor.getModel()!, 'nginx')
    }
  } catch (error) {
    ElMessage.error(t('config.readFailed'))
  } finally {
    loading.value = false
  }
}

async function loadSiteConfig(name: string) {
  loading.value = true
  currentFile.value = name
  currentFileName.value = name
  currentFileKey.value = '' // 实际文件名不需要翻译
  currentFileEnabled.value = configFiles.value.find(f => f.name === name)?.enabled || false

  try {
    const response = await api.get(`/api/config/file/${name}`)
    if (response.data.code === 0 && editor) {
      editor.setValue(response.data.data.content)
      monaco.editor.setModelLanguage(editor.getModel()!, 'nginx')
    }
  } catch (error) {
    ElMessage.error(t('config.readFailed'))
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
      ElMessage.success(t('config.configSaved'))
    } else {
      ElMessage.error(response.data.message || t('config.saveFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('config.saveFailed'))
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
      ElMessage.error(response.data.message || t('config.operationFailed'))
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('config.operationFailed'))
  }
}

async function deleteConfig() {
  if (!currentFile.value || currentFile.value === 'main') return

  try {
    await ElMessageBox.confirm(
      t('config.deleteConfirm', { name: currentFile.value }),
      t('common.tip'),
      { type: 'warning' }
    )

    const response = await api.delete(`/api/config/file/${currentFile.value}`)
    if (response.data.code === 0) {
      ElMessage.success(t('config.fileDeleted'))
      currentFile.value = ''
      currentFileKey.value = 'config.selectFile'
      currentFileName.value = ''
      if (editor) {
        editor.setValue('')
      }
      refreshFiles()
    } else {
      ElMessage.error(response.data.message || t('config.deleteFailed'))
    }
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('config.deleteFailed'))
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
