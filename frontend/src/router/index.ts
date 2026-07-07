import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
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

// 静态兜底路由(动态注册失败时用)
const fallbackChildren: RouteRecordRaw[] = [
  { path: 'dashboard', name: 'Dashboard', component: componentMap.Dashboard, meta: { title: 'menu.dashboard', permission: 'sys:dashboard:view' } },
  { path: 'sites', name: 'Sites', component: componentMap['sites/index'], meta: { title: 'menu.sites', permission: 'sys:site:view' } },
  { path: 'sites/:id', name: 'SiteDetail', component: componentMap.SiteDetail, meta: { title: 'siteDetail.title', permission: 'sys:site:view' } },
  { path: 'ssl', name: 'SSL', component: componentMap.SSL, meta: { title: 'menu.ssl', permission: 'sys:ssl:view' } },
  { path: 'logs', name: 'Logs', component: componentMap.Logs, meta: { title: 'menu.logs', permission: 'sys:log:view' } },
  { path: 'upstreams', name: 'Upstreams', component: componentMap.Upstreams, meta: { title: 'menu.upstreams', permission: 'sys:upstream:view' } },
  { path: 'templates', name: 'Templates', component: componentMap.Templates, meta: { title: 'menu.templates', permission: 'sys:template:view' } },
  { path: 'files', name: 'Files', component: componentMap['files/index'], meta: { title: 'menu.files', permission: 'sys:file:view' } },
  { path: 'terminal', name: 'Terminal', component: componentMap.Terminal, meta: { title: 'menu.terminal', permission: 'sys:terminal:view' } },
  { path: 'settings', name: 'Settings', component: componentMap.Settings, meta: { title: 'menu.settings', permission: 'sys:config:view' } },
  { path: 'settings/rbac/users', name: 'RbacUsers', component: componentMap.RbacUsers, meta: { title: 'menu.rbacUsers', permission: 'sys:user:manage' } },
  { path: 'settings/rbac/roles', name: 'RbacRoles', component: componentMap.RbacRoles, meta: { title: 'menu.rbacRoles', permission: 'sys:role:manage' } },
  { path: 'settings/rbac/role/:id', name: 'RbacRoleMenus', component: componentMap.RbacRoleMenus, meta: { title: 'menu.rbacRoles', permission: 'sys:role:manage' } },
  { path: 'settings/rbac/menus', name: 'RbacMenus', component: componentMap.RbacMenus, meta: { title: 'menu.rbacMenus', permission: 'sys:menu:manage' } },
  { path: 'settings/rbac/depts', name: 'RbacDepts', component: componentMap.RbacDepts, meta: { title: 'menu.rbacDepts', permission: 'sys:user:manage' } },
  { path: 'settings/rbac/posts', name: 'RbacPosts', component: componentMap.RbacPosts, meta: { title: 'menu.rbacPosts', permission: 'sys:user:manage' } },
  { path: 'settings/rbac/i18n', name: 'RbacI18n', component: componentMap.RbacI18n, meta: { title: 'menu.rbacI18n', permission: 'sys:config:view' } },
]

/** 注册静态兜底路由(已登录但 store 没菜单时用) */
export function registerFallbackRoutes() {
  for (const r of fallbackChildren) {
    if (!router.hasRoute(r.name as string)) {
      router.addRoute('Root', r)
    }
  }
}

/** ponytail: 动态路由——登录后从后端菜单 addRoute */
export function setupDynamicRoutes() {
  const authStore = useAuthStore()
  const added = new Set<string>()

  const walk = (nodes: MenuNode[]) => {
    for (const n of nodes) {
      if (n.type === 'C' && n.path && n.component) {
        const loader = componentMap[n.component]
        if (loader && !added.has(n.path)) {
          const path = n.path.replace(/^\//, '')
          const name = n.name
          // ponytail: 跳过 SiteDetail 这种特殊 path,保留静态兜底
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
          added.add(n.path)
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

  // ponytail: 已登录时,确保路由已注册 + i18n 已加载(刷新场景)
  let needReroute = false
  if (authStore.isAuthenticated) {
    const resolved = router.resolve(to)
    if (!resolved.matched.length) {
      if (authStore.menus.length === 0) {
        await authStore.fetchRbacInfo()
      }
      if (authStore.menus.length > 0) {
        setupDynamicRoutes()
      } else {
        registerFallbackRoutes()
      }
      needReroute = true
    }
    // 每次导航都确保 i18n 已合并（幂等，DB 翻译覆盖静态）
    await authStore.fetchI18n()
  }

  if (needReroute) {
    return { path: to.fullPath, replace: true }
  }

  if (authStore.isSuperAdmin) return true
  const need = to.meta?.permission as string | undefined
  if (need && !authStore.hasPermission(need)) {
    return { path: '/403' }
  }
  return true
})

export default router