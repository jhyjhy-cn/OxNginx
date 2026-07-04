<template>
  <div class="file-manager">
    <!-- 路径栏 -->
    <div class="fm-pathbar">
      <div class="pathbar-left">
        <el-button :icon="Back" size="small" @click="goBack" :disabled="!currentParent" />
        <el-input
          v-model="inputPath"
          size="small"
          class="path-input"
          :placeholder="currentPath"
          @keyup.enter="goToInputPath"
          @focus="inputPath = currentPath"
        >
          <template #prefix>
            <el-dropdown v-if="drives.length > 1" @command="handleDriveChange" trigger="click">
              <span class="drive-prefix">{{ currentDrive }}</span>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item v-for="d in drives" :key="d" :command="d" :class="{ active: d === currentDrive }">{{ d }}</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
        </el-input>
        <el-button :icon="Refresh" size="small" @click="fetchFiles" />
      </div>
      <div class="pathbar-right">
        <el-input
          v-model="searchQuery"
          size="small"
          class="search-input"
          :placeholder="t('files.search') || '搜索文件...'"
          clearable
          :prefix-icon="Search"
        />
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="fm-actionbar">
      <div class="actionbar-left">
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
      <div class="actionbar-right">
        <el-button size="small" disabled>
          <el-icon><Delete /></el-icon><span class="btn-text">回收站</span>
        </el-button>
        <el-button-group class="view-toggle">
          <el-button size="small" :type="viewMode === 'list' ? 'primary' : ''" @click="viewMode = 'list'">
            <el-icon><List /></el-icon>
          </el-button>
          <el-button size="small" :type="viewMode === 'card' ? 'primary' : ''" @click="viewMode = 'card'">
            <el-icon><Grid /></el-icon>
          </el-button>
        </el-button-group>
      </div>
    </div>

    <!-- 内容区域 -->
    <div class="fm-content" v-loading="loading">
      <!-- 列表视图 -->
      <el-table
        v-if="viewMode === 'list'"
        :data="filteredPagedFiles"
        style="width: 100%"
        @row-contextmenu="handleContextMenu"
        @row-dblclick="handleDblClick"
        highlight-current-row
        height="100%"
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
            <div class="note-cell" @mouseenter="handleNoteEnter(row)" @mouseleave="handleNoteLeave(row)">
              <template v-if="hoverNotePath === row.path">
                <el-input v-model="editingNote" size="small" :placeholder="t('files.notePlaceholder')" @keyup.enter="saveInlineNote(row)" @blur="saveInlineNote(row)" autofocus />
              </template>
              <template v-else>
                <span class="note-text" :class="{ empty: !row.note }">{{ row.note || '-' }}</span>
              </template>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="t('common.action')" width="150" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="handleRename(row)">{{ t('files.rename') }}</el-button>
            <el-button link type="danger" size="small" @click="handleDelete(row)">{{ t('files.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 卡片视图 -->
      <div v-else class="card-grid">
        <div
          v-for="row in filteredPagedFiles"
          :key="row.path"
          class="file-card"
          @dblclick="handleDblClick(row)"
          @contextmenu.prevent="handleContextMenu(row, $event)"
        >
          <div class="card-icon">
            <OnIcon :svgName="getFileIcon(row)" :size="40" />
          </div>
          <div class="card-name" :title="row.name">{{ row.name }}</div>
          <div class="card-meta">
            <span v-if="!row.is_dir">{{ formatSize(row.size) }}</span>
            <span v-else-if="row._size !== undefined">{{ formatSize(row._size) }}</span>
            <span v-else class="calc-link" @click.stop="calcFileSize(row)">计算</span>
          </div>
        </div>
        <div v-if="filteredPagedFiles.length === 0 && !loading" class="empty-tip">{{ t('files.emptyDir') }}</div>
      </div>
    </div>

    <!-- 分页栏 -->
    <div class="fm-pagination">
      <div class="pagination-left">
        <span>{{ t('files.totalItems', { n: filteredFiles.length }) }}</span>
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
        :total="filteredFiles.length"
        layout="sizes, prev, pager, next, jumper"
        background
        small
      />
    </div>

    <!-- 右键菜单 -->
    <div v-if="contextMenu.visible" class="context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click="contextMenu.visible = false">
      <div v-if="contextMenu.row?.is_dir" class="ctx-item" @click="enterDir(contextMenu.row)"><el-icon><FolderOpened /></el-icon>打开</div>
      <div v-if="!contextMenu.row?.is_dir" class="ctx-item" @click="handleEdit(contextMenu.row!)"><el-icon><Edit /></el-icon>{{ t('files.edit') }}</div>
      <div class="ctx-item" @click="handleRename(contextMenu.row!)"><el-icon><EditPen /></el-icon>{{ t('files.rename') }}</div>
      <div class="ctx-item" @click="handleCopy(contextMenu.row!)"><el-icon><CopyDocument /></el-icon>{{ t('files.copy') }}</div>
      <div class="ctx-item" @click="handleMove(contextMenu.row!)"><el-icon><Rank /></el-icon>{{ t('files.move') }}</div>
      <div class="ctx-divider"></div>
      <div class="ctx-item" @click="handleCompressSingle(contextMenu.row!)"><el-icon><FolderAdd /></el-icon>{{ t('files.compress') }}</div>
      <div v-if="isArchive(contextMenu.row?.name)" class="ctx-item" @click="handleExtract(contextMenu.row!)"><el-icon><FolderRemove /></el-icon>{{ t('files.extract') }}</div>
      <div class="ctx-item" @click="handleChmod(contextMenu.row!)"><el-icon><Lock /></el-icon>{{ t('files.changePermission') }}</div>
      <div class="ctx-divider"></div>
      <div class="ctx-item danger" @click="handleDelete(contextMenu.row!)"><el-icon><Delete /></el-icon>{{ t('files.delete') }}</div>
    </div>

    <!-- 弹窗们 -->
    <OnDialog v-model="createDialog.visible" :title="createDialog.isDir ? t('files.createFolder') : t('files.createFile')" width="420px">
      <el-form :model="createDialog" label-width="80px">
        <el-form-item :label="createDialog.isDir ? t('files.folderName') : t('files.fileName')">
          <el-input v-model="createDialog.name" :placeholder="createDialog.isDir ? t('files.enterFolderName') : t('files.enterFileName')" @keyup.enter="submitCreate" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createDialog.visible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submitCreate">{{ t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

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

    <OnDialog v-model="editDialog.visible" :title="t('files.editFile')" width="80%" maximizable>
      <div class="editor-wrapper">
        <textarea v-model="editDialog.content" class="file-editor" spellcheck="false" />
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
import { Back, Refresh, Search, Plus, List, Grid } from '@element-plus/icons-vue'
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
  _size?: number
  _calcLoading?: boolean
}

const loading = ref(false)
const currentPath = ref('')
const currentParent = ref<string | null>(null)
const files = ref<FileItem[]>([])
const inputPath = ref('')
const searchQuery = ref('')
const viewMode = ref<'list' | 'card'>('list')

// 搜索过滤
const filteredFiles = computed(() => {
  if (!searchQuery.value) return files.value
  const q = searchQuery.value.toLowerCase()
  return files.value.filter(f => f.name.toLowerCase().includes(q))
})

// 分页
const currentPage = ref(1)
const pageSize = ref(100)
const filteredPagedFiles = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredFiles.value.slice(start, start + pageSize.value)
})

