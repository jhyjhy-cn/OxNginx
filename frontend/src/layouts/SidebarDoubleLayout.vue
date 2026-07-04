<template>
  <el-container class="layout-container">
    <el-aside width="220px" class="sidebar" :style="{ backgroundColor: sidebarBg, '--menu-active-bg': menuActiveBg }">
      <div class="logo" :style="{ borderBottomColor: borderColor }">
        <el-icon :size="20" color="#fff"><Promotion /></el-icon>
        <span>OxNginx</span>
      </div>
      <el-menu :default-active="route.path" :background-color="sidebarBg" :text-color="menuTextColor" :active-text-color="menuActiveTextColor" router>
        <el-menu-item v-for="item in flatMenuItems" :key="item.path" :index="item.path">
          <el-icon><component :is="item.icon" /></el-icon>
          <span>{{ t(item.title) }}</span>
        </el-menu-item>
      </el-menu>
    </el-aside>

    <el-container class="right-container">
      <el-header class="topbar">
        <div class="topbar-left">
          <el-breadcrumb separator="/">
            <el-breadcrumb-item :to="{ path: '/' }">{{ t('layout.home') }}</el-breadcrumb-item>
            <el-breadcrumb-item v-if="route.meta.title">{{ t(route.meta.title as string) }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        <TopBarRight
          @open-theme-drawer="$emit('openThemeDrawer')"
          @change-username="$emit('changeUsername')"
          @change-password="$emit('changePassword')"
        />
      </el-header>

      <TabBar v-if="settingsStore.showTabs" />

      <el-main class="main-content">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import TopBarRight from './components/TopBarRight.vue'
import TabBar from './components/TabBar.vue'
import { useSidebarTheme } from '@/composables/useSidebarTheme'
import { useSettingsStore } from '@/stores/settings'
import { flatMenuItems } from '@/config/menu'

const route = useRoute()
const { t } = useI18n()
const settingsStore = useSettingsStore()
const { sidebarBg, menuTextColor, menuActiveTextColor, menuActiveBg, borderColor } = useSidebarTheme()

defineEmits<{
  openThemeDrawer: []
  changeUsername: []
  changePassword: []
}>()
</script>

<style scoped>
.layout-container {
  height: 100vh;
  overflow: hidden;
}
.sidebar {
  overflow-y: auto;
  overflow-x: hidden;
}
.sidebar::-webkit-scrollbar {
  width: 0;
}
.logo {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #fff;
  font-size: 18px;
  font-weight: 600;
  border-bottom: 1px solid hsla(0,0%,100%,.1);
}
:deep(.el-menu-item.is-active) {
  background-color: var(--menu-active-bg) !important;
  border-radius: 0;
}
.right-container {
  flex-direction: column;
  overflow: hidden;
}
.topbar {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color);
  padding: 0 16px;
}
.topbar-left {
  display: flex;
  align-items: center;
}
.main-content {
  background: var(--el-bg-color-page);
  overflow-y: auto;
  padding: 16px;
}

:deep(.el-menu) {
  border-right: none;
}
</style>
