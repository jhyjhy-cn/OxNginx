import { postData } from '@/api/http'
import api from '@/api'
import type { NginxTestResult } from './type'

export const testNginxConfig = () => postData<NginxTestResult>('/api/nginx/test')

export const reloadNginx = () => postData<string>('/api/nginx/reload')

export const startNginx = () => postData<string>('/api/nginx/start')

export const stopNginx = () => postData<string>('/api/nginx/stop')

export const restartNginx = () => postData<string>('/api/nginx/restart')

export const installNginx = async () => {
  const r = await api.post('/api/nginx/install', null, { timeout: 300000 })
  return r.data.data
}