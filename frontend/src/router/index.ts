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
      component: () => import('@/views/Layout.vue'),
      redirect: '/dashboard',
      children: [
        {
          path: 'dashboard',
          name: 'Dashboard',
          component: () => import('@/views/Dashboard.vue'),
          meta: { title: '仪表盘' },
        },
        {
          path: 'sites',
          name: 'Sites',
          component: () => import('@/views/Sites.vue'),
          meta: { title: '站点管理' },
        },
        {
          path: 'sites/:id',
          name: 'SiteDetail',
          component: () => import('@/views/SiteDetail.vue'),
          meta: { title: '站点详情' },
        },
        {
          path: 'ssl',
          name: 'SSL',
          component: () => import('@/views/SSL.vue'),
          meta: { title: 'SSL证书' },
        },
        {
          path: 'logs',
          name: 'Logs',
          component: () => import('@/views/Logs.vue'),
          meta: { title: '日志' },
        },
        {
          path: 'config',
          name: 'ConfigEditor',
          component: () => import('@/views/ConfigEditor.vue'),
          meta: { title: '配置编辑' },
        },
        {
          path: 'upstreams',
          name: 'Upstreams',
          component: () => import('@/views/Upstreams.vue'),
          meta: { title: '负载均衡' },
        },
        {
          path: 'access',
          name: 'AccessControl',
          component: () => import('@/views/AccessControl.vue'),
          meta: { title: '访问控制' },
        },
        {
          path: 'templates',
          name: 'Templates',
          component: () => import('@/views/Templates.vue'),
          meta: { title: '配置模板' },
        },
        {
          path: 'settings',
          name: 'Settings',
          component: () => import('@/views/Settings.vue'),
          meta: { title: '设置' },
        },
      ],
    },
  ],
})

// 路由守卫
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()

  if (to.path !== '/login' && !authStore.isAuthenticated) {
    next('/login')
  } else {
    next()
  }
})

export default router
