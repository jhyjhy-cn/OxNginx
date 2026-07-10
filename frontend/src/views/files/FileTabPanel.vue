<template>
  <div class="file-tab-panel">
    <!-- 路径栏 -->
    <div class="fm-pathbar">
      <div class="pathbar-left">
        <el-button :icon="Back" size="small" @click="fm.goBack()" :disabled="!fm.currentParent.value" />
        <!-- 盘符/根目录 - 始终显示 -->
        <button v-if="fm.drives.value.length > 1" class="path-seg-btn drive-btn" @click.stop="fm.openDriveDropdown($event)">
          {{ fm.currentDriveLabel.value }}
        </button>
        <button v-else class="path-seg-btn" @click.stop="fm.navigateToSegment('/')">根目录</button>
        <!-- 面包屑视图（未聚焦） -->
        <div v-if="!fm.pathInputFocused.value" class="path-breadcrumb" @dblclick="fm.startEditPath()">
          <template v-for="(seg, i) in fm.pathSegments.value" :key="seg.path">
            <button v-if="i > 0" class="path-sep" @click.stop="fm.togglePathDropdown(i - 1, $event)">›</button>
            <button class="path-seg-btn" @click.stop="fm.navigateToSegment(seg.path)">{{ seg.name }}</button>
          </template>
        </div>
        <!-- 输入框视图（聚焦） -->
        <el-input
          v-else
          v-model="fm.inputPath.value"
          size="small"
          class="path-input"
          :placeholder="fm.currentPath.value"
          @keyup.enter="fm.goToInputPath()"
          @keyup.escape="fm.pathInputFocused.value = false"
          @blur="fm.pathInputFocused.value = false"
          autofocus
        />
        <el-button :icon="Refresh" size="small" @click="fm.fetchFiles()" />
      </div>
      <div class="pathbar-right">
        <el-input
          v-model="fm.searchQuery.value"
          size="small"
          class="search-input"
          placeholder="搜索文件..."
          clearable
          :prefix-icon="Search"
        />
      </div>
    </div>

    <!-- 路径下拉菜单 -->
    <Teleport to="body">
      <div v-if="fm.pathDropdown.visible" class="path-dropdown-mask" @click="fm.closePathDropdown()"></div>
      <div v-if="fm.pathDropdown.visible" class="path-dropdown" :style="{ left: fm.pathDropdown.x + 'px', top: fm.pathDropdown.y + 'px' }">
        <div v-if="fm.pathDropdown.dirs.length === 0" class="path-dropdown-empty">无子目录</div>
        <div
          v-for="dir in fm.pathDropdown.dirs"
          :key="dir.path"
          class="path-dropdown-item"
          :class="{ active: fm.pathDropdown.level === -1 && dir.path.toLowerCase().startsWith(fm.currentDrive.value.toLowerCase()) }"
          @click="fm.navigateToSegment(dir.path)"
        >
          <template v-if="fm.pathDropdown.level === -1">{{ dir.name }}</template>
          <template v-else>
            <el-icon class="path-dropdown-icon"><Folder /></el-icon>
            {{ dir.name }}
          </template>
        </div>
      </div>
    </Teleport>

    <!-- 操作栏 -->
    <div class="fm-actionbar">
      <div class="actionbar-left">
        <el-dropdown @command="fm.handleCreate" trigger="click">
          <el-button size="small" type="primary">
            <el-icon><Plus /></el-icon>
            {{ t('sys.files.createFile') }}
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="file">{{ t('sys.files.createFile') }}</el-dropdown-item>
              <el-dropdown-item command="folder">{{ t('sys.files.createFolder') }}</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
      <div class="actionbar-right">
        <template v-if="fm.selectedFiles.value.length > 0">
          <span class="selected-count">已选 {{ fm.selectedFiles.value.length }} 项</span>
          <el-dropdown @command="fm.handleBatchCommand" trigger="click">
            <el-button size="small">
              更多
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="copy">
                  <el-icon><CopyDocument /></el-icon>
                  复制
                </el-dropdown-item>
                <el-dropdown-item command="move">
                  <el-icon><Rank /></el-icon>
                  剪切
                </el-dropdown-item>
                <el-dropdown-item command="compress">
                  <el-icon><FolderAdd /></el-icon>
                  压缩
                </el-dropdown-item>
                <el-dropdown-item command="chmod">
                  <el-icon><Lock /></el-icon>
                  权限
                </el-dropdown-item>
                <el-dropdown-item divided command="delete">
                  <el-icon><Delete /></el-icon>
                  <span style="color: var(--el-color-danger)">删除</span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </template>
        <el-button size="small" disabled>
          <el-icon><Delete /></el-icon>
          <span class="btn-text">回收站</span>
        </el-button>
        <el-button-group class="view-toggle">
          <el-button size="small" :type="fm.viewMode.value === 'list' ? 'primary' : ''" @click="fm.viewMode.value = 'list'">
            <el-icon><List /></el-icon>
          </el-button>
          <el-button size="small" :type="fm.viewMode.value === 'card' ? 'primary' : ''" @click="fm.viewMode.value = 'card'">
            <el-icon><Grid /></el-icon>
          </el-button>
        </el-button-group>
      </div>
    </div>

    <!-- 内容区域 -->
    <div class="fm-content" v-loading="fm.loading.value">
      <FileListView
        v-if="fm.viewMode.value === 'list'"
        :files="fm.filteredPagedFiles.value"
        :get-file-icon="fm.getFileIcon"
        :format-size="fm.formatSize"
        :hover-note-path="fm.hoverNotePath.value"
        :editing-note="fm.editingNote.value"
        @contextmenu="fm.handleContextMenu"
        @dblclick="fm.handleDblClick"
        @selection-change="fm.handleSelectionChange"
        @calc-size="fm.calcFileSize"
        @note-enter="fm.handleNoteEnter"
        @note-leave="fm.handleNoteLeave"
        @note-update="(v) => (fm.editingNote.value = v)"
        @note-save="fm.saveInlineNote"
        @rename="fm.handleRename"
        @delete="fm.handleDelete"
      />
      <FileGridView
        v-else
        :files="fm.filteredPagedFiles.value"
        :get-file-icon="fm.getFileIcon"
        :format-size="fm.formatSize"
        :empty-text="t('sys.files.emptyDir')"
        @dblclick="fm.handleDblClick"
        @contextmenu="(row, e) => fm.handleContextMenu(row, e)"
        @calc-size="fm.calcFileSize"
      />
    </div>

    <!-- 分页栏 -->
    <div class="fm-pagination">
      <div class="pagination-left">
        <span>{{ t('sys.files.totalItems', { n: fm.total.value }) }}</span>
        <span class="stat-sep">|</span>
        <span>{{ fm.dirCount.value }} 个目录</span>
        <span class="stat-sep">|</span>
        <span>{{ fm.fileCount.value }} 个文件</span>
        <span class="stat-sep">|</span>
        <span>
          大小:
          <template v-if="fm.totalSize.value !== null">{{ fm.formatSize(fm.totalSize.value) }}</template>
          <el-button v-else link type="primary" size="small" :loading="fm.calcTotalLoading.value" @click="fm.calcTotalSize()">
            计算
          </el-button>
        </span>
      </div>
      <el-pagination
        v-model:current-page="fm.currentPage.value"
        v-model:page-size="fm.pageSize.value"
        :page-sizes="[100, 500, 1000, 1500, 2000]"
        :total="fm.total.value"
        layout="sizes, prev, pager, next, jumper"
        background
        small
      />
    </div>

    <!-- 右键菜单 -->
    <FileContextMenu
      :visible="fm.contextMenu.visible"
      :x="fm.contextMenu.x"
      :y="fm.contextMenu.y"
      :row="fm.contextMenu.row"
      :is-archive="fm.isArchive"
      @close="fm.contextMenu.visible = false"
      @open="fm.enterDir"
      @edit="fm.handleEdit"
      @download="fm.handleDownload"
      @chmod="fm.handleChmod"
      @copy="fm.handleCopy"
      @move="fm.handleMove"
      @rename="fm.handleRename"
      @delete="fm.handleDelete"
      @compress="fm.handleCompressSingle"
      @extract="fm.handleExtract"
      @properties="fm.handleProperties"
    />

    <!-- 弹窗们 -->
    <FileDialogs
      :create-dialog="fm.createDialog"
      :rename-dialog="fm.renameDialog"
      :move-dialog="fm.moveDialog"
      :compress-dialog="fm.compressDialog"
      :chmod-dialog="fm.chmodDialog"
      :edit-dialog="fm.editDialog"
      :prop-dialog="fm.propDialog"
      :format-size="fm.formatSize"
      @submit-create="fm.submitCreate"
      @submit-rename="fm.submitRename"
      @submit-move="fm.submitMove"
      @submit-compress="fm.submitCompress"
      @submit-chmod="fm.submitChmod"
      @submit-edit="fm.submitEdit"
      @calc-size="fm.calcFileSize"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import {
  Back,
  Refresh,
  Search,
  Plus,
  List,
  Grid,
  ArrowDown,
  CopyDocument,
  Rank,
  FolderAdd,
  Lock,
  Delete,
  Folder,
} from '@element-plus/icons-vue'
import { useFileManager } from './useFileManager'
import FileListView from './FileListView.vue'
import FileGridView from './FileGridView.vue'
import FileContextMenu from './FileContextMenu.vue'
import FileDialogs from './FileDialogs.vue'

