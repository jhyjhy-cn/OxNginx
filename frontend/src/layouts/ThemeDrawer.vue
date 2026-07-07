<template>
  <el-drawer :model-value="visible" direction="rtl" :size="340" @close="$emit('close')">
    <template #header>
      <span class="drawer-title">{{ t('theme.settings') }}</span>
    </template>

    <!-- 布局切换 -->
    <div class="section">
      <div class="section-title">{{ t('theme.layout') }}</div>
      <div class="layout-options">
        <div
          v-for="opt in layoutOptions"
          :key="opt.value"
          class="layout-card"
          :class="{ active: settingsStore.layoutMode === opt.value }"
          @click="settingsStore.setLayoutMode(opt.value)"
        >
          <div class="layout-preview" :class="'preview-' + opt.value">
            <template v-if="opt.value === 'sidebar-tree'">
              <div class="pv-aside pv-aside-narrow"></div>
              <div class="pv-body"><div class="pv-header"></div><div class="pv-content"></div></div>
            </template>
            <template v-else>
              <div class="pv-top-header"></div>
              <div class="pv-content-full"></div>
            </template>
          </div>
          <span class="layout-label">{{ t(opt.label) }}</span>
        </div>
      </div>
    </div>

    <!-- 主题色 -->
    <div class="section">
      <div class="section-title">{{ t('theme.themeColor') }}</div>
      <div class="color-row">
        <div class="preset-colors">
          <span
            v-for="c in presetColors"
            :key="c"
            class="color-dot"
            :class="{ active: settingsStore.themeColor === c }"
            :style="{ background: c }"
            @click="settingsStore.setThemeColor(c)"
          >
            <el-icon v-if="settingsStore.themeColor === c" :size="12" color="#fff"><Check /></el-icon>
          </span>
        </div>
        <el-color-picker
          :model-value="settingsStore.themeColor"
          @change="handleColorInput"
          show-alpha
          size="small"
        />
      </div>
    </div>

    <!-- 暗黑模式 -->
    <div class="section">
      <div class="section-row">
        <span class="section-title no-margin">{{ t('theme.darkMode') }}</span>
        <el-switch
          :model-value="settingsStore.darkMode"
          @click="settingsStore.toggleDarkMode($event)"
        >
          <template #active-action><Moon :size="12" /></template>
          <template #inactive-action><Sunny :size="12" /></template>
        </el-switch>
      </div>
    </div>

    <!-- 标签页 -->
    <div class="section">
      <div class="section-row">
        <span class="section-title no-margin">{{ t('theme.showTabs') }}</span>
        <el-switch v-model="settingsStore.showTabs" />
      </div>
      <div v-if="settingsStore.showTabs" class="section-row" style="margin-top: 10px;">
        <span class="section-title no-margin">{{ t('theme.showTabIcons') }}</span>
        <el-switch v-model="settingsStore.showTabIcons" />
      </div>
    </div>

    <!-- 多语言 -->
    <div class="section">
      <div class="section-title">{{ t('theme.language') }}</div>
      <el-radio-group v-model="currentLang" @change="handleLanguageChange" size="default">
        <el-radio-button value="zh-CN">中文</el-radio-button>
        <el-radio-button value="en-US">English</el-radio-button>
      </el-radio-group>
    </div>
  </el-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore, type LayoutMode } from '@/stores/settings'

const props = defineProps<{
  visible: boolean
}>()

defineEmits<{
  close: []
}>()

const { t, locale } = useI18n()
const settingsStore = useSettingsStore()

const layoutOptions: { value: LayoutMode; label: string }[] = [
  { value: 'sidebar-tree', label: 'theme.sidebarTree' },
  { value: 'top-tree', label: 'theme.topTree' },
]

const presetColors = ['#409EFF', '#536dfe', '#9c27b0', '#00bfa5', '#ff5722', '#e91e63']

// 语言切换
const currentLang = computed({
  get: () => settingsStore.locale,
  set: () => {},
})
function handleLanguageChange(lang: 'zh-CN' | 'en-US') {
  settingsStore.setLocale(lang)
  locale.value = lang
}

function handleColorInput(color: string | null) {
  if (color) settingsStore.setThemeColor(color)
}
</script>

<style scoped>
.drawer-title {
  font-size: 16px;
  font-weight: 600;
}
.section {
  margin-bottom: 24px;
}
.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  margin-bottom: 12px;
}
.section-title.no-margin {
  margin-bottom: 0;
}
.section-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

/* 布局卡片 */
.layout-options {
  display: flex;
  gap: 12px;
}
.layout-card {
  flex: 1;
  min-width: 80px;
  cursor: pointer;
  border: 2px solid var(--el-border-color);
  border-radius: 8px;
  padding: 8px;
  transition: border-color .2s, box-shadow .2s;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}
.layout-card:hover {
  border-color: var(--el-color-primary-light-3);
}
.layout-card.active {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 1px var(--el-color-primary-light-7);
}
.layout-preview {
  width: 100%;
  aspect-ratio: 16 / 10;
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  background: var(--el-fill-color-light);
  border: 1px solid var(--el-border-color-lighter);
}
.layout-label {
  font-size: 12px;
  color: var(--el-text-color-regular);
  white-space: nowrap;
}
.layout-card.active .layout-label {
  color: var(--el-color-primary);
  font-weight: 600;
}

/* 左侧双栏 预览 */
.preview-sidebar-double .pv-aside { width: 30%; background: #304156; }
.preview-sidebar-double .pv-body { flex: 1; display: flex; flex-direction: column; }
.preview-sidebar-double .pv-header { height: 25%; background: var(--el-fill-color); border-bottom: 1px solid var(--el-border-color-lighter); }
.preview-sidebar-double .pv-content { flex: 1; background: var(--el-bg-color); }

/* 左侧树形 预览 */
.preview-sidebar-tree .pv-aside-narrow { width: 18%; background: #304156; }
.preview-sidebar-tree .pv-body { flex: 1; display: flex; flex-direction: column; }
.preview-sidebar-tree .pv-header { height: 25%; background: var(--el-fill-color); border-bottom: 1px solid var(--el-border-color-lighter); }
.preview-sidebar-tree .pv-content { flex: 1; background: var(--el-bg-color); }

/* 顶部树形 预览 */
.preview-top-tree { flex-direction: column; }
.preview-top-tree .pv-top-header { height: 25%; background: #304156; }
.preview-top-tree .pv-content-full { flex: 1; background: var(--el-bg-color); }

/* 颜色选择 */
.color-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.preset-colors {
  display: flex;
  gap: 10px;
  flex: 1;
}
.color-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color .2s, transform .15s;
  display: flex;
  align-items: center;
  justify-content: center;
}
.color-dot:hover {
  transform: scale(1.15);
}
.color-dot.active {
  border-color: var(--el-color-primary);
  transform: scale(1.15);
}
</style>
