<template>
  <div class="file-manager">
    <!-- 路径栏 -->
    <div class="fm-pathbar">
      <div class="pathbar-left">
        <el-button :icon="Back" size="small" @click="fm.goBack()" :disabled="!fm.currentParent.value" />
        <el-input v-model="fm.inputPath.value" size="small" class="path-input" :placeholder="fm.currentPath.value" @keyup.enter="fm.goToInputPath()" @focus="fm.inputPath.value = fm.currentPath.value">
          <template #prefix>
            <el-dropdown v-if="fm.drives.value.length > 1" @command="fm.handleDriveChange" trigger="click">
              <span class="drive-prefix">{{ fm.currentDrive.value }}</span>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item v-for="d in fm.drives.value" :key="d" :command="d" :class="{ active: d === fm.currentDrive.value }">{{ d }}</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
        </el-input>
        <el-button :icon="Refresh" size="small" @click="fm.fetchFiles()" />
      </div>
      <div class="pathbar-right">
        <el-input v-model="fm.searchQuery.value" size="small" class="search-input" placeholder="搜索文件..." clearable :prefix-icon="Search" />
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="fm-actionbar">
      <div class="actionbar-left">
        <el-dropdown @command="fm.handleCreate" trigger="click">
          <el-button size="small" type="primary"><el-icon><Plus /></el-icon>{{ t('files.createFile') }}</el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="file">{{ t('files.createFile') }}</el-dropdown-item>
              <el-dropdown-item command="folder">{{ t('files.createFolder') }}</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
      <div class="actionbar-right">
        <template v-if="fm.selectedFiles.value.length > 0">
          <span class="selected-count">已选 {{ fm.selectedFiles.value.length }} 项</span>
          <el-dropdown @command="fm.handleBatchCommand" trigger="click">
            <el-button size="small">更多<el-icon class="el-icon--right"><ArrowDown /></el-icon></el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="copy"><el-icon><CopyDocument /></el-icon>复制</el-dropdown-item>
                <el-dropdown-item command="move"><el-icon><Rank /></el-icon>剪切</el-dropdown-item>
                <el-dropdown-item command="compress"><el-icon><FolderAdd /></el-icon>压缩</el-dropdown-item>
                <el-dropdown-item command="chmod"><el-icon><Lock /></el-icon>权限</el-dropdown-item>
                <el-dropdown-item divided command="delete"><el-icon><Delete /></el-icon><span style="color:var(--el-color-danger)">删除</span></el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </template>
        <el-button size="small" disabled><el-icon><Delete /></el-icon><span class="btn-text">回收站</span></el-button>
        <el-button-group class="view-toggle">
          <el-button size="small" :type="fm.viewMode.value === 'list' ? 'primary' : ''" @click="fm.viewMode.value = 'list'"><el-icon><List /></el-icon></el-button>
          <el-button size="small" :type="fm.viewMode.value === 'card' ? 'primary' : ''" @click="fm.viewMode.value = 'card'"><el-icon><Grid /></el-icon></el-button>
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
        @note-update="v => fm.editingNote.value = v"
        @note-save="fm.saveInlineNote"
        @rename="fm.handleRename"
        @delete="fm.handleDelete"
      />
      <FileGridView
        v-else
        :files="fm.filteredPagedFiles.value"
        :get-file-icon="fm.getFileIcon"
        :format-size="fm.formatSize"
        :empty-text="t('files.emptyDir')"
        @dblclick="fm.handleDblClick"
        @contextmenu="(row, e) => fm.handleContextMenu(row, e)"
        @calc-size="fm.calcFileSize"
      />
    </div>

    <!-- 分页栏 -->
    <div class="fm-pagination">
      <div class="pagination-left">
        <span>{{ t('files.totalItems', { n: fm.filteredFiles.value.length }) }}</span>
        <span class="stat-sep">|</span>
        <span>{{ fm.dirCount.value }} 个目录</span>
        <span class="stat-sep">|</span>
        <span>{{ fm.fileCount.value }} 个文件</span>
        <span class="stat-sep">|</span>
        <span>大小: <template v-if="fm.totalSize.value !== null">{{ fm.formatSize(fm.totalSize.value) }}</template><el-button v-else link type="primary" size="small" :loading="fm.calcTotalLoading.value" @click="fm.calcTotalSize()">计算</el-button></span>
      </div>
      <el-pagination v-model:current-page="fm.currentPage.value" v-model:page-size="fm.pageSize.value" :page-sizes="[100, 500, 1000, 1500, 2000]" :total="fm.filteredFiles.value.length" layout="sizes, prev, pager, next, jumper" background small />
    </div>

    <!-- 右键菜单 -->
    <FileContextMenu
      :visible="fm.contextMenu.visible" :x="fm.contextMenu.x" :y="fm.contextMenu.y" :row="fm.contextMenu.row"
      :is-archive="fm.isArchive"
      @close="fm.contextMenu.visible = false"
      @open="fm.enterDir" @edit="fm.handleEdit" @download="fm.handleDownload"
      @chmod="fm.handleChmod" @copy="fm.handleCopy" @move="fm.handleMove"
      @rename="fm.handleRename" @delete="fm.handleDelete"
      @compress="fm.handleCompressSingle" @extract="fm.handleExtract" @properties="fm.handleProperties"
    />

    <!-- 弹窗们 -->
    <FileDialogs
      :create-dialog="fm.createDialog" :rename-dialog="fm.renameDialog" :move-dialog="fm.moveDialog"
      :compress-dialog="fm.compressDialog" :chmod-dialog="fm.chmodDialog" :edit-dialog="fm.editDialog"
      :prop-dialog="fm.propDialog" :format-size="fm.formatSize"
      @submit-create="fm.submitCreate" @submit-rename="fm.submitRename" @submit-move="fm.submitMove"
      @submit-compress="fm.submitCompress" @submit-chmod="fm.submitChmod" @submit-edit="fm.submitEdit"
      @calc-size="fm.calcFileSize"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Back, Refresh, Search, Plus, List, Grid, ArrowDown, CopyDocument, Rank, FolderAdd, Lock, Delete } from '@element-plus/icons-vue'
