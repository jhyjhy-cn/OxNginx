export interface PageQuery {
  page?: number
  page_size?: number
  keyword?: string
  [key: string]: unknown
}

export interface PageResult<T> {
  list: T[]
  total: number
  [key: string]: unknown
}

export interface Dept {
  id: number
  name: string
  parent_id?: number | null
  children?: Dept[]
}

export interface Post {
  id: number
  name: string
  code?: string
}

export interface Role {
  id: number
  name: string
  code?: string
}

export interface UserItem {
  id: number
  username: string
  nickname?: string
  dept_id?: number | null
  dept_name?: string
  phone?: string
  email?: string
  gender?: string
  disabled: number
  post_id?: number | null
  role_ids?: number[]
  remark?: string
  created_at?: string
}

export interface UserPayload {
  username: string
  password?: string
  nickname?: string
  dept_id?: number | null
  phone?: string
  email?: string
  gender?: string
  disabled?: number
  post_id?: number | null
  role_ids?: number[]
  remark?: string
}