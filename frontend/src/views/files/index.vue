<template>
  <div class="file-manager">
    <!-- 标签页栏 -->
    <div class="fm-tabs">
      <div v-for="tab in filesStore.tabs" :key="tab.id"
           class="fm-tab" :class="{ active: tab.id === filesStore.activeTabId }"
           @click="filesStore.setActiveTab(tab.id)">
        <OnIcon svgName="folder" :size="14" />
        <span class="fm-tab-title">{{ tab.title }}</span>
        <el-icon class="fm-tab-close" @click.stop="filesStore.closeTab(tab.id)"><Close /></el-icon>
      </div>
      <button class="fm-tab-add" @click="filesStore.addTab()">+</button>
    </div>

    <!-- 标签页内容（v-show 保持状态） -->
    <div v-for="tab in filesStore.tabs" :key="tab.id"
         v-show="tab.id === filesStore.activeTabId"
         class="fm-tab-panel">
      <FileTabPanel :tab-id="tab.id" :initial-path="tab.path" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { Close } from '@element-plus/icons-vue'
import { useFilesStore } from '@/stores/files'
import OnIcon from '@/components/OnIcon/index.vue'
import FileTabPanel from './FileTabPanel.vue'

const filesStore = useFilesStore()

onMounted(() => {
  if (filesStore.tabs.length === 0) filesStore.addTab()
})
</script>

<style scoped>
.file-manager { display: flex; flex-direction: column; height: 100%; overflow: hidden; }

.fm-tabs { display: flex; align-items: center; gap: 6px; padding: 8px 12px; background: var(--el-bg-color); border-bottom: 1px solid var(--el-border-color-lighter); flex-shrink: 0; overflow-x: auto; }
.fm-tab { display: flex; align-items: center; gap: 6px; width: 160px; padding: 6px 10px; font-size: 13px; cursor: pointer; border: 1px solid var(--el-border-color-lighter); border-radius: 6px; background: var(--el-fill-color-blank); white-space: nowrap; color: var(--el-text-color-regular); transition: all 0.2s; }
.fm-tab:hover { color: var(--el-color-primary); border-color: var(--el-color-primary-light-7); background: var(--el-color-primary-light-9); }
.fm-tab.active { color: var(--el-color-primary); border-color: var(--el-color-primary); background: var(--el-color-primary-light-9); font-weight: 500; }
.fm-tab-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
.fm-tab-close { font-size: 12px; border-radius: 50%; padding: 1px; }
.fm-tab-close:hover { background: var(--el-color-danger-light-9); color: var(--el-color-danger); }
.fm-tab-add { display: flex; align-items: center; justify-content: center; width: 28px; height: 28px; margin-left: 4px; border: 1px dashed var(--el-border-color); border-radius: 4px; background: transparent; cursor: pointer; font-size: 16px; color: var(--el-text-color-placeholder); }
.fm-tab-add:hover { color: var(--el-color-primary); border-color: var(--el-color-primary); }

.fm-tab-panel { flex: 1; min-height: 0; overflow: hidden; }
</style>