const props = defineProps<{ tabId: string; initialPath: string }>()
const { t } = useI18n()
const fm = useFileManager(props.initialPath, props.tabId)
</script>

<style scoped>
.file-tab-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

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
  max-width: 66%;
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

.fm-actionbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}
.actionbar-left,
.actionbar-right {
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
.selected-count {
  font-size: 13px;
  color: var(--el-color-primary);
  margin-right: 4px;
}

.fm-content {
  flex: 1;
  overflow: auto;
  background: var(--el-fill-color-blank);
  min-height: 0;
}

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

:deep(.el-dropdown-menu__item.active) {
  color: var(--el-color-primary);
  font-weight: 600;
}

/* 面包屑路径 */
.path-breadcrumb {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  max-width: 66%;
  gap: 2px;
  padding: 2px 8px;
  border: 1px dashed var(--el-border-color);
  border-radius: 4px;
  cursor: text;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: none;
}
.path-breadcrumb::-webkit-scrollbar {
  display: none;
}
.path-breadcrumb:hover {
  border-color: var(--el-border-color-darker);
}
.path-seg-btn {
  font-size: 13px;
  color: var(--el-text-color-regular);
  background: var(--el-fill-color-light);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 3px;
  padding: 2px 8px;
  cursor: pointer;
  white-space: nowrap;
  line-height: 1.4;
}
.path-seg-btn:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-7);
  background: var(--el-color-primary-light-9);
}
.drive-btn {
  font-weight: 600;
}
.drive-btn::after {
  content: '▾';
  margin-left: 2px;
  font-size: 10px;
}
.path-sep {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  background: var(--el-fill-color-light);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 3px;
  padding: 2px 4px;
  cursor: pointer;
  line-height: 1.4;
}
.path-sep:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-7);
  background: var(--el-color-primary-light-9);
}

/* 路径下拉 */
.path-dropdown-mask {
  position: fixed;
  inset: 0;
  z-index: 2000;
}
.path-dropdown {
  position: fixed;
  z-index: 2001;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color-light);
  border-radius: 4px;
  box-shadow: var(--el-box-shadow-light);
  padding: 4px 0;
  min-width: 120px;
  max-height: 280px;
  overflow-y: auto;
}
.path-dropdown-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}
.path-dropdown-item:hover {
  background: var(--el-fill-color-light);
  color: var(--el-color-primary);
}
.path-dropdown-item.active {
  color: var(--el-color-primary);
  font-weight: 600;
}
.path-dropdown-icon {
  font-size: 14px;
  flex-shrink: 0;
}
.path-dropdown-empty {
  padding: 8px 12px;
  font-size: 13px;
  color: var(--el-text-color-placeholder);
}
</style>
