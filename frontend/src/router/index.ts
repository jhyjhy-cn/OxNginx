import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: () => import('@/views/Login.vue'),
    },
    {
      path: '/',
      component: () => import('@/layouts/MainLayout.vue'),
      redirect: '/dashboard',
      children: [
        {
          path: 'dashboard',
          name: 'Dashboard',
          component: () => import('@/views/Dashboard.vue'),
          meta: { title: 'menu.dashboard' },
        },
        {
          path: 'sites',
          name: 'Sites',
          component: () => import('@/views/sites/index.vue'),
          meta: { title: 'menu.sites' },
        },
        {
          path: 'sites/:id',
          name: 'SiteDetail',
          component: () => import('@/views/SiteDetail.vue'),
          meta: { title: 'siteDetail.title' },
        },
        {
          path: 'ssl',
          name: 'SSL',
          component: () => import('@/views/SSL.vue'),
          meta: { title: 'menu.ssl' },
        },
        {
          path: 'logs',
          name: 'Logs',
          component: () => import('@/views/Logs.vue'),
          meta: { title: 'menu.logs' },
        },
        {
          path: 'config',
          name: 'ConfigEditor',
          component: () => import('@/views/ConfigEditor.vue'),
          meta: { title: 'menu.config' },
        },
        {
          path: 'upstreams',
          name: 'Upstreams',
          component: () => import('@/views/Upstreams.vue'),
          meta: { title: 'menu.upstreams' },
        },
        {
          path: 'access',
          name: 'AccessControl',
          component: () => import('@/views/AccessControl.vue'),
          meta: { title: 'menu.access' },
        },
        {
          path: 'templates',
          name: 'Templates',
          component: () => import('@/views/Templates.vue'),
          meta: { title: 'menu.templates' },
        },
        {
          path: 'files',
          name: 'Files',
          component: () => import('@/views/files/index.vue'),
          meta: { title: 'menu.files' },
        },
        {
          path: 'settings',
          name: 'Settings',
          component: () => import('@/views/Settings.vue'),
          meta: { title: 'menu.settings' },
        },
      ],
    },
  ],
})

// 路由守卫
router.beforeEach((to) => {
  const authStore = useAuthStore()

  // ponytail: vue-router 5 推荐返回值代替 next(callback)，消除 deprecation 警告
  if (to.path !== '/login' && !authStore.isAuthenticated) {
    return { path: '/login' }
  }
  return true
})

export default router
