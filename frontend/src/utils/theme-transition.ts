/**
 * 暗黑模式切换动画
 * 使用 View Transition API + clip-path: circle()
 * - 切暗黑: 禁用双方默认动画，old 亮色快照从外向按钮收缩，露出底下 new 暗色
 * - 切亮色: new 亮色快照从按钮向外展开覆盖 old 暗色
 */

function getMaxRadius(x: number, y: number): number {
  const w = window.innerWidth
  const h = window.innerHeight
  return Math.max(Math.hypot(x, y), Math.hypot(w - x, y), Math.hypot(x, h - y), Math.hypot(w - x, h - y))
}

export function toggleDarkWithAnimation(isDark: boolean, event?: MouseEvent) {
  if (!document.startViewTransition) {
    toggleThemeClass(isDark)
    return
  }

  let x = window.innerWidth - 10
  let y = 10
  if (event) {
    x = event.clientX
    y = event.clientY
  }

  const transition = document.startViewTransition(() => {
    toggleThemeClass(isDark)
  })

  transition.ready.then(() => {
    const radius = getMaxRadius(x, y)

    if (isDark) {
      // 禁用双方默认动画：old 不再淡出，new 不再淡入
      document.documentElement.animate(
        { opacity: [1, 1] },
        {
          duration: 400,
          pseudoElement: '::view-transition-old(root)',
          fill: 'forwards',
        }
      )
      document.documentElement.animate(
        { opacity: [1, 1] },
        {
          duration: 400,
          pseudoElement: '::view-transition-new(root)',
          fill: 'forwards',
        }
      )
      // old 亮色快照从外向内收缩，露出底下 new 暗色快照
      document.documentElement.animate(
        { clipPath: [`circle(${radius}px at ${x}px ${y}px)`, `circle(0px at ${x}px ${y}px)`] },
        { duration: 400, easing: 'ease-in', pseudoElement: '::view-transition-old(root)', fill: 'forwards' }
      )
    } else {
      // new 亮色快照从按钮向外展开，覆盖 old 暗色快照
      document.documentElement.animate(
        { clipPath: [`circle(0px at ${x}px ${y}px)`, `circle(${radius}px at ${x}px ${y}px)`] },
        { duration: 400, easing: 'ease-out', pseudoElement: '::view-transition-new(root)' }
      )
    }
  })
}

function toggleThemeClass(isDark: boolean) {
  if (isDark) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}
