export interface Dept {
  id: number
  parent_id: number | null
  name: string
  sort: number
  status?: string
  children?: Dept[]
}