// 统计
const dirCount = computed(() => filteredFiles.value.filter(f => f.is_dir).length)
const fileCount = computed(() => filteredFiles.value.filter(f => !f.is_dir).length)
const totalSize = ref<number | null>(null)
const calcTotalLoading = ref(false)

// 盘符
const drives = ref<string[]>([])
const currentDrive = computed(() => {
  const p = currentPath.value.replace(/\\/g, '/')
  const match = p.match(/^([A-Za-z]:)/)
  return match ? match[1] : '/'
})

// 右键菜单
const contextMenu = reactive({ visible: false, x: 0, y: 0, row: null as FileItem | null })

// 弹窗状态
const createDialog = reactive({ visible: false, isDir: false, name: '' })
const renameDialog = reactive({ visible: false, path: '', newName: '' })
const moveDialog = reactive({ visible: false, source: '', destination: '', isCopy: false })
const compressDialog = reactive({ visible: false, paths: [] as string[], name: '', format: 'zip' })
const chmodDialog = reactive({ visible: false, path: '', mode: '' })
const hoverNotePath = ref('')
const editingNote = ref('')
const editDialog = reactive({ visible: false, path: '', content: '' })

onMounted(() => {
  fetchDrives()
  fetchFiles()
  document.addEventListener('click', closeContextMenu)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', closeContextMenu)
})

