<template>
  <!-- ponytail: 自定义 sort-method + 列头下拉(流量指标)非 OnTable 能力，硬套反而更长且丢功能，保留原生 el-table -->
  <el-table :data="sites" style="width: 100%" v-loading="loading" @selection-change="(val: Site[]) => emit('selection-change', val)">
    <el-table-column type="selection" width="55" />
    <!-- 网站名 -->
    <el-table-column prop="name" :label="$t('sys.sites.siteName')" width="150">
      <template #default="{ row }">
        <el-button type="primary" link @click="emit('edit', row)">{{ row.name }}</el-button>
      </template>
    </el-table-column>
    <!-- 状态（可排序） -->
    <el-table-column
      :label="$t('common.status')"
      width="100"
      sortable
      :sort-method="(a: Site, b: Site) => (a.status === 'enabled' ? 0 : 1) - (b.status === 'enabled' ? 0 : 1)"
    >
      <template #default="{ row }">
        <el-switch
          :model-value="row.status === 'enabled'"
          inline-prompt
          active-text="启"
          inactive-text="停"
          @change="(val: boolean) => emit('toggle', row, val)"
        />
      </template>
    </el-table-column>
    <!-- 备份 -->
    <el-table-column :label="$t('sys.sites.backup')" width="100">
      <template #default="{ row }">
        <el-button v-if="row.backup_count > 0" type="primary" link @click="emit('open-backup', row)">
          {{ $t('sys.sites.hasBackup', { n: row.backup_count }) }}
        </el-button>
        <el-button v-else type="info" link @click="emit('open-backup', row)">
          {{ $t('sys.sites.noBackup') }}
        </el-button>
      </template>
    </el-table-column>
    <!-- 根目录 -->
    <el-table-column :label="$t('sys.sites.rootPath')" min-width="180" show-overflow-tooltip>
      <template #default="{ row }">
        <el-button v-if="row.root_path" type="primary" link @click="emit('open-file-manager', row.root_path)">
          {{ row.root_path }}
        </el-button>
        <span v-else>{{ row.proxy_pass || '-' }}</span>
      </template>
    </el-table-column>
    <!-- 日流量 -->
    <el-table-column width="140">
      <template #header>
        <el-dropdown @command="(cmd: string) => emit('update:trafficMetric', cmd)" trigger="click">
          <span class="traffic-header">
            {{ $t(`sites.traffic.${trafficMetric}`) }}
            <el-icon><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="ip" :class="{ active: trafficMetric === 'ip' }">{{ $t('sys.sites.traffic.ip') }}</el-dropdown-item>
              <el-dropdown-item command="pv" :class="{ active: trafficMetric === 'pv' }">{{ $t('sys.sites.traffic.pv') }}</el-dropdown-item>
              <el-dropdown-item command="request" :class="{ active: trafficMetric === 'request' }">
                {{ $t('sys.sites.traffic.request') }}
              </el-dropdown-item>
              <el-dropdown-item command="uv" :class="{ active: trafficMetric === 'uv' }">{{ $t('sys.sites.traffic.uv') }}</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </template>
      <template #default="{ row }">
        <span>{{ row.traffic?.[trafficMetric] ?? '-' }}</span>
      </template>
    </el-table-column>
    <!-- 到期时间 -->
    <el-table-column width="150" sortable :sort-method="sortExpireTime">
      <template #header>
        <span>{{ $t('sys.sites.expireTime') }}</span>
      </template>
      <template #default="{ row }">
        <el-tag v-if="!row.expire_time" size="small">{{ $t('sys.sites.permanent') }}</el-tag>
        <span v-else>{{ row.expire_time }}</span>
      </template>
    </el-table-column>
    <!-- 备注 -->
    <el-table-column prop="remark" :label="$t('sys.sites.remark')" width="120" show-overflow-tooltip>
      <template #default="{ row }">
        {{ row.remark || '-' }}
      </template>
    </el-table-column>
    <!-- SSL证书 -->
    <el-table-column :label="$t('sys.sites.sslCert')" width="140" sortable :sort-method="sortCert">
      <template #default="{ row }">
        <el-tag
          v-if="row.ssl === 1 && row.cert_expire_days != null"
          :type="row.cert_expire_days > 30 ? 'success' : row.cert_expire_days > 7 ? 'warning' : 'danger'"
          size="small"
        >
          {{ $t('sys.sites.daysRemaining', { n: row.cert_expire_days }) }}
        </el-tag>
        <el-tag v-else-if="row.ssl === 1" type="success" size="small">{{ $t('sys.sites.deployed') }}</el-tag>
        <el-tag v-else type="info" size="small">{{ $t('sys.sites.notDeployed') }}</el-tag>
      </template>
    </el-table-column>
    <!-- 操作 -->
    <el-table-column :label="$t('common.action')" width="200" fixed="right">
      <template #default="{ row }">
        <el-button type="primary" link @click="emit('edit', row)">{{ $t('common.edit') }}</el-button>
        <el-button type="primary" link @click="emit('deploy-ssl', row)" :loading="row._sslLoading">
          {{ $t('sys.sites.sslDeploy') }}
        </el-button>
        <el-button type="danger" link @click="emit('delete', row)">{{ $t('common.delete') }}</el-button>
      </template>
    </el-table-column>
  </el-table>
</template>

<script setup lang="ts">
import { ArrowDown } from '@element-plus/icons-vue'
import type { Site } from './types'

defineProps<{
  sites: Site[]
  loading: boolean
  trafficMetric: string
}>()

const emit = defineEmits<{
  edit: [site: Site]
  'open-backup': [site: Site]
  toggle: [site: Site, enable: boolean]
  'deploy-ssl': [site: Site]
  delete: [site: Site]
  'selection-change': [sites: Site[]]
  'open-file-manager': [path: string]
  'update:trafficMetric': [metric: string]
}>()

function sortExpireTime(a: Site, b: Site) {
  const va = a.expire_time ? new Date(a.expire_time).getTime() : Infinity
  const vb = b.expire_time ? new Date(b.expire_time).getTime() : Infinity
  return va - vb
}

function sortCert(a: Site, b: Site) {
  const va = a.cert_expire_days ?? 9999
  const vb = b.cert_expire_days ?? 9999
  return va - vb
}
</script>

<style scoped>
.traffic-header {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
}
.traffic-header:hover {
  color: var(--el-color-primary);
}
</style>
