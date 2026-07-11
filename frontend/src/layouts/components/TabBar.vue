<template>
  <div class="tab-bar">
    <!-- 仪表盘固定在最左侧，不参与排序 -->
    <div
      class="tab-item tab-home"
      :class="{ active: tabStore.activePath === '/dashboard' }"
      @click="navigateTo({ path: '/dashboard', title: 'sys.menu.dashboard', closable: false })"
      @contextmenu.prevent="onContextMenu($event, { path: '/dashboard', title: 'sys.menu.dashboard', closable: false }, 0)"
    >
      <el-icon v-if="settingsStore.showTabIcons" :size="12"><Odometer /></el-icon>
      <span class="tab-title">{{ t('sys.menu.dashboard') }}</span>
    </div>

    <!-- 向左滚动按钮 -->
    <el-icon v-show="hasOverflow" class="tab-scroll-btn" :class="{ disabled: !canScrollLeft }" :size="14" @click="scrollLeft">
      <DArrowLeft />
    </el-icon>

    <!-- 可拖拽排序的标签页 -->
    <draggable
      v-model="sortableTabs"
      item-key="path"
      class="tab-list"
      ref="tabListRef"
      animation="200"
      ghost-class="tab-ghost"
      chosen-class="tab-chosen"
      drag-class="tab-drag"
      :force-fallback="true"
      @end="onDragEnd"
    >
      <template #item="{ element }">
        <div
          class="tab-item"
          :class="{ active: tabStore.activePath === element.path }"
          @click="navigateTo(element)"
          @contextmenu.prevent="onContextMenu($event, element, 0)"
        >
          <el-icon v-if="settingsStore.showTabIcons" :size="12"><component :is="getTabIcon(element.path)" /></el-icon>
          <span class="tab-title">{{ t(element.title) }}</span>
          <el-icon class="tab-close" :size="12" @click.stop="closeTab(element)">
            <Close />
          </el-icon>
        </div>
      </template>
    </draggable>

    <!-- 向右滚动按钮 -->
    <el-icon v-show="hasOverflow" class="tab-scroll-btn" :class="{ disabled: !canScrollRight }" :size="14" @click="scrollRight">
      <DArrowRight />
    </el-icon>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div v-if="contextMenu.visible" class="tab-context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
        <div class="menu-item" @click="handleRefresh">
          <el-icon :size="14"><Refresh /></el-icon>
          <span>{{ t('sys.tabs.refresh') }}</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" :class="{ disabled: !contextMenu.tab?.closable }" @click="handleClose">
          <el-icon :size="14"><Close /></el-icon>
          <span>{{ t('sys.tabs.close') }}</span>
        </div>
        <div class="menu-item" @click="handleCloseLeft">
          <el-icon :size="14"><Back /></el-icon>
          <span>{{ t('sys.tabs.closeLeft') }}</span>
        </div>
        <div class="menu-item" @click="handleCloseRight">
          <el-icon :size="14"><Right /></el-icon>
          <span>{{ t('sys.tabs.closeRight') }}</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" @click="handleCloseOther">
          <el-icon :size="14"><CircleClose /></el-icon>
          <span>{{ t('sys.tabs.closeOther') }}</span>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import draggable from 'vuedraggable'
import { useTabStore, type TabItem } from '@/stores/tabs'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore, type MenuNode } from '@/stores/auth'
import { tabIconMap } from '@/config/menu'

const router = useRouter()
const { t } = useI18n()
const tabStore = useTabStore()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()

// ========== 滚动控制 ==========
const tabListRef = ref<{ $el: HTMLElement } | null>(null)
const hasOverflow = ref(false)
const canScrollLeft = ref(false)
const canScrollRight = ref(false)

function updateScrollState() {
  const el = tabListRef.value?.$el
  if (!el) return
  hasOverflow.value = el.scrollWidth > el.clientWidth
  canScrollLeft.value = el.scrollLeft > 0
  canScrollRight.value = el.scrollLeft < el.scrollWidth - el.clientWidth - 1
}

function scrollLeft() {
  const el = tabListRef.value?.$el
  if (!el) return
  el.scrollBy({ left: -200, behavior: 'smooth' })
  setTimeout(updateScrollState, 200)
}

function scrollRight() {
  const el = tabListRef.value?.$el
  if (!el) return
  el.scrollBy({ left: 200, behavior: 'smooth' })
  setTimeout(updateScrollState, 200)
}

// ponytail: 静态 tabIconMap 作兜底,动态菜单优先
function getTabIcon(path: string): string {
  if (tabIconMap[path]) return tabIconMap[path]
  const walk = (nodes: MenuNode[]): string => {
    for (const n of nodes) {
      if (n.path === path && n.icon) return n.icon
      if (n.children?.length) {
        const found = walk(n.children)
        if (found) return found
      }
    }
    return ''
  }
  return walk(authStore.menus) || 'Document'
}

// ========== 路由监听：自动添加标签页 ==========
const unwatch = router.afterEach((to) => {
  if (to.meta?.title && to.path !== '/login') {
    tabStore.addTab({
      path: to.path,
      title: to.meta.title as string,
      closable: to.path !== '/dashboard',
    })
  }
})
onUnmounted(() => unwatch())

