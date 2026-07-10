import { getData, postData, deleteData } from '@/api/http'
import type { PageQuery, PageResult } from '../users/type'
import type { I18nEntry, I18nUpsert } from './type'

export const listI18nLocales = () => getData<string[]>('/api/rbac/i18n/locales')

export const listI18n = (params?: PageQuery) =>
  getData<PageResult<I18nEntry>>('/api/rbac/i18n', params)

export const getI18nMessages = () =>
  getData<Record<string, string>>('/api/rbac/i18n/messages')

export const upsertI18n = (payload: I18nUpsert) => postData('/api/rbac/i18n', payload)

export const deleteI18n = (id: number) => deleteData(`/api/rbac/i18n/${id}`)