import axios from 'axios'
import { useAuthStore } from '@/stores/auth'
import router from '@/router'

const api = axios.create({
  baseURL: '',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 请求拦截器
api.interceptors.request.use(
  (config) => {
    const authStore = useAuthStore()
    if (authStore.token) {
      config.headers.Authorization = `Bearer ${authStore.token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// 响应拦截器
api.interceptors.response.use(
  (response) => {
    return response
  },
  (error) => {
    // ponytail: 打 console 暴露后端实际返回(text/plain axum 错误体 / 网络错),便于排查 4xx/5xx
    console.error('[http]', error.config?.method?.toUpperCase(), error.config?.url, '->',
      error.response?.status, error.response?.data ?? error.message)
    if (error.response?.status === 401) {
      const authStore = useAuthStore()
      authStore.logout()
      router.push('/login')
    } else if (error.response?.status === 403) {
      router.push('/403')
    }
    return Promise.reject(error)
  }
)

export default api