// ========== 可排序标签（排除 Dashboard） ==========
const sortableTabs = computed({
  get: () => tabStore.tabs.filter((t) => t.closable),
  set: (newList) => {
    tabStore.tabs = [tabStore.tabs[0], ...newList]
  },
})
watch(sortableTabs, () => nextTick(updateScrollState), { deep: true })

// ========== 导航 ==========
function navigateTo(tab: TabItem) {
  tabStore.setActive(tab.path)
  router.push(tab.path)
}

function closeTab(tab: TabItem) {
  if (tab.closable) {
    const nextPath = tabStore.closeTab(tab.path)
    if (nextPath) router.push(nextPath)
  }
}

function onDragEnd() {}

// ========== 右键菜单 ==========
const contextMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  tab: null as TabItem | null,
  idx: -1,
})

function onContextMenu(e: MouseEvent, tab: TabItem, idx: number) {
  contextMenu.visible = true
  contextMenu.x = Math.min(e.clientX, window.innerWidth - 160)
  contextMenu.y = Math.min(e.clientY, window.innerHeight - 200)
  contextMenu.tab = tab
  contextMenu.idx = idx
}

function closeContextMenu() {
  contextMenu.visible = false
}

onMounted(() => {
  document.addEventListener('click', closeContextMenu)
  const el = tabListRef.value?.$el
  if (el) {
    el.addEventListener('scroll', updateScrollState)
    updateScrollState()
  }
})
onUnmounted(() => {
  document.removeEventListener('click', closeContextMenu)
  const el = tabListRef.value?.$el
  if (el) el.removeEventListener('scroll', updateScrollState)
})

function handleRefresh() {
  closeContextMenu()
  if (!contextMenu.tab) return
  const path = contextMenu.tab.path
  router.replace('/login').then(() => router.replace(path))
}

function handleClose() {
  closeContextMenu()
  if (contextMenu.tab?.closable) {
    const nextPath = tabStore.closeTab(contextMenu.tab.path)
    if (nextPath) router.push(nextPath)
  }
}

function handleCloseLeft() {
  closeContextMenu()
  if (contextMenu.tab) tabStore.closeLeft(contextMenu.tab.path)
}

function handleCloseRight() {
  closeContextMenu()
  if (contextMenu.tab) tabStore.closeRight(contextMenu.tab.path)
}

function handleCloseOther() {
  closeContextMenu()
  if (contextMenu.tab) {
    tabStore.closeOther(contextMenu.tab.path)
    router.push(contextMenu.tab.path)
  }
}
</script>

<style scoped>
.tab-bar {
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  padding: 0 8px;
  height: 34px;
  display: flex;
  align-items: center;
}

.tab-home {
  margin-right: 6px;
  cursor: pointer;
}

.tab-scroll-btn {
  flex-shrink: 0;
  cursor: pointer;
  color: var(--el-text-color-regular);
  transition: color 0.2s;
  padding: 4px;
}
.tab-scroll-btn:hover:not(.disabled) {
  color: var(--el-color-primary);
}
.tab-scroll-btn.disabled {
  color: var(--el-text-color-disabled);
  cursor: not-allowed;
}

.tab-list {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow-x: auto;
  flex: 1;
  scrollbar-width: none;
}
.tab-list::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 26px;
  padding: 0 10px;
  border-radius: 4px;
  font-size: 12px;
  color: var(--el-text-color-regular);
  background: var(--el-fill-color-light);
  cursor: grab;
  white-space: nowrap;
  flex-shrink: 0;
  transition:
    color 0.2s,
    background 0.2s;
  user-select: none;
  will-change: transform;
}

.tab-item:hover {
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.tab-item.active {
  color: #fff;
  background: var(--el-color-primary);
}

.tab-ghost {
  opacity: 0.4;
  background: var(--el-color-primary-light-7) !important;
  cursor: grabbing !important;
}
.tab-chosen {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}
.tab-drag {
  opacity: 0.9;
  cursor: grabbing !important;
}

.tab-title {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  border-radius: 50%;
  padding: 1px;
  transition: all 0.15s;
  cursor: pointer;
}
.tab-close:hover {
  background: rgba(0, 0, 0, 0.15);
  color: var(--el-color-danger);
}
.tab-item.active .tab-close:hover {
  background: rgba(255, 255, 255, 0.25);
  color: #fff;
}
</style>

<style>
.tab-context-menu {
  position: fixed;
  z-index: 9999;
  background: var(--el-bg-color-overlay, #fff);
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  padding: 4px 0;
  min-width: 150px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
}
.tab-context-menu .menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  font-size: 13px;
  color: var(--el-text-color-primary);
  cursor: pointer;
  transition: background 0.15s;
}
.tab-context-menu .menu-item:hover {
  background: var(--el-fill-color-light);
  color: var(--el-color-primary);
}
.tab-context-menu .menu-item.disabled {
  color: var(--el-text-color-disabled);
  cursor: not-allowed;
  pointer-events: none;
}
.tab-context-menu .menu-divider {
  height: 1px;
  background: var(--el-border-color-lighter);
  margin: 4px 0;
}
</style>
