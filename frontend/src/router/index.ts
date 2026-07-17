import { MenuType } from '@/consts'
import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore, type MenuNode } from '@/stores/auth'
import { useWsStore } from '@/stores/ws'

// 自动扫描所有 views 下的 .vue 文件, key 格式: ../views/dashboard/index.vue
const modules = import.meta.glob('../views/**/*.vue')

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: modules['../views/login/index.vue'],
    },
    {
      path: '/403',
      name: 'Forbidden',
      component: modules['../views/forbidden/index.vue'],
      meta: { title: 'forbidden.title' },
    },
    {
      path: '/',
      name: 'Root',
      component: () => import('@/layouts/MainLayout.vue'),
      redirect: '/dashboard',
      children: [],
    },
  ],
})

function loadComponent(component: string) {
  return modules[`../views/${component}.vue`]
}

/** ponytail: 从菜单数组注册动态路由 */
function addRoutesFromMenus(menus: MenuNode[]) {
  const walk = (nodes: MenuNode[]) => {
    for (const n of nodes) {
      if (n.type === MenuType.Menu && n.path && n.component) {
        const loader = loadComponent(n.component)
        if (loader) {
          const path = n.path.replace(/^\//, '')
          if (!router.hasRoute(n.name)) {
            router.addRoute('Root', {
              path,
              name: n.name,
              component: loader,
              meta: {
                title: n.title,
                permission: n.permission ?? undefined,
                icon: n.icon ?? undefined,
              },
            })
          }
        }
      }
      if (n.children?.length) walk(n.children)
    }
  }
  walk(menus)
}

/** ponytail: 登录后调用 */
export function setupDynamicRoutes() {
  addRoutesFromMenus(useAuthStore().menus)
}

/** ponytail: app.mount 前调用，从 localStorage 恢复路由，避免首次解析时告警 */
export function restoreDynamicRoutes() {
  const authStore = useAuthStore()
  if (authStore.isAuthenticated && authStore.menus.length > 0) {
    addRoutesFromMenus(authStore.menus)
  }
}

router.beforeEach(async (to) => {
  const authStore = useAuthStore()

  if (to.path !== '/login' && !authStore.isAuthenticated) {
    return { path: '/login' }
  }

  if (authStore.isAuthenticated) {
    if (authStore.menus.length === 0) {
      await authStore.fetchRbacInfo()
    }
    setupDynamicRoutes()
    // ponytail: 已登录态刷新页面也要建立 ws，监听 kick
    useWsStore().setEventListener((frame) => {
      if (frame.cmd !== 'event') return
      if (frame.payload.type === 'Kick') {
        useAuthStore().logout()
      }
    })
  }

  // ponytail: 国际化已迁回前端 ts，无需再拉 DB

  if (authStore.isSuperAdmin) return true
  const need = to.meta?.permission as string | undefined
  if (need && !authStore.hasPermission(need)) {
    return { path: '/403' }
  }
  return true
})

export default router
