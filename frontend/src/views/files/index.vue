<template>
  <div class="file-manager">
    <!-- 标签页栏 -->
    <div class="fm-tabs">
      <draggable v-model="sortableTabs" item-key="id" class="fm-tab-list" animation="200" ghost-class="fm-tab-ghost" :force-fallback="true">
        <template #item="{ element }">
          <div
            class="fm-tab"
            :class="{ active: element.id === filesStore.activeTabId }"
            @click="filesStore.setActiveTab(element.id)"
            @contextmenu.prevent="onContextMenu($event, element)"
          >
            <OnIcon svgName="folder" :size="14" />
            <span class="fm-tab-title">{{ element.title }}</span>
            <el-icon class="fm-tab-close" @click.stop="filesStore.closeTab(element.id)"><Close /></el-icon>
          </div>
        </template>
      </draggable>
      <button class="fm-tab-add" @click="filesStore.addTab()">+</button>
    </div>

    <!-- 标签页内容（v-show 保持状态） -->
    <div v-for="tab in filesStore.tabs" :key="tab.id" v-show="tab.id === filesStore.activeTabId" class="fm-tab-panel">
      <FileTabPanel :tab-id="tab.id" :initial-path="tab.path" />
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div v-if="ctxMenu.visible" class="fm-ctx-menu" :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }">
        <div class="fm-ctx-item" @click="closeTab">
          <el-icon :size="14"><Close /></el-icon>
          <span>{{ $t('sys.tabs.close') }}</span>
        </div>
        <div class="fm-ctx-item" :class="{ disabled: !canCloseLeft }" @click="closeLeft">
          <el-icon :size="14"><Back /></el-icon>
          <span>{{ $t('sys.tabs.closeLeft') }}</span>
        </div>
        <div class="fm-ctx-item" :class="{ disabled: !canCloseRight }" @click="closeRight">
          <el-icon :size="14"><Right /></el-icon>
          <span>{{ $t('sys.tabs.closeRight') }}</span>
        </div>
        <div class="fm-ctx-item" :class="{ disabled: !canCloseOther }" @click="closeOther">
          <el-icon :size="14"><CircleClose /></el-icon>
          <span>{{ $t('sys.tabs.closeOther') }}</span>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import draggable from 'vuedraggable'
import { Close, Back, Right, CircleClose } from '@element-plus/icons-vue'
import { useFilesStore, type FileTab } from '@/stores/files'
import OnIcon from '@/components/OnIcon/index.vue'
import FileTabPanel from './FileTabPanel.vue'

const filesStore = useFilesStore()

// 拖拽排序（和 TabBar.vue 同模式）
const sortableTabs = computed({
  get: () => filesStore.tabs,
  set: (newList) => {
    filesStore.tabs = newList
  },
})

// 右键菜单
const ctxMenu = ref({ visible: false, x: 0, y: 0, tabId: '' })
const ctxTabIndex = computed(() => filesStore.tabs.findIndex((t) => t.id === ctxMenu.value.tabId))
const canCloseLeft = computed(() => ctxTabIndex.value > 0)
const canCloseRight = computed(() => ctxTabIndex.value < filesStore.tabs.length - 1)
const canCloseOther = computed(() => filesStore.tabs.length > 1)

function onContextMenu(e: MouseEvent, tab: FileTab) {
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, tabId: tab.id }
}

function closeCtxMenu() {
  ctxMenu.value.visible = false
}

function closeTab() {
  filesStore.closeTab(ctxMenu.value.tabId)
  closeCtxMenu()
}

function closeLeft() {
  if (!canCloseLeft.value) return
  filesStore.closeLeft(ctxMenu.value.tabId)
  closeCtxMenu()
}

function closeRight() {
  if (!canCloseRight.value) return
  filesStore.closeRight(ctxMenu.value.tabId)
  closeCtxMenu()
}

function closeOther() {
  if (!canCloseOther.value) return
  filesStore.closeOther(ctxMenu.value.tabId)
  closeCtxMenu()
}

onMounted(() => {
  if (filesStore.tabs.length === 0) filesStore.addTab()
  document.addEventListener('click', closeCtxMenu)
})

onUnmounted(() => {
  document.removeEventListener('click', closeCtxMenu)
})
</script>

<style scoped>
.file-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.fm-tabs {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  flex-shrink: 0;
}
.fm-tab-list {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow-x: auto;
}
.fm-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 120px;
  max-width: 180px;
  padding: 6px 10px;
  font-size: 13px;
  cursor: grab;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  background: var(--el-fill-color-blank);
  white-space: nowrap;
  color: var(--el-text-color-regular);
  transition:
    color 0.2s,
    border-color 0.2s,
    background 0.2s;
  user-select: none;
  -webkit-user-select: none;
  will-change: transform;
}
.fm-tab:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary-light-7);
  background: var(--el-color-primary-light-9);
}
.fm-tab.active {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
  font-weight: 500;
}
.fm-tab-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}
.fm-tab-close {
  font-size: 12px;
  border-radius: 50%;
  padding: 1px;
  cursor: pointer;
}
.fm-tab-close:hover {
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}
.fm-tab-add {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  margin-left: 4px;
  border: 1px dashed var(--el-border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
  color: var(--el-text-color-placeholder);
  flex-shrink: 0;
}
.fm-tab-add:hover {
  color: var(--el-color-primary);
  border-color: var(--el-color-primary);
}

.fm-tab-ghost {
  opacity: 0.4;
  cursor: grabbing !important;
}
.fm-tab-panel {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
</style>

<style>
/* 拖拽相关 */
.fm-ctx-menu {
  position: fixed;
  z-index: 9999;
  min-width: 140px;
  padding: 4px 0;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  box-shadow: var(--el-box-shadow-light);
}
.fm-ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  color: var(--el-text-color-regular);
  cursor: pointer;
  transition: all 0.15s;
}
.fm-ctx-item:hover {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}
.fm-ctx-item.disabled {
  color: var(--el-text-color-placeholder);
  cursor: not-allowed;
}
.fm-ctx-item.disabled:hover {
  background: transparent;
  color: var(--el-text-color-placeholder);
}
</style>
