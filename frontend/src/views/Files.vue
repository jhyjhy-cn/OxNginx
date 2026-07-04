<template>
  <div class="file-manager">
    <el-card>
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <!-- 盘符切换 -->
            <el-dropdown v-if="drives.length > 1" @command="handleDriveChange" trigger="click" class="drive-selector">
              <el-button size="small" type="primary" plain>
                <el-icon><Coin /></el-icon>{{ currentDrive }}<el-icon class="el-icon--right"><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item v-for="d in drives" :key="d" :command="d" :class="{ active: d === currentDrive }">{{ d }}</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
            <!-- 面包屑路径 -->
            <div class="path-breadcrumb">
              <span class="path-item clickable" @click="goRoot">
                <el-icon><HomeFilled /></el-icon>
              </span>
              <template v-for="(seg, idx) in pathSegments" :key="idx">
                <span class="path-sep">/</span>
                <span
                  class="path-item clickable"
                  @click="goToSegment(idx)"
                >{{ seg }}</span>
              </template>
            </div>
          </div>
          <div class="header-right">
            <el-button size="small" @click="goBack" :disabled="!currentParent">
              <el-icon><Back /></el-icon>{{ t('files.back') }}
            </el-button>
            <el-button size="small" @click="goRoot">
              <el-icon><HomeFilled /></el-icon>{{ t('files.root') }}
            </el-button>
            <el-button size="small" @click="fetchFiles">
              <el-icon><Refresh /></el-icon>{{ t('files.refresh') }}
            </el-button>
            <el-dropdown @command="handleCreate" trigger="click">
              <el-button size="small" type="primary">
                <el-icon><Plus /></el-icon>{{ t('files.createFile') }}
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="file">{{ t('files.createFile') }}</el-dropdown-item>
                  <el-dropdown-item command="folder">{{ t('files.createFolder') }}</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </template>

      <!-- 文件列表 -->
      <el-table
        :data="pagedFiles"
        style="width: 100%"
        v-loading="loading"
        @row-contextmenu="handleContextMenu"
        @row-dblclick="handleDblClick"
        highlight-current-row
      >
        <el-table-column :label="t('files.name')" min-width="300">
          <template #default="{ row }">
            <div class="file-name-cell">
              <OnIcon :svgName="getFileIcon(row)" :size="18" class="file-icon" />
              <span class="file-name" :class="{ 'is-dir': row.is_dir }">{{ row.name }}</span>
              <el-tag v-if="row.note" size="small" type="info" class="note-tag">{{ row.note }}</el-tag>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="t('files.permissions') + ' / ' + t('files.owner')" width="200">
          <template #default="{ row }">
            <span v-if="row.permissions || row.owner">{{ row.permissions }}<template v-if="row.permissions && row.owner"> / </template>{{ row.owner }}</span>
            <span v-else>-</span>
          </template>
        </el-table-column>
        <el-table-column :label="t('files.size')" width="120">
          <template #default="{ row }">
            <span v-if="!row.is_dir">{{ formatSize(row.size) }}</span>
            <span v-else-if="row._size !== undefined">{{ formatSize(row._size) }}</span>
            <el-button v-else link type="primary" size="small" :loading="row._calcLoading" @click="calcFileSize(row)">计算</el-button>
          </template>
        </el-table-column>
        <el-table-column prop="modified" :label="t('files.modified')" width="180" />
        <el-table-column :label="t('files.note')" min-width="180">
          <template #default="{ row }">
            <div
              class="note-cell"
              @mouseenter="handleNoteEnter(row)"
              @mouseleave="handleNoteLeave(row)"
            >
              <template v-if="hoverNotePath === row.path">
                <el-input
                  v-model="editingNote"
                  size="small"
                  :placeholder="t('files.notePlaceholder')"
                  @keyup.enter="saveInlineNote(row)"
                  @blur="saveInlineNote(row)"
                  autofocus
                />
              </template>
              <template v-else>
                <span class="note-text" :class="{ empty: !row.note }">{{ row.note || '-' }}</span>
              </template>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.action')" width="180" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="handleRename(row)">{{ t('files.rename') }}</el-button>
            <el-button link type="danger" size="small" @click="handleDelete(row)">{{ t('files.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-wrap">
        <div class="pagination-left">
          <span>{{ t('files.totalItems', { n: files.length }) }}</span>
          <span class="stat-sep">|</span>
          <span>{{ dirCount }} 个目录</span>
          <span class="stat-sep">|</span>
          <span>{{ fileCount }} 个文件</span>
          <span class="stat-sep">|</span>
          <span>大小: <template v-if="totalSize !== null">{{ formatSize(totalSize) }}</template><el-button v-else link type="primary" size="small" :loading="calcTotalLoading" @click="calcTotalSize">计算</el-button></span>
        </div>
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[100, 500, 1000, 1500, 2000]"
          :total="files.length"
          layout="sizes, prev, pager, next, jumper"
          background
        />
      </div>
    </el-card>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click="contextMenu.visible = false"
    >
      <div v-if="contextMenu.row?.is_dir" class="ctx-item" @click="enterDir(contextMenu.row)">
        <el-icon><FolderOpened /></el-icon>{{ t('files.open') || '打开' }}
      </div>
      <div v-if="!contextMenu.row?.is_dir" class="ctx-item" @click="handleEdit(contextMenu.row!)">
        <el-icon><Edit /></el-icon>{{ t('files.edit') }}
      </div>
      <div class="ctx-item" @click="handleRename(contextMenu.row!)">
        <el-icon><EditPen /></el-icon>{{ t('files.rename') }}
      </div>
      <div class="ctx-item" @click="handleCopy(contextMenu.row!)">
        <el-icon><CopyDocument /></el-icon>{{ t('files.copy') }}
      </div>
      <div class="ctx-item" @click="handleMove(contextMenu.row!)">
        <el-icon><Rank /></el-icon>{{ t('files.move') }}
      </div>
      <div class="ctx-divider"></div>
      <div class="ctx-item" @click="handleCompressSingle(contextMenu.row!)">
        <el-icon><FolderAdd /></el-icon>{{ t('files.compress') }}
      </div>
      <div v-if="isArchive(contextMenu.row?.name)" class="ctx-item" @click="handleExtract(contextMenu.row!)">
        <el-icon><FolderRemove /></el-icon>{{ t('files.extract') }}
      </div>
      <div class="ctx-item" @click="handleChmod(contextMenu.row!)">
        <el-icon><Lock /></el-icon>{{ t('files.changePermission') }}
      </div>
      <div class="ctx-item" @click="handleEditNote(contextMenu.row!)">
        <el-icon><Memo /></el-icon>{{ t('files.editNote') }}
      </div>
      <div class="ctx-divider"></div>
      <div class="ctx-item danger" @click="handleDelete(contextMenu.row!)">
        <el-icon><Delete /></el-icon>{{ t('files.delete') }}
      </div>
    </div>

    <!-- 新建文件/文件夹弹窗 -->
    <OnDialog v-model="createDialog.visible" :title="createDialog.isDir ? t('files.createFolder') : t('files.createFile')" width="420px">
      <el-form :model="createDialog" label-width="80px">
        <el-form-item :label="createDialog.isDir ? t('files.folderName') : t('files.fileName')">
          <el-input
            v-model="createDialog.name"
            :placeholder="createDialog.isDir ? t('files.enterFolderName') : t('files.enterFileName')"
            @keyup.enter="submitCreate"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitCreate">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 重命名弹窗 -->
    <OnDialog v-model="renameDialog.visible" :title="t('files.rename')" width="420px">
      <el-form :model="renameDialog" label-width="80px">
        <el-form-item :label="t('files.name')">
          <el-input v-model="renameDialog.newName" :placeholder="t('files.enterNewName')" @keyup.enter="submitRename" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="renameDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitRename">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 复制/移动弹窗 -->
    <OnDialog v-model="moveDialog.visible" :title="moveDialog.isCopy ? t('files.copyTo') : t('files.moveTo')" width="500px">
      <el-form label-width="80px">
        <el-form-item :label="t('files.destination')">
          <el-input v-model="moveDialog.destination" :placeholder="t('files.enterDestination')" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="moveDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitMove">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 压缩弹窗 -->
    <OnDialog v-model="compressDialog.visible" :title="t('files.compress')" width="460px">
      <el-form label-width="80px">
        <el-form-item :label="t('files.compressName')">
          <el-input v-model="compressDialog.name" :placeholder="t('files.enterCompressName')" />
        </el-form-item>
        <el-form-item :label="t('files.compressFormat')">
          <el-radio-group v-model="compressDialog.format">
            <el-radio value="zip">.zip</el-radio>
            <el-radio value="tar.gz">.tar.gz</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="compressDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitCompress">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 权限弹窗 -->
    <OnDialog v-model="chmodDialog.visible" :title="t('files.changePermission')" width="400px">
      <el-form label-width="80px">
        <el-form-item :label="t('files.permissionMode')">
          <el-input v-model="chmodDialog.mode" :placeholder="t('files.enterPermission')" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="chmodDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitChmod">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 备注弹窗 -->
    <OnDialog v-model="noteDialog.visible" :title="t('files.editNote')" width="460px">
      <el-input
        v-model="noteDialog.note"
        type="textarea"
        :rows="3"
        :placeholder="t('files.notePlaceholder')"
      />
      <template #footer>
        <el-button @click="noteDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitNote">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 编辑文件弹窗 -->
    <OnDialog v-model="editDialog.visible" :title="t('files.editFile')" width="80%" maximizable>
      <div class="editor-wrapper">
        <textarea
          v-model="editDialog.content"
          class="file-editor"
          spellcheck="false"
        />
      </div>
      <template #footer>
        <el-button @click="editDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitEdit">{{ t('files.saveFile') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'
import OnIcon from '@/components/OnIcon/index.vue'

const { t } = useI18n()

interface FileItem {
  name: string
  path: string
  is_dir: boolean
  size: number
  permissions: string
  owner: string
  modified: string
  extension: string
  note: string | null
}

const loading = ref(false)
const currentPath = ref('')
const currentParent = ref<string | null>(null)
const files = ref<FileItem[]>([])

// 分页
const currentPage = ref(1)
const pageSize = ref(100)
const pagedFiles = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return files.value.slice(start, start + pageSize.value)
})

// 统计
const dirCount = computed(() => files.value.filter(f => f.is_dir).length)
const fileCount = computed(() => files.value.filter(f => !f.is_dir).length)
const totalSize = ref<number | null>(null)
const calcTotalLoading = ref(false)

// 盘符（Windows）
const drives = ref<string[]>([])
const currentDrive = computed(() => {
  const p = currentPath.value.replace(/\\/g, '/')
  const match = p.match(/^([A-Za-z]:)/)
  return match ? match[1] : '/'
})

// 路径面包屑分段
const pathSegments = computed(() => {
  if (!currentPath.value || currentPath.value === '/') return []
  // 统一用 / 分割
  return currentPath.value
    .replace(/\\/g, '/')
    .split('/')
    .filter(Boolean)
})

// 右键菜单
const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  row: null as FileItem | null,
})

// 弹窗状态
const createDialog = reactive({ visible: false, isDir: false, name: '' })
const renameDialog = reactive({ visible: false, path: '', newName: '' })
const moveDialog = reactive({ visible: false, source: '', destination: '', isCopy: false })
const compressDialog = reactive({ visible: false, paths: [] as string[], name: '', format: 'zip' })
const chmodDialog = reactive({ visible: false, path: '', mode: '' })
const noteDialog = reactive({ visible: false, path: '', note: '' })
const hoverNotePath = ref('')
const editingNote = ref('')
const editDialog = reactive({ visible: false, path: '', content: '' })

onMounted(() => {
  fetchDrives()
  fetchFiles()
  // 点击空白处关闭右键菜单
  document.addEventListener('click', closeContextMenu)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', closeContextMenu)
})

function closeContextMenu() {
  contextMenu.visible = false
}

/** 获取文件列表 */
async function fetchFiles() {
  loading.value = true
  totalSize.value = null
  try {
    const { data } = await api.get('/api/files/list', { params: { path: currentPath.value } })
    if (data.code === 0) {
      files.value = data.data.items
      currentPath.value = data.data.path
      currentParent.value = data.data.parent
      currentPage.value = 1
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('files.readError'))
  } finally {
    loading.value = false
  }
}

/** 获取盘符列表 */
async function fetchDrives() {
  try {
    const { data } = await api.get('/api/files/roots')
    if (data.code === 0) {
      drives.value = data.data
    }
  } catch {
    // 非 Windows 环境下可能无此接口
  }
}

/** 切换盘符 */
function handleDriveChange(drive: string) {
  currentPath.value = drive
  fetchFiles()
}

/** 计算当前目录总大小 */
async function calcTotalSize() {
  calcTotalLoading.value = true
  try {
    const { data } = await api.get('/api/files/size', { params: { path: currentPath.value } })
    if (data.code === 0) {
      totalSize.value = data.data.size
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  } finally {
    calcTotalLoading.value = false
  }
}

/** 计算单个文件大小 */
async function calcFileSize(row: FileItem & { _size?: number; _calcLoading?: boolean }) {
  row._calcLoading = true
  try {
    const { data } = await api.get('/api/files/size', { params: { path: row.path } })
    if (data.code === 0) {
      row._size = data.data.size
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  } finally {
    row._calcLoading = false
  }
}

/** 格式化文件大小 */
function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

/** 判断是否压缩文件 */
function isArchive(name?: string): boolean {
  if (!name) return false
  return /\.(zip|tar\.gz|tgz|tar|gz|rar|7z)$/i.test(name)
}

/** 根据文件类型返回图标名 */
const extIconMap: Record<string, string> = {
  pdf: 'pdf',
  doc: 'word', docx: 'word',
  ppt: 'ppt', pptx: 'ppt',
  xls: 'excel', xlsx: 'excel',
  js: 'js', mjs: 'js',
  json: 'json',
  java: 'java',
  c: 'c', h: 'c',
  cpp: 'cpp', cc: 'cpp', cxx: 'cpp', hpp: 'cpp',
  py: 'python',
}

function getFileIcon(row: FileItem): string {
  if (row.is_dir) return 'folder'
  const ext = row.extension.toLowerCase()
  return extIconMap[ext] || 'file'
}

/** 导航 */
function goRoot() {
  currentPath.value = ''
  fetchFiles()
}

function goBack() {
  if (currentParent.value) {
    currentPath.value = currentParent.value
    fetchFiles()
  }
}

function goToSegment(index: number) {
  const parts = currentPath.value.replace(/\\/g, '/').split('/').filter(Boolean)
  // 重建路径
  const sep = currentPath.value.includes('\\') ? '\\' : '/'
  currentPath.value = parts.slice(0, index + 1).join(sep)
  // Windows 盘符处理
  if (/^[A-Za-z]:$/.test(parts[0])) {
    currentPath.value = parts[0] + sep + parts.slice(1, index + 1).join(sep)
  }
  fetchFiles()
}

/** 双击进入目录/编辑文件 */
function handleDblClick(row: FileItem) {
  if (row.is_dir) {
    enterDir(row)
  } else {
    handleEdit(row)
  }
}

function enterDir(row: FileItem) {
  currentPath.value = row.path
  fetchFiles()
}

/** 右键菜单 */
function handleContextMenu(row: FileItem, event: MouseEvent) {
  event.preventDefault()
  contextMenu.visible = true
  contextMenu.x = event.clientX
  contextMenu.y = event.clientY
  contextMenu.row = row
}

/** 新建 */
function handleCreate(cmd: string) {
  createDialog.isDir = cmd === 'folder'
  createDialog.name = ''
  createDialog.visible = true
}

async function submitCreate() {
  if (!createDialog.name) {
    ElMessage.warning(createDialog.isDir ? t('files.enterFolderName') : t('files.enterFileName'))
    return
  }
  const url = createDialog.isDir ? '/api/files/mkdir' : '/api/files/touch'
  try {
    const { data } = await api.post(url, { path: currentPath.value, name: createDialog.name })
    if (data.code === 0) {
      ElMessage.success(t('files.createSuccess'))
      createDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}

/** 重命名 */
function handleRename(row: FileItem) {
  renameDialog.path = row.path
  renameDialog.newName = row.name
  renameDialog.visible = true
  contextMenu.visible = false
}

async function submitRename() {
  if (!renameDialog.newName) {
    ElMessage.warning(t('files.enterNewName'))
    return
  }
  try {
    const { data } = await api.post('/api/files/rename', { path: renameDialog.path, new_name: renameDialog.newName })
    if (data.code === 0) {
      ElMessage.success(t('files.renameSuccess'))
      renameDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}

/** 编辑文件 */
async function handleEdit(row: FileItem) {
  if (row.is_dir) return
  contextMenu.visible = false
  // 大文件提示
  if (row.size > 5 * 1024 * 1024) {
    ElMessage.warning(t('files.fileTooLarge'))
    return
  }
  try {
    const { data } = await api.get('/api/files/read', { params: { path: row.path } })
    if (data.code === 0) {
      editDialog.path = row.path
      editDialog.content = data.data.content
      editDialog.visible = true
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('files.readError'))
  }
}

async function submitEdit() {
  try {
    const { data } = await api.post('/api/files/write', { path: editDialog.path, content: editDialog.content })
    if (data.code === 0) {
      ElMessage.success(t('files.saveSuccess'))
      editDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}

/** 复制 */
function handleCopy(row: FileItem) {
  moveDialog.source = row.path
  moveDialog.destination = row.path
  moveDialog.isCopy = true
  moveDialog.visible = true
  contextMenu.visible = false
}

/** 移动 */
function handleMove(row: FileItem) {
  moveDialog.source = row.path
  moveDialog.destination = row.path
  moveDialog.isCopy = false
  moveDialog.visible = true
  contextMenu.visible = false
}

async function submitMove() {
  const url = moveDialog.isCopy ? '/api/files/copy' : '/api/files/move'
  try {
    const { data } = await api.post(url, { source: moveDialog.source, destination: moveDialog.destination })
    if (data.code === 0) {
      ElMessage.success(moveDialog.isCopy ? t('files.copySuccess') : t('files.moveSuccess'))
      moveDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}

/** 删除 */
async function handleDelete(row: FileItem) {
  contextMenu.visible = false
  try {
    await ElMessageBox.confirm(
      t('files.confirmDelete', { name: row.name }),
      t('common.warning'),
      { type: 'warning' },
    )
    const { data } = await api.delete('/api/files/delete', { data: { path: row.path } })
    if (data.code === 0) {
      ElMessage.success(t('files.deleteSuccess'))
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    // 取消
  }
}

/** 压缩（单个） */
function handleCompressSingle(row: FileItem) {
  compressDialog.paths = [row.path]
  compressDialog.name = row.name + '.zip'
  compressDialog.format = 'zip'
  compressDialog.visible = true
  contextMenu.visible = false
}

async function submitCompress() {
  if (!compressDialog.name) {
    ElMessage.warning(t('files.enterCompressName'))
    return
  }
  const dest = currentPath.value.replace(/[\\/]+$/, '') + '/' + compressDialog.name
  try {
    const { data } = await api.post('/api/files/compress', {
      paths: compressDialog.paths,
      destination: dest,
      format: compressDialog.format,
    })
    if (data.code === 0) {
      ElMessage.success(t('files.compressSuccess'))
      compressDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}

/** 解压 */
function handleExtract(row: FileItem) {
  const dest = currentPath.value.replace(/[\\/]+$/, '') + '/' + row.name.replace(/\.(zip|tar\.gz|tgz)$/i, '')
  api.post('/api/files/extract', { path: row.path, destination: dest }).then(({ data }) => {
    if (data.code === 0) {
      ElMessage.success(t('files.extractSuccess'))
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  }).catch(() => ElMessage.error(t('common.operationFailed')))
  contextMenu.visible = false
}

/** 权限 */
function handleChmod(row: FileItem) {
  chmodDialog.path = row.path
  // 从 permissions 字段提取八进制（如 drwxr-xr-x → 755）
  const perm = row.permissions
  if (perm.length === 10) {
    const owner = (perm[1] === 'r' ? 4 : 0) + (perm[2] === 'w' ? 2 : 0) + (perm[3] === 'x' ? 1 : 0)
    const group = (perm[4] === 'r' ? 4 : 0) + (perm[5] === 'w' ? 2 : 0) + (perm[6] === 'x' ? 1 : 0)
    const other = (perm[7] === 'r' ? 4 : 0) + (perm[8] === 'w' ? 2 : 0) + (perm[9] === 'x' ? 1 : 0)
    chmodDialog.mode = `${owner}${group}${other}`
  } else {
    chmodDialog.mode = '644'
  }
  chmodDialog.visible = true
  contextMenu.visible = false
}

async function submitChmod() {
  try {
    const { data } = await api.post('/api/files/chmod', { path: chmodDialog.path, mode: chmodDialog.mode })
    if (data.code === 0) {
      ElMessage.success(t('files.permissionSuccess'))
      chmodDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}

/** 备注 */
function handleEditNote(row: FileItem) {
  noteDialog.path = row.path
  noteDialog.note = row.note || ''
  noteDialog.visible = true
  contextMenu.visible = false
}

/** 鼠标进入备注列时初始化编辑值 */
function handleNoteEnter(row: FileItem) {
  hoverNotePath.value = row.path
  editingNote.value = row.note || ''
}

/** 鼠标离开备注列时隐藏输入框 */
function handleNoteLeave(row: FileItem) {
  // 如果值有变化，先保存
  if (editingNote.value !== (row.note || '')) {
    saveInlineNote(row)
  }
  hoverNotePath.value = ''
}

/** 内联保存备注 */
async function saveInlineNote(row: FileItem) {
  if (editingNote.value === (row.note || '')) {
    hoverNotePath.value = ''
    return
  }
  try {
    const { data } = await api.post('/api/files/note', { path: row.path, note: editingNote.value })
    if (data.code === 0) {
      row.note = editingNote.value || null
    }
  } catch {
    // 静默失败
  }
  hoverNotePath.value = ''
}

async function submitNote() {
  try {
    const { data } = await api.post('/api/files/note', { path: noteDialog.path, note: noteDialog.note })
    if (data.code === 0) {
      ElMessage.success(t('files.noteSuccess'))
      noteDialog.visible = false
      fetchFiles()
    } else {
      ElMessage.error(data.message)
    }
  } catch {
    ElMessage.error(t('common.operationFailed'))
  }
}
</script>

<style scoped>
.file-manager {
  height: 100%;
}

.file-manager :deep(.el-card) {
  height: calc(100vh - 140px);
  display: flex;
  flex-direction: column;
}

.file-manager :deep(.el-card__body) {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.file-manager :deep(.el-table) {
  flex: 1;
  overflow: auto;
}

.pagination-wrap {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0 0;
  flex-shrink: 0;
}

.pagination-left {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}

.stat-sep {
  color: var(--el-border-color);
}

/* 备注列 */
.note-cell {
  min-height: 24px;
  display: flex;
  align-items: center;
}

.note-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: default;
}

.note-text.empty {
  color: var(--el-text-color-placeholder);
}

.drive-selector {
  margin-right: 8px;
}

:deep(.el-dropdown-menu__item.active) {
  color: var(--el-color-primary);
  font-weight: 600;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
}

.header-left {
  display: flex;
  align-items: center;
  min-width: 0;
  flex: 1;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.path-breadcrumb {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 2px;
  font-size: 14px;
  min-width: 0;
}

.path-item {
  display: inline-flex;
  align-items: center;
  padding: 2px 4px;
  border-radius: 3px;
  white-space: nowrap;
}

.path-item.clickable {
  cursor: pointer;
  color: var(--el-color-primary);
}

.path-item.clickable:hover {
  background: var(--el-color-primary-light-9);
}

.path-sep {
  color: var(--el-text-color-placeholder);
  margin: 0 1px;
}

/* 文件名列 */
.file-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-icon {
  flex-shrink: 0;
  color: var(--el-text-color-secondary);
}

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-name.is-dir {
  color: var(--el-color-primary);
  cursor: pointer;
}

.note-tag {
  flex-shrink: 0;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  padding: 4px 0;
  min-width: 160px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  cursor: pointer;
  color: var(--el-text-color-regular);
  transition: background 0.15s;
}

.ctx-item:hover {
  background: var(--el-fill-color-light);
}

.ctx-item.danger {
  color: var(--el-color-danger);
}

.ctx-divider {
  height: 1px;
  background: var(--el-border-color-lighter);
  margin: 4px 0;
}

/* 编辑器 */
.editor-wrapper {
  height: 60vh;
}

.file-editor {
  width: 100%;
  height: 100%;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  padding: 12px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  outline: none;
  background: var(--el-bg-color);
  color: var(--el-text-color-primary);
  tab-size: 4;
}

.file-editor:focus {
  border-color: var(--el-color-primary);
}
</style>
