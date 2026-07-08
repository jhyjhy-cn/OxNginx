import { computed } from 'vue'
import { useSettingsStore } from '@/stores/settings'

/**
 * 根据主题色生成侧边栏配色方案
 * 返回 el-menu 需要的三个颜色 + 侧边栏背景
 */
export function useSidebarTheme() {
  const settingsStore = useSettingsStore()

  const sidebarBg = computed(() => {
    const hsl = hexToHsl(settingsStore.themeColor)
    // 深色背景：保留色相，饱和度微增，亮度压到 12%
    return `hsl(${hsl.h}, ${Math.min(hsl.s + 5, 100)}%, 12%)`
  })

  const menuTextColor = computed(() => {
    return 'hsla(0, 0%, 100%, 0.65)'
  })

  const menuActiveTextColor = computed(() => {
    return '#ffffff'
  })

  /** 选中菜单项背景色：比侧边栏背景亮 10%，带半透明白色叠加 */
  const menuActiveBg = computed(() => {
    const hsl = hexToHsl(settingsStore.themeColor)
    return `hsl(${hsl.h}, ${Math.min(hsl.s + 3, 100)}%, 22%)`
  })

  /** 分割线颜色 */
  const borderColor = computed(() => {
    return 'hsla(0, 0%, 100%, 0.1)'
  })

  return {
    sidebarBg,
    menuTextColor,
    menuActiveTextColor,
    menuActiveBg,
    borderColor,
  }
}

function hexToHsl(hex: string): { h: number; s: number; l: number } {
  hex = hex.replace('#', '')
  const r = parseInt(hex.substring(0, 2), 16) / 255
  const g = parseInt(hex.substring(2, 4), 16) / 255
  const b = parseInt(hex.substring(4, 6), 16) / 255
  const max = Math.max(r, g, b),
    min = Math.min(r, g, b)
  let h = 0,
    s = 0
  const l = (max + min) / 2
  if (max !== min) {
    const d = max - min
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    switch (max) {
      case r:
        h = ((g - b) / d + (g < b ? 6 : 0)) / 6
        break
      case g:
        h = ((b - r) / d + 2) / 6
        break
      case b:
        h = ((r - g) / d + 4) / 6
        break
    }
  }
  return { h: Math.round(h * 360), s: Math.round(s * 100), l: Math.round(l * 100) }
}
