<template>
  <!--
    ponytail: 原注释说自定义排序 + 列头下拉"非 OnTable 能力"——已给 OnTable 补上
    sortMethod(类型透传)和 headerSlot(表头插槽透传),现在可以无损迁移。
  -->
  <OnTable
    :data="sites"
    :columns="columns"
    :loading="loading"
    :pagination="pagination"
    :options="{ height: 'auto', rowKey: 'id' }"
    @selectionChange="(val: Site[]) => emit('selection-change', val)"
    @command="onCommand"
    @reload="emit('reload')"
    @page-change="(p: number, s: number) => emit('page-change', p, s)"
  >
    <!-- 工具栏左侧透传(添加/批量按钮由父组件注入) -->
    <template #toolbar-left>
      <slot name="toolbar-left" />
    </template>

    <!-- 网站名 -->
    <template #name="{ row }">
      <el-button type="primary" link @click="emit('edit', row)">{{ row.name }}</el-button>
    </template>

    <!-- 状态 -->
    <template #status="{ row }">
      <el-switch
        :model-value="row.status === 'enabled'"
        inline-prompt
        active-text="启"
        inactive-text="停"
        @change="(val: boolean) => emit('toggle', row, val)"
      />
    </template>

    <!-- 备份 -->
    <template #backup="{ row }">
      <el-button v-if="row.backup_count > 0" type="primary" link @click="emit('open-backup', row)">
        {{ $t('sys.sites.hasBackup', { n: row.backup_count }) }}
      </el-button>
      <el-button v-else type="info" link @click="emit('open-backup', row)">
        {{ $t('sys.sites.noBackup') }}
      </el-button>
    </template>

    <!-- 根目录 -->
    <template #root="{ row }">
      <el-button v-if="row.root_path" type="primary" link @click="emit('open-file-manager', row.root_path)">
        {{ row.root_path }}
      </el-button>
      <span v-else>{{ row.proxy_pass || '-' }}</span>
    </template>

    <!-- 日流量:列头下拉切换指标 -->
    <template #trafficHeader>
      <el-dropdown @command="(cmd: string) => emit('update:trafficMetric', cmd)" trigger="click">
        <span class="traffic-header">
          {{ $t(`sys.sites.traffic.${trafficMetric}`) }}
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
    <template #traffic="{ row }">
      <span>{{ row.traffic?.[trafficMetric] ?? '-' }}</span>
    </template>

    <!-- 到期时间 -->
    <template #expire="{ row }">
      <el-tag v-if="!row.expire_time" size="small">{{ $t('sys.sites.permanent') }}</el-tag>
      <span v-else>{{ row.expire_time }}</span>
    </template>

    <!-- 备注 -->
    <template #remark="{ row }">{{ row.remark || '-' }}</template>

    <!-- SSL证书 -->
    <template #ssl="{ row }">
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
  </OnTable>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ArrowDown } from '@element-plus/icons-vue'
import OnTable from '@/components/OnTable/index.vue'
import type { TableColumn } from '@/components/OnTable/types'
import type { Site } from './types'

const props = withDefaults(
  defineProps<{
    sites: Site[]
    loading: boolean
    trafficMetric: string
    pagination?: boolean | { total?: number; currentPage?: number; pageSize?: number }
  }>(),
  { pagination: false }
)

const emit = defineEmits<{
  edit: [site: Site]
  'open-backup': [site: Site]
  toggle: [site: Site, enable: boolean]
  'deploy-ssl': [site: Site]
  delete: [site: Site]
  'selection-change': [sites: Site[]]
  'open-file-manager': [path: string]
  'update:trafficMetric': [metric: string]
  'page-change': [page: number, size: number]
  reload: []
}>()

// ---- 客户端排序(透传 el-table-column sort-method)----
function sortStatus(a: Site, b: Site) {
  return (a.status === 'enabled' ? 0 : 1) - (b.status === 'enabled' ? 0 : 1)
}

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

const columns = computed<TableColumn[]>(() => [
  { type: 'selection', width: 55 },
  { prop: 'name', label: 'sys.sites.siteName', width: 150, slot: 'name' },
  { label: 'common.status', width: 100, sortable: true, sortMethod: sortStatus, slot: 'status' },
  { label: 'sys.sites.backup', width: 100, slot: 'backup' },
  { label: 'sys.sites.rootPath', minWidth: 180, showOverflowTooltip: true, slot: 'root' },
  { label: `sys.sites.traffic.${props.trafficMetric}`, width: 140, headerSlot: 'trafficHeader', slot: 'traffic' },
  { label: 'sys.sites.expireTime', width: 150, sortable: true, sortMethod: sortExpireTime, slot: 'expire' },
  { prop: 'remark', label: 'sys.sites.remark', width: 120, showOverflowTooltip: true, slot: 'remark' },
  { label: 'sys.sites.sslCert', width: 140, sortable: true, sortMethod: sortCert, slot: 'ssl' },
  {
    label: 'common.action',
    width: 200,
    fixed: 'right',
    buttons: [
      { name: 'common.edit', command: 'edit', size: 'small' },
      { name: 'sys.sites.sslDeploy', command: 'deploySsl', size: 'small' },
      { name: 'common.delete', command: 'delete', type: 'danger', size: 'small' },
    ],
  },
])

function onCommand(command: string | number, row: Site) {
  if (command === 'edit') emit('edit', row)
  else if (command === 'deploySsl') emit('deploy-ssl', row)
  else if (command === 'delete') emit('delete', row)
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
/* el-dropdown 根是 inline-flex,比表头行盒(23px)矮,默认基线对齐显得偏上 → 垂直居中 */
.el-dropdown {
  vertical-align: middle;
}
</style>
