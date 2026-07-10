export interface ParamItem {
  id: number
  key: string
  value?: string
  name: string
  group_code?: string
  remark?: string
  sort?: number
  created_at?: string
  updated_at?: string
}

export interface ParamPayload {
  key: string
  value?: string
  name: string
  group_code?: string
  remark?: string
  sort?: number
}