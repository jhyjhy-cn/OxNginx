<template>
  <el-container class="layout-container">
    <el-aside
      :width="settingsStore.sidebarCollapsed ? '64px' : '220px'"
      class="sidebar"
      :style="{ backgroundColor: sidebarBg, '--menu-active-bg': menuActiveBg }"
    >
      <div class="logo" :class="{ collapsed: settingsStore.sidebarCollapsed }" :style="{ borderBottomColor: borderColor }">
        <el-icon :size="20" color="#fff"><Promotion /></el-icon>
        <span v-show="!settingsStore.sidebarCollapsed">OxNginx</span>
      </div>
      <el-menu
        :default-active="route.path"
        :collapse="settingsStore.sidebarCollapsed"
        :background-color="sidebarBg"
        :text-color="menuTextColor"
        :active-text-color="menuActiveTextColor"
        router
        :collapse-transition="false"
      >
        <template v-for="node in authStore.menus" :key="node.id">
          <!-- M 类型:目录,渲染子菜单 -->
          <el-sub-menu v-if="node.type === MenuType.Directory && node.children?.length" :index="node.id.toString()">
            <template #title>
              <el-icon v-if="node.icon"><component :is="node.icon" /></el-icon>
              <span>{{ t(node.title) }}</span>
            </template>
            <template v-for="child in node.children" :key="child.id">
              <el-menu-item v-if="child.type === MenuType.Menu && child.path" :index="child.path">
                <el-icon v-if="child.icon"><component :is="child.icon" /></el-icon>
                <template #title>{{ t(child.title) }}</template>
              </el-menu-item>
            </template>
          </el-sub-menu>
          <!-- C 类型:菜单项,直接渲染 -->
          <el-menu-item v-else-if="node.type === MenuType.Menu && node.path" :index="node.path">
            <el-icon v-if="node.icon"><component :is="node.icon" /></el-icon>
            <template #title>{{ t(node.title) }}</template>
          </el-menu-item>
        </template>
      </el-menu>

      <div class="collapse-btn" @click="settingsStore.toggleSidebar()">
        <el-icon :size="16" color="hsla(0,0%,100%,.65)">
          <DArrowLeft v-if="!settingsStore.sidebarCollapsed" />
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
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import TopBarRight from './components/TopBarRight.vue'
import TabBar from './components/TabBar.vue'
import { useSidebarTheme } from '@/composables/useSidebarTheme'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { MenuType } from '@/enums'

const route = useRoute()
const { t } = useI18n()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()
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
  overflow: hidden;
  transition: width 0.3s;
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
  border-bottom: 1px solid hsla(0, 0%, 100%, 0.1);
  flex-shrink: 0;
}
.logo.collapsed {
  gap: 0;
}
.collapse-btn {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-top: 1px solid hsla(0, 0%, 100%, 0.1);
  background: inherit;
  flex-shrink: 0;
}
.collapse-btn:hover {
  background: hsla(0, 0%, 100%, 0.05);
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
  flex: 1;
  overflow-y: auto;
}
:deep(.el-menu::-webkit-scrollbar) {
  width: 0;
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
