<template>
  <el-form>
    <el-form-item>
      <div style="display: flex; gap: 8px; width: 100%">
        <el-input
          v-model="domainInput"
          type="textarea"
          :autosize="{ minRows: 6, maxRows: 8 }"
          :placeholder="domainPlaceholder"
          style="flex: 1"
        />
        <el-button type="primary" style="align-self: flex-end" @click="addDomains">
          {{ $t('sys.sites.addDomain') }}
        </el-button>
      </div>
    </el-form-item>
  </el-form>
  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px">
    <el-button v-if="domainSelected.length > 0" type="danger" size="small" @click="deleteSelectedDomains">
      {{ $t('common.delete') }} ({{ domainSelected.length }})
    </el-button>
    <span v-else />
    <span style="font-size: 12px; color: #909399">{{ $t('sys.sites.domainCount', { n: domains.length }) }}</span>
  </div>
  <!-- ponytail: 数据源是 props.domains 字符串数组(无网络列表)，textarea 批量添加，OnTable 无收益，保留 el-table -->
  <el-table :data="domainsDisplay" style="width: 100%" max-height="380" @selection-change="(val: DomainItem[]) => (domainSelected = val)">
    <el-table-column type="selection" width="45" />
    <el-table-column :label="$t('sys.sites.domain')">
      <template #default="{ row }">
        <el-button type="primary" link @click="openDomain(row.domain)">{{ row.domain }}</el-button>
      </template>
    </el-table-column>
    <el-table-column :label="$t('common.action')" width="80">
      <template #default="{ row }">
        <el-button type="danger" link size="small" @click="deleteDomain(row.domain)">{{ $t('common.delete') }}</el-button>
      </template>
    </el-table-column>
  </el-table>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import type { DomainItem } from '../types'
import { updateSite } from '@/api/sites'

const { t } = useI18n()

const props = defineProps<{
  siteId: number | null
  domains: string[]
}>()

const emit = defineEmits<{
  'update:domains': [value: string[]]
  saved: []
}>()

const domainInput = ref('')
const domainSelected = ref<DomainItem[]>([])

const domainsDisplay = computed(() => props.domains.map((d) => ({ domain: d })))

const domainPlaceholder = computed(
  () => `${t('sys.sites.domainHint')}\n${t('sys.sites.domainFormatIp')}\n${t('sys.sites.domainFormatPort')}\n${t('sys.sites.domainFormatIpv6')}`
)

function openDomain(domain: string) {
  window.open('http://' + domain, '_blank')
}

function addDomains() {
  const lines = domainInput.value
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean)
  const newDomains = [...props.domains]
  let added = false
  for (const d of lines) {
    if (!newDomains.includes(d)) {
      newDomains.push(d)
      added = true
    }
  }
  domainInput.value = ''
  if (added) {
    emit('update:domains', newDomains)
    saveDomains(newDomains)
  }
}

function deleteDomain(domain: string) {
  const newDomains = props.domains.filter((d) => d !== domain)
  emit('update:domains', newDomains)
  saveDomains(newDomains)
}

function deleteSelectedDomains() {
  const toDelete = new Set(domainSelected.value.map((d) => d.domain))
  const newDomains = props.domains.filter((d) => !toDelete.has(d))
  domainSelected.value = []
  emit('update:domains', newDomains)
  saveDomains(newDomains)
}

function extractPort(domains: string): string {
  const first = domains.split('\n')[0]?.trim() || ''
  const ipv6Match = first.match(/^\[.+?\]:(\d+)$/)
  if (ipv6Match) return ipv6Match[1]
  const portMatch = first.match(/:(\d+)$/)
  if (portMatch) return portMatch[1]
  return '80'
}

// 动态导入 api 避免循环依赖
async function saveDomains(domainsList: string[]) {
  if (!props.siteId) return
  try {
    const server_name = domainsList.join(' ')
    const listen = extractPort(domainsList[0] || '80')
    await updateSite(props.siteId, { server_name, listen })
    ElMessage.success(t('common.success'))
    emit('saved')
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.sites.operationFailed'))
  }
}
</script>
