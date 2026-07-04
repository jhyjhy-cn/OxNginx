<template>
  <el-container class="layout-container">
    <el-aside :width="isCollapsed ? '64px' : '220px'" class="sidebar" :style="{ backgroundColor: sidebarBg, '--menu-active-bg': menuActiveBg }">
      <div class="logo" :class="{ collapsed: isCollapsed }" :style="{ borderBottomColor: borderColor }">
        <el-icon :size="20" color="#fff"><Promotion /></el-icon>
        <span v-show="!isCollapsed">OxNginx</span>
      </div>
      <el-menu
        :default-active="route.path"
        :collapse="isCollapsed"
        :background-color="sidebarBg"
        :text-color="menuTextColor"
        :active-text-color="menuActiveTextColor"
        router
        :collapse-transition="false"
      >
        <!-- 仪表盘 -->
        <el-menu-item :index="flatMenuItems[0].path">
          <el-icon><component :is="flatMenuItems[0].icon" /></el-icon>
          <template #title>{{ t(flatMenuItems[0].title) }}</template>
        </el-menu-item>
        <!-- 分组菜单 -->
        <el-sub-menu v-for="group in groupedMenuItems" :key="group.index" :index="group.index">
          <template #title>
            <el-icon><component :is="group.icon" /></el-icon>
            <span>{{ t(group.title) }}</span>
          </template>
          <el-menu-item v-for="child in group.children" :key="child.path" :index="child.path">
            {{ t(child.title) }}
          </el-menu-item>
        </el-sub-menu>
        <!-- 设置 -->
        <el-menu-item :index="settingsItem.path">
          <el-icon><component :is="settingsItem.icon" /></el-icon>
          <template #title>{{ t(settingsItem.title) }}</template>
        </el-menu-item>
      </el-menu>

      <div class="collapse-btn" @click="isCollapsed = !isCollapsed">
        <el-icon :size="16" color="hsla(0,0%,100%,.65)">
          <DArrowLeft v-if="!isCollapsed" />
          <DArrowRight v-else />
        </el-icon>
      </div>
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
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import TopBarRight from './components/TopBarRight.vue'
import TabBar from './components/TabBar.vue'
import { useSidebarTheme } from '@/composables/useSidebarTheme'
import { useSettingsStore } from '@/stores/settings'
import { flatMenuItems, groupedMenuItems } from '@/config/menu'

const route = useRoute()
const { t } = useI18n()
const isCollapsed = ref(false)
const settingsStore = useSettingsStore()
const { sidebarBg, menuTextColor, menuActiveTextColor, menuActiveBg, borderColor } = useSidebarTheme()
const settingsItem = flatMenuItems[flatMenuItems.length - 1]

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
  transition: width .3s;
  display: flex;
  flex-direction: column;
  position: relative;
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
  flex-shrink: 0;
}
.logo.collapsed {
  gap: 0;
}
.collapse-btn {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-top: 1px solid hsla(0,0%,100%,.1);
  background: inherit;
}
.collapse-btn:hover {
  background: hsla(0,0%,100%,.05);
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
:deep(.el-menu--collapse .el-sub-menu__title span),
:deep(.el-menu--collapse .el-sub-menu__title .el-sub-menu__icon-arrow) {
  display: none;
}
:deep(.el-menu-item.is-active) {
  background-color: var(--menu-active-bg) !important;
}
:deep(.el-sub-menu .el-menu-item.is-active) {
  background-color: var(--menu-active-bg) !important;
}
</style>
