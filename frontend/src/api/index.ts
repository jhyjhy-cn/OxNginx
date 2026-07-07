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

// Dashboard 数据类型
export interface DashboardData {
  nginx_running: boolean
  nginx_version: string
  worker_processes: number
  active_connections: number
  sites_count: number
  certificates_count: number
  config_files: number
  uptime: string
  nginx_conf_path: string
  os: string
  cpu_cores: number
  total_memory: string
}

// Dashboard API
export const getDashboard = async (): Promise<DashboardData> => {
  const response = await api.get('/api/dashboard')
  return response.data.data
}

// 站点 API
export const getSites = async () => {
  const response = await api.get('/api/sites')
  return response.data.data
}

export const getSite = async (id: number) => {
  const response = await api.get(`/api/sites/${id}`)
  return response.data.data
}

export const createSite = async (data: any) => {
  const response = await api.post('/api/sites', data)
  return response.data
}

export const updateSite = async (id: number, data: any) => {
  const response = await api.put(`/api/sites/${id}`, data)
  return response.data
}

export const deleteSite = async (id: number) => {
  const response = await api.delete(`/api/sites/${id}`)
  return response.data
}

// Nginx API
export const testNginxConfig = async () => {
  const response = await api.post('/api/nginx/test')
  return response.data
}

export const reloadNginx = async () => {
  const response = await api.post('/api/nginx/reload')
  return response.data
}

export const getNginxStatus = async () => {
  const response = await api.get('/api/nginx/status')
  return response.data.data
}

// 日志 API
export const getAccessLogs = async (params?: { lines?: number }) => {
  const response = await api.get('/api/log/access', { params })
  return response.data.data
}

export const getErrorLogs = async (params?: { lines?: number }) => {
  const response = await api.get('/api/log/error', { params })
  return response.data.data
}

// 备份 API
export const getBackups = async () => {
  const response = await api.get('/api/backups')
  return response.data.data
}

export const createBackup = async () => {
  const response = await api.post('/api/backups')
  return response.data
}

export const restoreBackup = async (filename: string) => {
  const response = await api.post(`/api/backups/${filename}/restore`)
  return response.data
}

export const deleteBackup = async (filename: string) => {
  const response = await api.delete(`/api/backups/${filename}`)
  return response.data
}

// SSL 证书 API
export const getCertificates = async () => {
  const response = await api.get('/api/certificates')
  return response.data.data
}

export const requestCertificate = async (data: { domain: string }) => {
  const response = await api.post('/api/certificates', data)
  return response.data
}

export const revokeCertificate = async (id: number) => {
  const response = await api.delete(`/api/certificates/${id}`)
  return response.data
}
