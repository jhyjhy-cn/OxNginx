import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import {
  listSitesWithCerts,
  toggleSiteStatus,
  batchEnable,
  batchDisable,
  batchDeleteSites,
} from '@/api/sites'
import type { Site } from '@/views/sites/types'

export type { Site } from '@/views/sites/types'
import { useMessage } from '@/hooks'

export function useSites() {
  const { t } = useI18n()
  const { confirm } = useMessage()

  const sites = ref<Site[]>([])
  const selectedSites = ref<Site[]>([])
  const loading = ref(false)
  const trafficMetric = ref<'ip' | 'pv' | 'request' | 'uv'>('ip')

  async function fetchSites() {
    loading.value = true
    try {
      const data = ((await listSitesWithCerts()) || []) as unknown as Site[]
      sites.value = data.map((s: Site) => {
        if (s.cert_expire_time) {
          const expireDate = new Date(s.cert_expire_time as string)
          const now = new Date()
          ;(s as any).cert_expire_days = Math.ceil((expireDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
        }
        return s
      })
    } catch (error) {
      console.error('获取站点列表失败:', error)
    } finally {
      loading.value = false
    }
  }

  async function toggleSite(site: Site, enable?: boolean) {
    const newStatus = enable !== undefined ? (enable ? 'enabled' : 'disabled') : site.status === 'enabled' ? 'disabled' : 'enabled'
    try {
      await toggleSiteStatus(site.id, { status: newStatus })
      ElMessage.success(newStatus === 'enabled' ? t('common.enabled') : t('common.disabled'))
      fetchSites()
    } catch (error: any) {
      ElMessage.error(error.message || t('sys.sites.operationFailed'))
    }
  }

  async function batchEnableSites() {
    const ok = await confirm({
      message: 'sys.sites.batchEnableConfirm',
      params: { count: selectedSites.value.length },
    })
    if (!ok) return
    try {
      const msg = await batchEnable(selectedSites.value.map((s) => s.id))
      ElMessage.success(msg || t('common.success'))
      fetchSites()
    } catch (error: any) {
      ElMessage.error(error.message || t('sys.sites.operationFailed'))
    }
  }

  async function batchDisableSites() {
    const ok = await confirm({
      message: 'sys.sites.batchDisableConfirm',
      params: { count: selectedSites.value.length },
    })
    if (!ok) return
    try {
      const msg = await batchDisable(selectedSites.value.map((s) => s.id))
      ElMessage.success(msg || t('common.success'))
      fetchSites()
    } catch (error: any) {
      ElMessage.error(error.message || t('sys.sites.operationFailed'))
    }
  }

  async function batchDelete() {
    const ok = await confirm({
      message: 'sys.sites.batchDeleteConfirm',
      params: { count: selectedSites.value.length },
    })
    if (!ok) return
    try {
      const msg = await batchDeleteSites(selectedSites.value.map((s) => s.id))
      ElMessage.success(msg || t('common.success'))
      fetchSites()
    } catch (error: any) {
      ElMessage.error(error.message || t('sys.sites.operationFailed'))
    }
  }

  return {
    sites,
    selectedSites,
    loading,
    trafficMetric,
    fetchSites,
    toggleSite,
    batchEnable: batchEnableSites,
    batchDisable: batchDisableSites,
    batchDelete,
  }
}