export interface Role {
  id: number
  name: string
  code?: string
  remark?: string
  sort?: number
  status?: number
  menu_ids?: number[]
}

export interface RolePayload {
  name: string
  code?: string
  remark?: string
  sort?: number
  status?: number
}