<template>
  <el-container class="layout-container">
    <el-aside :width="isCollapsed ? '64px' : '220px'" class="sidebar">
      <div class="logo" :class="{ collapsed: isCollapsed }">
        <el-icon :size="20" color="#fff"><Promotion /></el-icon>
        <span v-show="!isCollapsed">OxNginx</span>
      </div>
      <el-menu
        :default-active="route.path"
        :collapse="isCollapsed"
        background-color="#001529"
        text-color="hsla(0,0%,100%,.65)"
        active-text-color="#fff"
        router
        :collapse-transition="false"
      >
        <el-menu-item index="/dashboard">
          <el-icon><Odometer /></el-icon>
          <template #title>{{ t('menu.dashboard') }}</template>
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
          <template #title>{{ t('menu.settings') }}</template>
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

const route = useRoute()
const { t } = useI18n()
const isCollapsed = ref(false)

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
  background-color: #001529;
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
  background: #001529;
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
</style>
