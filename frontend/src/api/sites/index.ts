import { getData, postData, putData, deleteData } from '@/api/http'
import type { PageQuery, PageResult } from '@/api/sys/users/type'
import type {
  Site,
  Backup,
  Upstream,
  Proxy,
  AccessRule,
  Template,
  SiteConfigFile,
} from './type'

export const listSites = () => getData<Site[]>('/api/sites')

export const listSitesWithCerts = () => getData<Site[]>('/api/sites/with-certs')

export const getSite = (id: number) => getData<Site>(`/api/sites/${id}`)

export const createSite = (payload: Record<string, unknown>) => postData<Site>('/api/sites', payload)

export const updateSite = (id: number, payload: Record<string, unknown>) =>
  putData<Site>(`/api/sites/${id}`, payload)

export const deleteSite = (id: number, params?: { remove_files?: boolean }) =>
  deleteData(`/api/sites/${id}`, params)

export const toggleSiteStatus = (id: number, payload: { status: string }) =>
  putData<unknown>(`/api/sites/${id}`, payload)

export const deploySsl = (id: number) => postData<string>(`/api/sites/${id}/deploy-ssl`)

export const batchEnable = (ids: number[]) => postData<string>('/api/sites/batch/enable', { ids })

export const batchDisable = (ids: number[]) => postData<string>('/api/sites/batch/disable', { ids })

export const batchDeleteSites = (ids: number[], remove_files?: boolean) =>
  postData<string>('/api/sites/batch/delete', { ids, remove_files: !!remove_files })

// 反向代理
export const listProxies = (siteId: number) =>
  getData<Proxy[]>(`/api/sites/${siteId}/proxies`)

export const createProxy = (siteId: number, payload: Record<string, unknown>) =>
  postData<Proxy>(`/api/sites/${siteId}/proxies`, payload)

export const updateProxy = (id: number, payload: Record<string, unknown>) =>
  putData<Proxy>(`/api/proxies/${id}`, payload)

export const deleteProxy = (id: number) => deleteData(`/api/proxies/${id}`)

// upstream
export const listUpstreams = (params?: PageQuery) =>
  getData<PageResult<Upstream>>('/api/upstreams', params)

export const getUpstream = (id: number) => getData<Upstream>(`/api/upstreams/${id}`)

export const createUpstream = (payload: Record<string, unknown>) =>
  postData<Upstream>('/api/upstreams', payload)

export const updateUpstream = (id: number, payload: Record<string, unknown>) =>
  putData<Upstream>(`/api/upstreams/${id}`, payload)

export const deleteUpstream = (id: number) => deleteData(`/api/upstreams/${id}`)

// access-rules
export const listAccessRules = () => getData<AccessRule[]>('/api/access-rules')

export const createAccessRule = (payload: Record<string, unknown>) =>
  postData<AccessRule>('/api/access-rules', payload)

export const getAccessRule = (id: number) => getData<AccessRule>(`/api/access-rules/${id}`)

export const updateAccessRule = (id: number, payload: Record<string, unknown>) =>
  putData(`/api/access-rules/${id}`, payload)

export const deleteAccessRule = (id: number) => deleteData(`/api/access-rules/${id}`)

// templates
export const listTemplates = () => getData<Template[]>('/api/templates')

export const createTemplate = (payload: Record<string, unknown>) =>
  postData<Template>('/api/templates', payload)

export const updateTemplate = (id: number, payload: Record<string, unknown>) =>
  putData<Template>(`/api/templates/${id}`, payload)

export const deleteTemplate = (id: number) => deleteData(`/api/templates/${id}`)

export const previewTemplate = (id: number, payload: Record<string, unknown>) =>
  postData<string>(`/api/templates/${id}/preview`, payload)

// 站点备份
export const listSiteBackups = (siteId: number, params?: Record<string, unknown>) =>
  getData<Backup[]>(`/api/sites/${siteId}/backups`, params)

export const createSiteBackup = (siteId: number) =>
  postData<Backup>(`/api/sites/${siteId}/backups`)

export const deleteSiteBackup = (siteId: number, filename: string) =>
  deleteData(`/api/sites/${siteId}/backups/${encodeURIComponent(filename)}`)

export const downloadSiteBackup = (siteId: number, filename: string) =>
  `/api/sites/${siteId}/backups/${encodeURIComponent(filename)}/download`

export const batchDeleteSiteBackups = (siteId: number, filenames: string[]) =>
  postData(`/api/sites/${siteId}/backups/batch-delete`, { filenames })

// nginx 配置
export const getMainConfig = () => getData<string>('/api/config/main')

export const saveMainConfig = (content: string) =>
  putData('/api/config/main', { content })

export const listConfigFiles = () =>
  getData<Array<{ name: string; enabled: boolean }>>('/api/config/files')

export const getSiteConfig = (name: string) =>
  getData<SiteConfigFile>(`/api/config/file/${name}`)

export const saveSiteConfig = (name: string, content: string) =>
  putData(`/api/config/file/${name}`, { content })

export const toggleSiteConfig = (name: string) =>
  postData(`/api/config/file/${name}/toggle`)

export const deleteSiteConfig = (name: string) =>
  deleteData(`/api/config/file/${name}`)

// 备份（/api/backups/{id} 含义是按 siteId）
export const listBackups = (siteId: number | string) =>
  getData<Backup[]>(`/api/backups/${siteId}`)

export const createBackup = (siteId: number | string) =>
  postData<Backup>(`/api/backups/${siteId}`)

export const deleteBackup = (siteId: number | string, backupId: number | string) =>
  deleteData(`/api/backups/${siteId}`, { id: backupId })

export const restoreBackup = (siteId: number | string, backupId: number | string) =>
  postData(`/api/backups/restore/${backupId}`, { site_id: siteId })

export const diffBackups = (payload: { left: string; right: string }) =>
  postData<{ diff: string }>('/api/backups/diff', payload)