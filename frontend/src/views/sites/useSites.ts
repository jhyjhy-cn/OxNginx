import { ref, computed } from 'vue'
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

  // ponytail: 站点数量少,后端 /api/sites/with-certs 一次返回全部且不分页,
  // 故在客户端切片分页,不动后端。升级路径:数据量大时改为后端分页(参考 roles 的 useCrud)
  const page = ref(1)
  const pageSize = ref(20)
  const pagedSites = computed(() => {
    const start = (page.value - 1) * pageSize.value
    return sites.value.slice(start, start + pageSize.value)
  })
  function onPageChange(p: number, s: number) {
    page.value = p
    pageSize.value = s
  }

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
    pagedSites,
    page,
    pageSize,
    onPageChange,
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