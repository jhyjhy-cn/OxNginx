export type LocaleCode = string

export interface I18nEntry {
  id: number
  locale: string
  key: string
  value: string
  group?: string
}

export interface I18nUpsert {
  locale: string
  entries: { key: string; value: string }[]
}