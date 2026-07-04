import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'
import { useFilesStore } from '@/stores/files'

export interface FileItem {
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

export function useFileManager(initialPath?: string, tabId?: string) {
  const { t } = useI18n()
  const filesStore = useFilesStore()

  // ===== 核心状态 =====
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
    const p = currentPath.value.replace(/\\\\\?\\/g, '').replace(/\\/g, '/')
    const match = p.match(/^([A-Za-z]:)/)
    return match ? match[1] : '/'
  })
  const currentDriveLabel = computed(() => {
    const d = currentDrive.value
    const letter = d.replace(/[^A-Za-z]/g, '').toUpperCase()
    return letter ? letter + '盘' : d
  })

  // 路径面包屑
  const pathSegments = computed(() => {
    const p = currentPath.value.replace(/\\\\\?\\/g, '').replace(/\\/g, '/')
    const segs: { name: string; path: string }[] = []
    // 盘符
    const driveMatch = p.match(/^([A-Za-z]:)/)
    if (driveMatch) {
      segs.push({ name: driveMatch[1], path: driveMatch[1] + '/' })
    }
    const rest = driveMatch ? p.slice(driveMatch[0].length).replace(/^\//, '') : p.replace(/^\//, '')
    if (!rest) return segs
    const parts = rest.split('/').filter(Boolean)
    let accumulated = driveMatch ? driveMatch[1] + '/' : '/'
    for (const part of parts) {
      accumulated += part + '/'
      segs.push({ name: part, path: accumulated })
    }
    return segs
  })

  // 右键菜单
  const contextMenu = reactive({ visible: false, x: 0, y: 0, row: null as FileItem | null })

  // 路径下拉菜单
  const pathInputFocused = ref(false)
  const pathDropdown = reactive({ visible: false, x: 0, y: 0, dirs: [] as { name: string; path: string }[], level: -1 })

  // 选中
  const selectedFiles = ref<FileItem[]>([])

  // 弹窗状态
  const createDialog = reactive({ visible: false, isDir: false, name: '' })
  const renameDialog = reactive({ visible: false, path: '', newName: '' })
  const moveDialog = reactive({ visible: false, source: '', destination: '', isCopy: false })
  const compressDialog = reactive({ visible: false, paths: [] as string[], name: '', format: 'zip' })
  const chmodDialog = reactive({ visible: false, path: '', mode: '' })
  const hoverNotePath = ref('')
  const editingNote = ref('')
  const editDialog = reactive({ visible: false, path: '', content: '' })
  const propDialog = reactive({ visible: false, item: null as FileItem | null })

  // ===== 生命周期 =====
  onMounted(() => {
    if (initialPath) currentPath.value = initialPath
    else if (filesStore.lastPath) currentPath.value = filesStore.lastPath
    fetchDrives()
    fetchFiles()
    document.addEventListener('click', closeContextMenu)
  })

  onBeforeUnmount(() => {
    document.removeEventListener('click', closeContextMenu)
  })

  function closeContextMenu() { contextMenu.visible = false; closePathDropdown() }

  // ===== API =====
  async function fetchFiles() {
    loading.value = true
    totalSize.value = null
    searchQuery.value = ''
    try {
      const { data } = await api.get('/api/files/list', { params: { path: currentPath.value } })
      if (data.code === 0) {
        files.value = data.data.items
        currentPath.value = data.data.path.replace(/\\\\\?\\/, '').replace(/\\/g, '/')
        currentParent.value = data.data.parent ? data.data.parent.replace(/\\\\\?\\/, '').replace(/\\/g, '/') : null
        currentPage.value = 1
        filesStore.lastPath = currentPath.value
        if (tabId) filesStore.updateTabPath(tabId, currentPath.value)
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

  function handleDriveChange(drive: string) { currentPath.value = drive.replace(/\\\\\?\\/, '').replace(/\\/g, '/'); fetchFiles() }

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

  // ===== 工具 =====
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

  // ===== 导航 =====
  function goBack() { if (currentParent.value) { currentPath.value = currentParent.value; fetchFiles() } }
  function goToInputPath() {
    if (!inputPath.value) return
    const normalized = inputPath.value.replace(/\\\\\?\\/, '').replace(/\\/g, '/').replace(/\/+/g, '/')
    currentPath.value = normalized
    inputPath.value = ''
    pathInputFocused.value = false
    fetchFiles()
  }

  async function togglePathDropdown(level: number, event: MouseEvent) {
    if (pathDropdown.visible && pathDropdown.level === level) { pathDropdown.visible = false; return }
    const seg = pathSegments.value[level]
    if (!seg) return
    const targetPath = seg.path
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
    pathDropdown.x = rect.left
    pathDropdown.y = rect.bottom + 4
    pathDropdown.level = level
    pathDropdown.dirs = []
    pathDropdown.visible = true
    try {
      const { data } = await api.get('/api/files/list', { params: { path: targetPath } })
      if (data.code === 0) {
        pathDropdown.dirs = data.data.items
          .filter((f: FileItem) => f.is_dir)
          .map((f: FileItem) => ({ name: f.name, path: f.path }))
      }
    } catch { pathDropdown.dirs = [] }
  }

  function navigateToSegment(path: string) {
    pathDropdown.visible = false
    currentPath.value = path
    fetchFiles()
  }

  function closePathDropdown() { pathDropdown.visible = false }

  function openDriveDropdown(event: MouseEvent) {
    if (pathDropdown.visible && pathDropdown.level === -1) { pathDropdown.visible = false; return }
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
    pathDropdown.x = rect.left
    pathDropdown.y = rect.bottom + 4
    pathDropdown.level = -1
    pathDropdown.dirs = drives.value.map(d => {
      const letter = d.replace(/[^A-Za-z]/g, '').toUpperCase()
      return { name: letter ? letter + '盘' : d, path: d.replace(/\\/g, '/') + '/' }
    })
    pathDropdown.visible = true
  }

  function handleDblClick(row: FileItem) { row.is_dir ? enterDir(row) : handleEdit(row) }
  function enterDir(row: FileItem) { currentPath.value = row.path.replace(/\\\\\?\\/, '').replace(/\\/g, '/'); fetchFiles() }

  function handleContextMenu(row: FileItem, colOrEvent: unknown, tableEvent?: MouseEvent) {
    const event = tableEvent || (colOrEvent instanceof MouseEvent ? colOrEvent : null)
    if (!event) return
    event.preventDefault()
    contextMenu.visible = true
    contextMenu.x = event.clientX
    contextMenu.y = event.clientY
    contextMenu.row = row
  }

  // ===== CRUD =====
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

  function handleCopy(row: FileItem) { moveDialog.source = row.path; moveDialog.destination = ''; moveDialog.isCopy = true; moveDialog.visible = true; contextMenu.visible = false }
  function handleMove(row: FileItem) { moveDialog.source = row.path; moveDialog.destination = ''; moveDialog.isCopy = false; moveDialog.visible = true; contextMenu.visible = false }

  async function submitMove() {
    const url = moveDialog.isCopy ? '/api/files/copy' : '/api/files/move'
    const sources = moveDialog.source.split('|').filter(Boolean)
    const destBase = moveDialog.destination.replace(/[\\/]+$/, '')
    try {
      let successCount = 0
      for (const src of sources) {
        const name = src.replace(/\\/g, '/').split('/').pop() || src
        const dest = destBase + '/' + name
        const { data } = await api.post(url, { source: src, destination: dest })
        if (data.code === 0) successCount++
        else ElMessage.error(data.message)
      }
      if (successCount > 0) {
        ElMessage.success(moveDialog.isCopy ? t('files.copySuccess') : t('files.moveSuccess'))
        moveDialog.visible = false; selectedFiles.value = []; fetchFiles()
      }
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

  function handleCompressSingle(row: FileItem) {
    compressDialog.paths = [row.path]
    const isWin = navigator.platform.toLowerCase().includes('win')
    compressDialog.name = row.name + (isWin ? '.zip' : '.tar.gz')
    compressDialog.format = isWin ? 'zip' : 'tar.gz'
    compressDialog.visible = true; contextMenu.visible = false
  }

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

  function handleDownload(row: FileItem) {
    contextMenu.visible = false
    const a = document.createElement('a')
    a.href = `/api/files/download?path=${encodeURIComponent(row.path)}`
    a.download = row.name
    document.body.appendChild(a); a.click(); document.body.removeChild(a)
  }

  function handleProperties(row: FileItem) { contextMenu.visible = false; propDialog.item = row; propDialog.visible = true }

  // ===== 选中 & 批量 =====
  function handleSelectionChange(rows: FileItem[]) { selectedFiles.value = rows }

  function handleBatchCommand(cmd: string) {
    const paths = selectedFiles.value.map(f => f.path)
    switch (cmd) {
      case 'copy':
        moveDialog.source = paths.join('|'); moveDialog.destination = ''; moveDialog.isCopy = true; moveDialog.visible = true; break
      case 'move':
        moveDialog.source = paths.join('|'); moveDialog.destination = ''; moveDialog.isCopy = false; moveDialog.visible = true; break
      case 'compress': {
        const isWin = navigator.platform.toLowerCase().includes('win')
        compressDialog.paths = paths; compressDialog.name = 'archive' + (isWin ? '.zip' : '.tar.gz')
        compressDialog.format = isWin ? 'zip' : 'tar.gz'; compressDialog.visible = true; break
      }
      case 'chmod':
        chmodDialog.path = paths[0]; chmodDialog.mode = '644'; chmodDialog.visible = true; break
      case 'delete': batchDelete(); break
    }
  }

  async function batchDelete() {
    try {
      await ElMessageBox.confirm(`确定删除选中的 ${selectedFiles.value.length} 项吗？`, t('common.warning'), { type: 'warning' })
      for (const f of selectedFiles.value) {
        await api.delete('/api/files/delete', { data: { path: f.path } })
      }
      ElMessage.success(t('files.deleteSuccess')); selectedFiles.value = []; fetchFiles()
    } catch { /* 取消 */ }
  }

  // ===== 备注 =====
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

  // ===== 返回 =====
  return {
    // 状态
    loading, currentPath, currentParent, files, inputPath, searchQuery, viewMode,
    filteredFiles, currentPage, pageSize, filteredPagedFiles,
    dirCount, fileCount, totalSize, calcTotalLoading,
    drives, currentDrive, currentDriveLabel, contextMenu, selectedFiles,
    createDialog, renameDialog, moveDialog, compressDialog, chmodDialog,
    hoverNotePath, editingNote, editDialog, propDialog,
    pathInputFocused, pathDropdown, pathSegments,
    // 方法
    fetchFiles, fetchDrives, handleDriveChange, calcTotalSize, calcFileSize,
    formatSize, isArchive, getFileIcon,
    goBack, goToInputPath, handleDblClick, enterDir, handleContextMenu, closeContextMenu,
    togglePathDropdown, navigateToSegment, closePathDropdown, openDriveDropdown,
    handleCreate, submitCreate, handleRename, submitRename, handleEdit, submitEdit,
    handleCopy, handleMove, submitMove, handleDelete,
    handleCompressSingle, submitCompress, handleExtract,
    handleChmod, submitChmod, handleDownload, handleProperties,
    handleSelectionChange, handleBatchCommand,
    handleNoteEnter, handleNoteLeave, saveInlineNote,
  }
}
