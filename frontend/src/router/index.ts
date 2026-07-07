import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore, type MenuNode } from '@/stores/auth'

// 组件路径 → 懒加载
const componentMap: Record<string, () => Promise<any>> = {
  Dashboard: () => import('@/views/Dashboard.vue'),
  'sites/index': () => import('@/views/sites/index.vue'),
  SSL: () => import('@/views/SSL.vue'),
  Templates: () => import('@/views/Templates.vue'),
  Upstreams: () => import('@/views/Upstreams.vue'),
  Logs: () => import('@/views/Logs.vue'),
  'files/index': () => import('@/views/files/index.vue'),
  Terminal: () => import('@/views/Terminal.vue'),
  Settings: () => import('@/views/Settings.vue'),
  RbacUsers: () => import('@/views/sys/users/index.vue'),
  RbacRoles: () => import('@/views/sys/roles/index.vue'),
  RbacRoleMenus: () => import('@/views/sys/role-menus/index.vue'),
  RbacMenus: () => import('@/views/sys/menus/index.vue'),
  RbacDepts: () => import('@/views/sys/depts/index.vue'),
  RbacPosts: () => import('@/views/sys/posts/index.vue'),
  RbacI18n: () => import('@/views/sys/i18n/index.vue'),
  RbacDicts: () => import('@/views/sys/dicts/index.vue'),
  SiteDetail: () => import('@/views/SiteDetail.vue'),
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: () => import('@/views/Login.vue'),
    },
    {
      path: '/403',
      name: 'Forbidden',
      component: () => import('@/views/Forbidden.vue'),
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

/** ponytail: 动态路由——登录后从后端菜单 addRoute */
export function setupDynamicRoutes() {
  const authStore = useAuthStore()

  const walk = (nodes: MenuNode[]) => {
    for (const n of nodes) {
      if (n.type === 'C' && n.path && n.component) {
        const loader = componentMap[n.component]
        if (loader) {
          const path = n.path.replace(/^\//, '')
          const name = n.name
          if (!router.hasRoute(name)) {
            router.addRoute('Root', {
              path,
              name,
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

  walk(authStore.menus)
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
  }

  await authStore.fetchI18n()

  if (authStore.isSuperAdmin) return true
  const need = to.meta?.permission as string | undefined
  if (need && !authStore.hasPermission(need)) {
    return { path: '/403' }
  }
  return true
})

export default router