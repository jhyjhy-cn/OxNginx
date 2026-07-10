<template>
  <div class="layout" :class="{ 'sidebar-collapsed': settingsStore.sidebarCollapsed }">
    <!-- 左侧菜单栏 -->
    <aside class="sidebar" :style="sidebarStyle">
      <div class="logo" :class="{ collapsed: settingsStore.sidebarCollapsed }">
        <img src="@/assets/imgs/logo.png" alt="OxNginx" class="logo-icon" />
        <span v-if="!settingsStore.sidebarCollapsed" class="logo-text">OxNginx</span>
      </div>
      <div class="menu-scroll">
        <ul class="menu-list">
          <template v-for="item in flatMenus" :key="item.path || item.title">
            <li v-if="item.isGroup" class="menu-group-header">
              <span v-if="!settingsStore.sidebarCollapsed">{{ t(item.title) }}</span>
            </li>
            <li
              v-else
              class="menu-item"
              :class="{
                active:
                  route.path === item.path || (item.path !== '/dashboard' && item.path !== '/settings' && route.path.startsWith(item.path)),
              }"
              @click="router.push(item.path)"
            >
              <el-icon :size="18"><component :is="item.icon" /></el-icon>
              <span v-if="!settingsStore.sidebarCollapsed" class="menu-text">{{ t(item.title) }}</span>
            </li>
          </template>
        </ul>
      </div>

      <!-- 折叠按钮 -->
      <div class="collapse-btn" @click="settingsStore.toggleSidebar()">
        <el-icon :size="16">
          <DArrowLeft v-if="!settingsStore.sidebarCollapsed" />
          <DArrowRight v-else />
        </el-icon>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <div class="main-area">
      <header class="topbar" :style="topbarStyle">
        <div class="topbar-left">
          <el-breadcrumb separator="/">
            <el-breadcrumb-item v-if="route.path !== '/dashboard'">
              {{ t('sys.menu.dashboard') }}
            </el-breadcrumb-item>
            <el-breadcrumb-item v-for="name in route.matched.map((r) => r.meta?.title as string).filter(Boolean)" :key="name">
              {{ t(name) }}
            </el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        <TopBarRight
          @open-theme-drawer="$emit('openThemeDrawer')"
          @change-username="$emit('changeUsername')"
          @change-password="$emit('changePassword')"
        />
      </header>

      <!-- 标签栏 -->
      <TabBar v-if="settingsStore.showTabs" />

      <main class="content-area">
        <router-view v-slot="{ Component: ViewComponent }">
          <Transition name="page-fade" mode="out-in">
            <component :is="ViewComponent" />
          </Transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { useSidebarTheme } from '@/composables/useSidebarTheme'
import { useAuthStore, type MenuNode } from '@/stores/auth'
import { tabIconMap } from '@/config/menu'
import TopBarRight from '@/layouts/components/TopBarRight.vue'
import TabBar from '@/layouts/components/TabBar.vue'
import { MenuType } from '@/consts'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { sidebarBg, menuTextColor, menuActiveTextColor, menuActiveBg, borderColor } = useSidebarTheme()

// ponytail: 后端菜单是树形,前端渲染拍扁;M 类型渲染为分组标题,C 类型渲染为菜单项
const flatMenus = computed(() => {
  type FlatItem = { path: string; title: string; icon: string; isGroup: boolean }
  const out: FlatItem[] = []
  const walk = (nodes: MenuNode[]) => {
    for (const n of nodes) {
      if (n.type === MenuType.Directory && n.children?.length) {
        // M 类型目录：渲染为分组标题,然后展开子项
        out.push({ path: '', title: n.title, icon: n.icon || '', isGroup: true })
        walk(n.children)
      } else if (n.type === MenuType.Menu && n.path) {
        out.push({
          path: n.path,
          title: n.title,
          icon: n.icon || tabIconMap[n.path] || 'Menu',
          isGroup: false,
        })
      }
    }
  }
  walk(authStore.menus)
  return out
})

const collapsedWidth = 64
const expandedWidth = 200
const sidebarWidth = computed(() => (settingsStore.sidebarCollapsed ? collapsedWidth : expandedWidth))

const sidebarStyle = computed(() => ({
  width: `${sidebarWidth.value}px`,
  minWidth: `${sidebarWidth.value}px`,
  backgroundColor: sidebarBg.value,
  borderRight: `1px solid ${borderColor.value}`,
}))

const topbarStyle = computed(() => ({
  borderBottom: `1px solid ${borderColor.value}`,
}))

defineEmits<{
  openThemeDrawer: []
  changeUsername: []
  changePassword: []
}>()
</script>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ===== 侧边栏 ===== */
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100vh;
  flex-shrink: 0;
  transition:
    width 0.28s ease,
    min-width 0.28s ease;
  overflow: hidden;
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 60px;
  font-size: 20px;
  font-weight: bold;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
}

.logo-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  object-fit: contain;
  flex-shrink: 0;
}

.logo-text {
  margin-left: 10px;
}

.sidebar-collapsed .logo-text {
  display: none;
}

.menu-scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.menu-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.menu-item {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 20px;
  cursor: pointer;
  transition: background-color 0.2s;
  white-space: nowrap;
  overflow: hidden;
  gap: 10px;
}

.sidebar-collapsed .menu-item {
  padding: 0;
  justify-content: center;
}

.menu-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
}

.menu-item.active {
  background-color: v-bind(menuActiveBg);
}

.menu-item .el-icon {
  color: v-bind(menuTextColor);
  flex-shrink: 0;
}

.menu-item.active .el-icon {
  color: v-bind(menuActiveTextColor);
}

.menu-text {
  color: v-bind(menuTextColor);
  font-size: 14px;
}

.menu-item.active .menu-text {
  color: v-bind(menuActiveTextColor);
}

/* M 类型目录分组标题 */
.menu-group-header {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 20px 0 16px;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: v-bind(menuTextColor);
  opacity: 0.6;
  white-space: nowrap;
  overflow: hidden;
}

.sidebar-collapsed .menu-group-header {
  height: 8px;
  padding: 0;
}

.sidebar-collapsed .menu-group-header span {
  display: none;
}

/* 折叠按钮 */
.collapse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 40px;
  cursor: pointer;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  color: v-bind(menuTextColor);
  transition: background-color 0.2s;
}

.collapse-btn:hover {
  background-color: rgba(255, 255, 255, 0.08);
}

/* ===== 右侧主区域 ===== */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 50px;
  padding: 0 20px;
  background: #fff;
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

:root.dark .topbar {
  background: #1d1e1f;
}

.content-area {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background: var(--el-bg-color-page);
}

/* ===== 页面过渡动画 ===== */
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.2s ease;
}

.page-fade-enter-from,
.page-fade-leave-to {
  opacity: 0;
}
</style>
