import { getData } from '@/api/http'
import type { LogLines, OperationLog, LoginLog } from './type'

export const getAccessLog = (params?: { lines?: number }) =>
  getData<LogLines>('/api/log/access', params)

export const getErrorLog = (params?: { lines?: number }) =>
  getData<LogLines>('/api/log/error', params)

export const getLog = (type: 'access' | 'error', params?: { lines?: number }) =>
  getData<LogLines>(`/api/log/${type}`, params)

export const listOperationLogs = (params?: Record<string, unknown>) =>
  getData<{ list: OperationLog[]; total: number }>('/api/log/operation', params)

export const exportOperationLogs = () => '/api/log/operation/export'

export const listLoginLogs = (params?: Record<string, unknown>) =>
  getData<{ list: LoginLog[]; total: number }>('/api/log/login', params)

export const exportLoginLogs = () => '/api/log/login/export'