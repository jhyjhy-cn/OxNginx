import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface TabItem {
  path: string
  title: string // i18n key，如 'menu.dashboard'
  closable: boolean
}

const DEFAULT_TAB: TabItem = { path: '/dashboard', title: 'menu.dashboard', closable: false }

export const useTabStore = defineStore(
  'tabs',
  () => {
    const tabs = ref<TabItem[]>([DEFAULT_TAB])
    const activePath = ref('/dashboard')

    /** 添加标签页（已存在则只激活） */
    function addTab(tab: TabItem) {
      const exists = tabs.value.find((t) => t.path === tab.path)
      if (!exists) {
        tabs.value.push(tab)
      }
      activePath.value = tab.path
    }

    /** 关闭标签页 */
    function closeTab(path: string) {
      const idx = tabs.value.findIndex((t) => t.path === path)
      if (idx === -1 || !tabs.value[idx].closable) return
      tabs.value.splice(idx, 1)
      // 如果关闭的是当前激活的，激活相邻标签
      if (activePath.value === path) {
        const next = tabs.value[idx] || tabs.value[idx - 1] || tabs.value[0]
        activePath.value = next.path
      }
      return activePath.value
    }

    /** 关闭左侧 */
    function closeLeft(path: string) {
      const idx = tabs.value.findIndex((t) => t.path === path)
      if (idx <= 0) return
      tabs.value = [...tabs.value.filter((t, i) => i === 0 || i >= idx || !t.closable)]
      // 确保当前激活的还在
      if (!tabs.value.find((t) => t.path === activePath.value)) {
        activePath.value = tabs.value[0].path
      }
    }

    /** 关闭右侧 */
    function closeRight(path: string) {
      const idx = tabs.value.findIndex((t) => t.path === path)
      if (idx === -1) return
      tabs.value = tabs.value.filter((t, i) => i <= idx || !t.closable)
      if (!tabs.value.find((t) => t.path === activePath.value)) {
        activePath.value = tabs.value[tabs.value.length - 1].path
      }
    }

    /** 关闭其他 */
    function closeOther(path: string) {
      tabs.value = tabs.value.filter((t) => !t.closable || t.path === path)
      activePath.value = path
    }

    /** 重排标签 */
    function reorder(fromIdx: number, toIdx: number) {
      const item = tabs.value.splice(fromIdx, 1)[0]
      tabs.value.splice(toIdx, 0, item)
    }

    /** 设置激活 */
    function setActive(path: string) {
      activePath.value = path
    }

    return {
      tabs,
      activePath,
      addTab,
      closeTab,
      closeLeft,
      closeRight,
      closeOther,
      reorder,
      setActive,
    }
  },
  {
    persist: true,
  }
)
