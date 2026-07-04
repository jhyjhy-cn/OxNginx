import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toggleDarkWithAnimation } from '@/utils/theme-transition'

export type LayoutMode = 'sidebar-double' | 'sidebar-tree' | 'top-tree'

export const useSettingsStore = defineStore('settings', () => {
  const locale = ref<'zh-CN' | 'en-US'>('zh-CN')
  const themeColor = ref('#409EFF')
  const darkMode = ref(false)
  const layoutMode = ref<LayoutMode>('sidebar-double')
  const showTabs = ref(true)
  const showTabIcons = ref(true)
  const sidebarCollapsed = ref(false)

  function setLocale(val: 'zh-CN' | 'en-US') {
    locale.value = val
  }

  function setThemeColor(color: string) {
    themeColor.value = color
    applyThemeColor(color)
  }

  function toggleDarkMode(event?: MouseEvent) {
    darkMode.value = !darkMode.value
    toggleDarkWithAnimation(darkMode.value, event)
  }

  /** 应用主题色到 CSS 变量 */
  function applyThemeColor(hex: string) {
    const el = document.documentElement
    el.style.setProperty('--el-color-primary', hex)
    // 生成 light-3 ~ light-9 的浅色
    for (let i = 1; i <= 9; i++) {
      el.style.setProperty(
        `--el-color-primary-light-${i}`,
        mixColor(hex, '#ffffff', i * 10)
      )
    }
    // 生成 dark-2 的深色
    el.style.setProperty('--el-color-primary-dark-2', mixColor(hex, '#000000', 20))
  }

  /** 应用暗黑模式 */
  function applyDarkMode(isDark: boolean) {
    const el = document.documentElement
    if (isDark) {
      el.classList.add('dark')
    } else {
      el.classList.remove('dark')
    }
  }

  /** 设置布局模式 */
  function setLayoutMode(mode: LayoutMode) {
    layoutMode.value = mode
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  /** 初始化：从持久化 state 恢复到 DOM */
  function initTheme() {
    applyThemeColor(themeColor.value)
    applyDarkMode(darkMode.value)
  }

  return {
    locale,
    themeColor,
    darkMode,
    layoutMode,
    showTabs,
    showTabIcons,
    sidebarCollapsed,
    setLocale,
    setThemeColor,
    toggleDarkMode,
    setLayoutMode,
    toggleSidebar,
    initTheme,
  }
}, {
  persist: true,
})

/**
 * 将颜色与白色/黑色按比例混合
 * @param color  原色 hex
 * @param mix    目标色 hex
 * @param weight 混合比例 0-100（越大越接近 mix）
 */
function mixColor(color: string, mix: string, weight: number): string {
  const c = hexToRgb(color)
  const m = hexToRgb(mix)
  const w = weight / 100
  const r = Math.round(c.r + (m.r - c.r) * w)
  const g = Math.round(c.g + (m.g - c.g) * w)
  const b = Math.round(c.b + (m.b - c.b) * w)
  return `rgb(${r}, ${g}, ${b})`
}

function hexToRgb(hex: string) {
  hex = hex.replace('#', '')
  return {
    r: parseInt(hex.substring(0, 2), 16),
    g: parseInt(hex.substring(2, 4), 16),
    b: parseInt(hex.substring(4, 6), 16),
  }
}
