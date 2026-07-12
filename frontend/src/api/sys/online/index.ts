import { getData, deleteData } from '@/api/http'

export interface OnlineItem {
  id: number
  token: string
  user_id: number
  username: string
  nickname?: string | null
  dept_id?: number | null
  dept_name?: string | null
  ip?: string | null
  os?: string | null
  browser?: string | null
  user_agent?: string | null
  created_at?: string | null
  expires_at: string
}

export interface OnlineQuery {
  page?: number
  page_size?: number
  keyword?: string
}

export const listOnline = (params: OnlineQuery = {}) =>
  getData<{ list: OnlineItem[]; total: number; page: number; page_size: number }>(
    '/api/sys/online',
    params as unknown as Record<string, unknown>
  )

export const kickOnline = (id: number) => deleteData(`/api/sys/online/${id}`)