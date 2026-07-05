import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface FileTab {
  id: string
  title: string
  path: string
}

export const useFilesStore = defineStore('files', () => {
  const tabs = ref<FileTab[]>([])
  const activeTabId = ref('')
  const lastPath = ref('') // 兼容旧数据

  const activeTab = computed(() => tabs.value.find(t => t.id === activeTabId.value))

  function addTab(path?: string) {
    const p = path || lastPath.value || ''
    const segs = p.replace(/\\/g, '/').split('/').filter(Boolean)
    const title = segs[segs.length - 1] || '/'
    const id = crypto.randomUUID()
    tabs.value.push({ id, title, path: p })
    activeTabId.value = id
    return id
  }

  function closeTab(id: string) {
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx === -1 || tabs.value.length <= 1) return // 至少保留一个
    tabs.value.splice(idx, 1)
    if (activeTabId.value === id) {
      const next = tabs.value[Math.min(idx, tabs.value.length - 1)]
      activeTabId.value = next.id
    }
  }

  function closeLeft(id: string) {
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx <= 0) return
    tabs.value = tabs.value.filter((t, i) => i >= idx || t.id === activeTabId.value)
    if (!tabs.value.find(t => t.id === activeTabId.value)) {
      activeTabId.value = tabs.value[0].id
    }
  }

  function closeRight(id: string) {
    const idx = tabs.value.findIndex(t => t.id === id)
    if (idx === -1) return
    tabs.value = tabs.value.filter((t, i) => i <= idx || t.id === activeTabId.value)
    if (!tabs.value.find(t => t.id === activeTabId.value)) {
      activeTabId.value = tabs.value[tabs.value.length - 1].id
    }
  }

  function closeOther(id: string) {
    tabs.value = tabs.value.filter(t => t.id === id)
    activeTabId.value = id
  }

  function reorder(newTabs: FileTab[]) {
    tabs.value = newTabs
  }

  function setActiveTab(id: string) {
    activeTabId.value = id
  }

  function updateTabPath(id: string, path: string) {
    const tab = tabs.value.find(t => t.id === id)
    if (!tab) return
    tab.path = path
    const segs = path.replace(/\\/g, '/').split('/').filter(Boolean)
    tab.title = segs[segs.length - 1] || '/'
    lastPath.value = path // 兼容
  }

  return { tabs, activeTabId, activeTab, lastPath, addTab, closeTab, closeLeft, closeRight, closeOther, setActiveTab, updateTabPath, reorder }
}, {
  persist: true,
})