import { useFileManager } from './useFileManager'
import FileListView from './FileListView.vue'
import FileGridView from './FileGridView.vue'
import FileContextMenu from './FileContextMenu.vue'
import FileDialogs from './FileDialogs.vue'

const { t } = useI18n()
const fm = useFileManager()
</script>

<style scoped>
.file-manager { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

.fm-pathbar { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 8px 16px; background: var(--el-bg-color); border-bottom: 1px solid var(--el-border-color-lighter); flex-shrink: 0; }
.pathbar-left { display: flex; align-items: center; gap: 6px; flex: 1; min-width: 0; }
.path-input { flex: 1; min-width: 200px; }
.drive-prefix { cursor: pointer; font-size: 12px; color: var(--el-color-primary); margin-right: 2px; }
.pathbar-right { flex-shrink: 0; }
.search-input { width: 220px; }

.fm-actionbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; background: var(--el-bg-color); border-bottom: 1px solid var(--el-border-color-lighter); flex-shrink: 0; }
.actionbar-left, .actionbar-right { display: flex; align-items: center; gap: 8px; }
.btn-text { margin-left: 4px; }
.view-toggle { margin-left: 4px; }
.selected-count { font-size: 13px; color: var(--el-color-primary); margin-right: 4px; }

.fm-content { flex: 1; overflow: auto; background: var(--el-fill-color-blank); min-height: 0; }

.fm-pagination { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; background: var(--el-bg-color); border-top: 1px solid var(--el-border-color-lighter); flex-shrink: 0; }
.pagination-left { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--el-text-color-secondary); }
.stat-sep { color: var(--el-border-color); }

:deep(.el-dropdown-menu__item.active) { color: var(--el-color-primary); font-weight: 600; }
</style>
