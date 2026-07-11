import type { LoginLogType, LogStatus } from '@/consts'

export interface LogLines {
  lines: string[]
}

export interface OperationLog {
  id: number
  trace_id?: string | null
  username?: string
  module?: string | null
  action?: string
  method?: string | null
  uri?: string | null
  ip?: string | null
  status?: LogStatus
  cost_ms?: number | null
  request_body?: string | null
  response_body?: string | null
  created_at?: string | null
  [key: string]: unknown
}

export interface LoginLog {
  id: number
  username?: string
  ip?: string | null
  os?: string | null
  browser?: string | null
  type?: LoginLogType
  status?: LogStatus
  created_at?: string | null
  [key: string]: unknown
}