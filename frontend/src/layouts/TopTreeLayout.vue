import { MenuType } from '@/enums'
<template>
  <el-container class="layout-container" direction="vertical">
    <div class="top-header" :style="{ background: sidebarBg, '--menu-active-bg': menuActiveBg }">
      <div class="header-left">
        <div class="logo">
          <el-icon :size="20" color="#fff"><Promotion /></el-icon>
          <span>OxNginx</span>
        </div>
        <el-menu
          :default-active="route.path"
          mode="horizontal"
          :background-color="sidebarBg"
          :text-color="menuTextColor"
          :active-text-color="menuActiveTextColor"
          router
          :ellipsis="false"
          class="top-menu"
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
                  <span>{{ t(child.title) }}</span>
                </el-menu-item>
              </template>
            </el-sub-menu>
            <!-- C 类型:菜单项 -->
            <el-menu-item v-else-if="node.type === MenuType.Menu && node.path" :index="node.path">
              <el-icon v-if="node.icon"><component :is="node.icon" /></el-icon>
              <span>{{ t(node.title) }}</span>
            </el-menu-item>
          </template>
        </el-menu>
      </div>
      <div class="header-right">
        <TopBarRight
          @open-theme-drawer="$emit('openThemeDrawer')"
          @change-username="$emit('changeUsername')"
          @change-password="$emit('changePassword')"
        />
      </div>
    </div>

    <el-container class="body-container">
      <el-header class="breadcrumb-bar" height="auto">
        <el-breadcrumb separator="/">
          <el-breadcrumb-item :to="{ path: '/' }">{{ t('layout.home') }}</el-breadcrumb-item>
          <el-breadcrumb-item v-if="route.meta.title">{{ t(route.meta.title as string) }}</el-breadcrumb-item>
        </el-breadcrumb>
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

const route = useRoute()
const { t } = useI18n()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { sidebarBg, menuTextColor, menuActiveTextColor, menuActiveBg } = useSidebarTheme()

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
.top-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 16px;
  flex-shrink: 0;
}
.header-left {
  display: flex;
  align-items: center;
  overflow: hidden;
}
.header-right {
  flex-shrink: 0;
}
.header-right :deep(.action-icon) {
  color: hsla(0, 0%, 100%, 0.65);
}
.header-right :deep(.action-icon:hover) {
  color: #fff;
}
.header-right :deep(.user-trigger) {
  color: hsla(0, 0%, 100%, 0.65);
}
.logo {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #fff;
  font-size: 18px;
  font-weight: 600;
  padding: 0 20px;
  flex-shrink: 0;
  height: 50px;
}
.top-menu {
  border-bottom: none !important;
  height: 50px;
}
.body-container {
  flex: 1;
  flex-direction: column;
  overflow: hidden;
}
.breadcrumb-bar {
  padding: 12px 16px;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.main-content {
  background: var(--el-bg-color-page);
  overflow-y: auto;
  padding: 16px;
}
:deep(.top-menu .el-menu-item.is-active) {
  background-color: var(--menu-active-bg) !important;
}
:deep(.top-menu .el-sub-menu .el-menu-item.is-active) {
  background-color: var(--menu-active-bg) !important;
}
</style>
