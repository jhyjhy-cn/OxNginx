export interface Dict {
  id?: number
  name: string
  code: string
  remark?: string
  status: string
  items?: DictItem[]
}

export interface DictItem {
  id?: number
  dict_id?: number
  label: string
  value: string
  sort: number
  status: string
}