import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from 'vue-i18n'
import api from '@/api'
import type { Site } from './types'

export function useSites() {
  const { t } = useI18n()

  const sites = ref<Site[]>([])
  const selectedSites = ref<Site[]>([])
  const loading = ref(false)
  const trafficMetric = ref<'ip' | 'pv' | 'request' | 'uv'>('ip')

  async function fetchSites() {
    loading.value = true
    try {
      const response = await api.get('/api/sites/with-certs')
      if (response.data.code === 0) {
        sites.value = (response.data.data || []).map((s: Site) => {
          if (s.cert_expire_time) {
            const expireDate = new Date(s.cert_expire_time)
            const now = new Date()
            s.cert_expire_days = Math.ceil(
              (expireDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24),
            )
          }
          return s
        })
      }
    } catch (error) {
      console.error('获取站点列表失败:', error)
    } finally {
      loading.value = false
    }
  }

  async function toggleSite(site: Site, enable?: boolean) {
    const newStatus =
      enable !== undefined
        ? enable
          ? 'enabled'
          : 'disabled'
        : site.status === 'enabled'
          ? 'disabled'
          : 'enabled'
    try {
      await api.put(`/api/sites/${site.id}`, { status: newStatus })
      ElMessage.success(
        newStatus === 'enabled' ? t('common.enabled') : t('common.disabled'),
      )
      fetchSites()
    } catch (error: any) {
      ElMessage.error(
        error.response?.data?.message || t('sites.operationFailed'),
      )
    }
  }

  async function batchEnable() {
    try {
      await ElMessageBox.confirm(
        t('sites.batchEnableConfirm', { count: selectedSites.value.length }),
        t('common.tip'),
      )
      const response = await api.post('/api/sites/batch/enable', {
        ids: selectedSites.value.map((s) => s.id),
      })
      if (response.data.code === 0) {
        ElMessage.success(
          t('sites.batchEnableSuccess', { count: response.data.data.success }),
        )
        fetchSites()
      }
    } catch (error: any) {
      if (error !== 'cancel') {
        ElMessage.error(
          error.response?.data?.message || t('sites.operationFailed'),
        )
      }
    }
  }

  async function batchDisable() {
    try {
      await ElMessageBox.confirm(
        t('sites.batchDisableConfirm', { count: selectedSites.value.length }),
        t('common.tip'),
      )
      const response = await api.post('/api/sites/batch/disable', {
        ids: selectedSites.value.map((s) => s.id),
      })
      if (response.data.code === 0) {
        ElMessage.success(
          t('sites.batchDisableSuccess', { count: response.data.data.success }),
        )
        fetchSites()
      }
    } catch (error: any) {
      if (error !== 'cancel') {
        ElMessage.error(
          error.response?.data?.message || t('sites.operationFailed'),
        )
      }
    }
  }

  async function batchDelete() {
    try {
      await ElMessageBox.confirm(
        t('sites.batchDeleteConfirm', { count: selectedSites.value.length }),
        t('common.warning'),
        { type: 'warning' },
      )
      const response = await api.post('/api/sites/batch/delete', {
        ids: selectedSites.value.map((s) => s.id),
      })
      if (response.data.code === 0) {
        ElMessage.success(
          t('sites.batchDeleteSuccess', { count: response.data.data.success }),
        )
        fetchSites()
      }
    } catch (error: any) {
      if (error !== 'cancel') {
        ElMessage.error(
          error.response?.data?.message || t('sites.operationFailed'),
        )
      }
    }
  }

  return {
    sites,
    selectedSites,
    loading,
    trafficMetric,
    fetchSites,
    toggleSite,
    batchEnable,
    batchDisable,
    batchDelete,
  }
}
