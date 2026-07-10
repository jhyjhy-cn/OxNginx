import type { MenuType } from '@/enums'

export interface MenuItem {
  id: number
  parent_id: number | null
  name: string
  title: string
  icon: string | null
  path: string | null
  component: string | null
  permission: string | null
  type: MenuType
  sort: number
  children?: MenuItem[]
}