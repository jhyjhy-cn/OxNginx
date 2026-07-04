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
          <el-menu-item index="/dashboard">
            <el-icon><Odometer /></el-icon>
            <span>{{ t('menu.dashboard') }}</span>
          </el-menu-item>
          <el-sub-menu index="site-group">
            <template #title>
              <el-icon><Grid /></el-icon>
              <span>{{ t('menu.sites') }}</span>
            </template>
            <el-menu-item index="/sites">{{ t('menu.sites') }}</el-menu-item>
            <el-menu-item index="/upstreams">{{ t('menu.upstreams') }}</el-menu-item>
            <el-menu-item index="/templates">{{ t('menu.templates') }}</el-menu-item>
          </el-sub-menu>
          <el-sub-menu index="security-group">
            <template #title>
              <el-icon><Lock /></el-icon>
              <span>{{ t('menu.ssl') }}</span>
            </template>
            <el-menu-item index="/ssl">{{ t('menu.ssl') }}</el-menu-item>
            <el-menu-item index="/access">{{ t('menu.access') }}</el-menu-item>
          </el-sub-menu>
          <el-sub-menu index="config-group">
            <template #title>
              <el-icon><Edit /></el-icon>
              <span>{{ t('menu.config') }}</span>
            </template>
            <el-menu-item index="/config">{{ t('menu.config') }}</el-menu-item>
            <el-menu-item index="/logs">{{ t('menu.logs') }}</el-menu-item>
          </el-sub-menu>
          <el-menu-item index="/settings">
            <el-icon><Setting /></el-icon>
            <span>{{ t('menu.settings') }}</span>
          </el-menu-item>
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
import { useSidebarTheme } from '@/composables/useSidebarTheme'

const route = useRoute()
const { t } = useI18n()
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
  color: hsla(0,0%,100%,.65);
}
.header-right :deep(.action-icon:hover) {
  color: #fff;
}
.header-right :deep(.user-trigger) {
  color: hsla(0,0%,100%,.65);
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