function closeContextMenu() { contextMenu.visible = false }

async function fetchFiles() {
  loading.value = true
  totalSize.value = null
  searchQuery.value = ''
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

async function fetchDrives() {
  try {
    const { data } = await api.get('/api/files/roots')
    if (data.code === 0) drives.value = data.data
  } catch { /* 非 Windows */ }
}

function handleDriveChange(drive: string) { currentPath.value = drive; fetchFiles() }

async function calcTotalSize() {
  calcTotalLoading.value = true
  try {
    const { data } = await api.get('/api/files/size', { params: { path: currentPath.value } })
    if (data.code === 0) totalSize.value = data.data.size
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
  finally { calcTotalLoading.value = false }
}

async function calcFileSize(row: FileItem) {
  row._calcLoading = true
  try {
    const { data } = await api.get('/api/files/size', { params: { path: row.path } })
    if (data.code === 0) row._size = data.data.size
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
  finally { row._calcLoading = false }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i]
}

function isArchive(name?: string): boolean {
  if (!name) return false
  return /\.(zip|tar\.gz|tgz|tar|gz|rar|7z)$/i.test(name)
}

const extIconMap: Record<string, string> = {
  pdf: 'pdf', doc: 'word', docx: 'word', ppt: 'ppt', pptx: 'ppt',
  xls: 'excel', xlsx: 'excel', js: 'js', mjs: 'js', json: 'json',
  java: 'java', c: 'c', h: 'c', cpp: 'cpp', cc: 'cpp', cxx: 'cpp', hpp: 'cpp', py: 'python',
}

function getFileIcon(row: FileItem): string {
  if (row.is_dir) return 'folder'
  return extIconMap[row.extension.toLowerCase()] || 'file'
}

// 导航
function goBack() { if (currentParent.value) { currentPath.value = currentParent.value; fetchFiles() } }
function goToInputPath() { if (inputPath.value) { currentPath.value = inputPath.value; inputPath.value = ''; fetchFiles() } }

function handleDblClick(row: FileItem) { row.is_dir ? enterDir(row) : handleEdit(row) }
function enterDir(row: FileItem) { currentPath.value = row.path; fetchFiles() }

function handleContextMenu(row: FileItem, event: MouseEvent) {
  event.preventDefault()
  contextMenu.visible = true
  contextMenu.x = event.clientX
  contextMenu.y = event.clientY
  contextMenu.row = row
}

// CRUD
function handleCreate(cmd: string) { createDialog.isDir = cmd === 'folder'; createDialog.name = ''; createDialog.visible = true }

async function submitCreate() {
  if (!createDialog.name) { ElMessage.warning(createDialog.isDir ? t('files.enterFolderName') : t('files.enterFileName')); return }
  const url = createDialog.isDir ? '/api/files/mkdir' : '/api/files/touch'
  try {
    const { data } = await api.post(url, { path: currentPath.value, name: createDialog.name })
    if (data.code === 0) { ElMessage.success(t('files.createSuccess')); createDialog.visible = false; fetchFiles() }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
}

function handleRename(row: FileItem) { renameDialog.path = row.path; renameDialog.newName = row.name; renameDialog.visible = true; contextMenu.visible = false }

async function submitRename() {
  if (!renameDialog.newName) { ElMessage.warning(t('files.enterNewName')); return }
  try {
    const { data } = await api.post('/api/files/rename', { path: renameDialog.path, new_name: renameDialog.newName })
    if (data.code === 0) { ElMessage.success(t('files.renameSuccess')); renameDialog.visible = false; fetchFiles() }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
}

async function handleEdit(row: FileItem) {
  if (row.is_dir) return
  contextMenu.visible = false
  if (row.size > 5 * 1024 * 1024) { ElMessage.warning(t('files.fileTooLarge')); return }
  try {
    const { data } = await api.get('/api/files/read', { params: { path: row.path } })
    if (data.code === 0) { editDialog.path = row.path; editDialog.content = data.data.content; editDialog.visible = true }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('files.readError')) }
}

async function submitEdit() {
  try {
    const { data } = await api.post('/api/files/write', { path: editDialog.path, content: editDialog.content })
    if (data.code === 0) { ElMessage.success(t('files.saveSuccess')); editDialog.visible = false; fetchFiles() }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
}

function handleCopy(row: FileItem) { moveDialog.source = row.path; moveDialog.destination = row.path; moveDialog.isCopy = true; moveDialog.visible = true; contextMenu.visible = false }
function handleMove(row: FileItem) { moveDialog.source = row.path; moveDialog.destination = row.path; moveDialog.isCopy = false; moveDialog.visible = true; contextMenu.visible = false }

async function submitMove() {
  const url = moveDialog.isCopy ? '/api/files/copy' : '/api/files/move'
  try {
    const { data } = await api.post(url, { source: moveDialog.source, destination: moveDialog.destination })
    if (data.code === 0) { ElMessage.success(moveDialog.isCopy ? t('files.copySuccess') : t('files.moveSuccess')); moveDialog.visible = false; fetchFiles() }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
}

async function handleDelete(row: FileItem) {
  contextMenu.visible = false
  try {
    await ElMessageBox.confirm(t('files.confirmDelete', { name: row.name }), t('common.warning'), { type: 'warning' })
    const { data } = await api.delete('/api/files/delete', { data: { path: row.path } })
    if (data.code === 0) { ElMessage.success(t('files.deleteSuccess')); fetchFiles() }
    else ElMessage.error(data.message)
  } catch { /* 取消 */ }
}

function handleCompressSingle(row: FileItem) { compressDialog.paths = [row.path]; compressDialog.name = row.name + '.zip'; compressDialog.format = 'zip'; compressDialog.visible = true; contextMenu.visible = false }

async function submitCompress() {
  if (!compressDialog.name) { ElMessage.warning(t('files.enterCompressName')); return }
  const dest = currentPath.value.replace(/[\\/]+$/, '') + '/' + compressDialog.name
  try {
    const { data } = await api.post('/api/files/compress', { paths: compressDialog.paths, destination: dest, format: compressDialog.format })
    if (data.code === 0) { ElMessage.success(t('files.compressSuccess')); compressDialog.visible = false; fetchFiles() }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
}

function handleExtract(row: FileItem) {
  const dest = currentPath.value.replace(/[\\/]+$/, '') + '/' + row.name.replace(/\.(zip|tar\.gz|tgz)$/i, '')
  api.post('/api/files/extract', { path: row.path, destination: dest }).then(({ data }) => {
    if (data.code === 0) { ElMessage.success(t('files.extractSuccess')); fetchFiles() }
    else ElMessage.error(data.message)
  }).catch(() => ElMessage.error(t('common.operationFailed')))
  contextMenu.visible = false
}

function handleChmod(row: FileItem) {
  chmodDialog.path = row.path
  const perm = row.permissions
  if (perm.length === 10) {
    const o = (perm[1] === 'r' ? 4 : 0) + (perm[2] === 'w' ? 2 : 0) + (perm[3] === 'x' ? 1 : 0)
    const g = (perm[4] === 'r' ? 4 : 0) + (perm[5] === 'w' ? 2 : 0) + (perm[6] === 'x' ? 1 : 0)
    const w = (perm[7] === 'r' ? 4 : 0) + (perm[8] === 'w' ? 2 : 0) + (perm[9] === 'x' ? 1 : 0)
    chmodDialog.mode = `${o}${g}${w}`
  } else { chmodDialog.mode = '644' }
  chmodDialog.visible = true; contextMenu.visible = false
}

async function submitChmod() {
  try {
    const { data } = await api.post('/api/files/chmod', { path: chmodDialog.path, mode: chmodDialog.mode })
    if (data.code === 0) { ElMessage.success(t('files.permissionSuccess')); chmodDialog.visible = false; fetchFiles() }
    else ElMessage.error(data.message)
  } catch { ElMessage.error(t('common.operationFailed')) }
}

// 备注
function handleNoteEnter(row: FileItem) { hoverNotePath.value = row.path; editingNote.value = row.note || '' }
function handleNoteLeave(row: FileItem) { if (editingNote.value !== (row.note || '')) saveInlineNote(row); hoverNotePath.value = '' }
async function saveInlineNote(row: FileItem) {
  if (editingNote.value === (row.note || '')) { hoverNotePath.value = ''; return }
  try {
    const { data } = await api.post('/api/files/note', { path: row.path, note: editingNote.value })
    if (data.code === 0) row.note = editingNote.value || null
  } catch { /* 静默 */ }
  hoverNotePath.value = ''
}
</script>

<style scoped>
.file-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* 路径栏 */
.fm-pathbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}

.pathbar-left {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.path-input {
  flex: 1;
  min-width: 200px;
}

.drive-prefix {
  cursor: pointer;
  font-size: 12px;
  color: var(--el-color-primary);
  margin-right: 2px;
}

.pathbar-right {
  flex-shrink: 0;
}

.search-input {
  width: 220px;
}

/* 操作栏 */
.fm-actionbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}

.actionbar-left, .actionbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-text {
  margin-left: 4px;
}

.view-toggle {
  margin-left: 4px;
}

/* 内容区 */
.fm-content {
  flex: 1;
  overflow: auto;
  background: var(--el-fill-color-blank);
  min-height: 0;
}

/* 卡片视图 */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 12px;
  padding: 16px;
}

.file-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
  border: 1px solid transparent;
}

.file-card:hover {
  background: var(--el-fill-color-light);
  border-color: var(--el-border-color);
}

.card-icon {
  margin-bottom: 8px;
}

.card-name {
  font-size: 12px;
  text-align: center;
  word-break: break-all;
  line-height: 1.4;
  max-height: 2.8em;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.card-meta {
  font-size: 11px;
  color: var(--el-text-color-placeholder);
  margin-top: 4px;
}

.calc-link {
  color: var(--el-color-primary);
  cursor: pointer;
  font-size: 11px;
}

.calc-link:hover {
  text-decoration: underline;
}

.empty-tip {
  text-align: center;
  color: var(--el-text-color-placeholder);
  padding: 60px 0;
  grid-column: 1 / -1;
}

/* 分页栏 */
.fm-pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--el-bg-color);
  border-top: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}

.pagination-left {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.stat-sep {
  color: var(--el-border-color);
}

/* 表格内部 */
.file-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-icon { flex-shrink: 0; }
.file-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-name.is-dir { color: var(--el-color-primary); cursor: pointer; }
.note-tag { flex-shrink: 0; max-width: 120px; overflow: hidden; text-overflow: ellipsis; }

.note-cell { min-height: 24px; display: flex; align-items: center; }
.note-text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.note-text.empty { color: var(--el-text-color-placeholder); }

/* 右键菜单 */
.context-menu {
  position: fixed; z-index: 9999;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px; padding: 4px 0; min-width: 160px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}
.ctx-item { display: flex; align-items: center; gap: 8px; padding: 8px 16px; font-size: 13px; cursor: pointer; color: var(--el-text-color-regular); transition: background 0.15s; }
.ctx-item:hover { background: var(--el-fill-color-light); }
.ctx-item.danger { color: var(--el-color-danger); }
.ctx-divider { height: 1px; background: var(--el-border-color-lighter); margin: 4px 0; }

/* 编辑器 */
.editor-wrapper { height: 60vh; }
.file-editor {
  width: 100%; height: 100%;
  border: 1px solid var(--el-border-color); border-radius: 4px; padding: 12px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace; font-size: 13px; line-height: 1.6;
  resize: none; outline: none;
  background: var(--el-bg-color); color: var(--el-text-color-primary); tab-size: 4;
}
.file-editor:focus { border-color: var(--el-color-primary); }

:deep(.el-dropdown-menu__item.active) { color: var(--el-color-primary); font-weight: 600; }
</style>
