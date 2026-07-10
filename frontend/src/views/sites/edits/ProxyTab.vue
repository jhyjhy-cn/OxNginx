<template>
  <div style="display: flex; gap: 8px; margin-bottom: 12px">
    <el-button type="primary" size="small" @click="openProxyForm()">添加</el-button>
    <el-button size="small" @click="fetchProxies">刷新</el-button>
  </div>
  <OnTable
    :data="proxyList"
    :columns="tableColumns"
    :loading="proxyLoading"
    :pagination="false"
    @command="handleCommand"
  >
    <template #cache="{ row }">
      <el-tag :type="row.cache === 1 ? 'success' : 'info'" size="small">
        {{ row.cache === 1 ? '已开启' : '已关闭' }}
      </el-tag>
    </template>
    <template #status="{ row }">
      <el-switch
        :model-value="row.status === 'enabled'"
        inline-prompt
        active-text="启"
        inactive-text="停"
        size="small"
        @change="(val: boolean) => toggleProxy(row, val)"
      />
    </template>
  </OnTable>

  <ProxyFormDialog v-model:visible="proxyFormVisible" :site-id="siteId" :proxy="proxyFormTarget" @saved="fetchProxies" />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import OnTable from '@/components/OnTable/index.vue'
import type { TableColumn } from '@/components/OnTable/types'
import ProxyFormDialog from '../ProxyFormDialog.vue'
import { listProxies, updateProxy, deleteProxy } from '@/api/sites'
import type { Proxy } from '@/api/sites/type'
import { useMessage } from '@/hooks'

const { confirm } = useMessage()

const props = defineProps<{
  siteId: number
}>()

const proxyList = ref<Proxy[]>([])
const proxyLoading = ref(false)
const proxyFormVisible = ref(false)
const proxyFormTarget = ref<Proxy | null>(null)

const tableColumns: TableColumn[] = [
  { prop: 'name', label: '名称', width: 120 },
  { prop: 'proxy_dir', label: '代理目录', width: 120 },
  { prop: 'target_url', label: '目标URL', minWidth: 180, showOverflowTooltip: true },
  { prop: 'cache', label: '缓存', width: 100, slot: 'cache' },
  { prop: 'status', label: '状态', width: 100, slot: 'status' },
  {
    label: '操作',
    width: 180,
    fixed: 'right',
    buttons: [
      { name: { zh: '编辑', en: 'Edit' }, command: 'edit', size: 'small' },
      { name: { zh: '删除', en: 'Delete' }, command: 'delete', type: 'danger', size: 'small' },
    ],
  },
]

function handleCommand(command: string | number, row: Proxy) {
  if (command === 'edit') openProxyForm(row)
  else if (command === 'delete') delProxy(row)
}

async function fetchProxies() {
  proxyLoading.value = true
  try {
    proxyList.value = (await listProxies(props.siteId)) || []
  } catch {
    proxyList.value = []
  } finally {
    proxyLoading.value = false
  }
}

function openProxyForm(proxy?: Proxy) {
  proxyFormTarget.value = proxy || null
  proxyFormVisible.value = true
}

async function toggleProxy(proxy: Proxy, enable: boolean) {
  try {
    await updateProxy(proxy.id, { status: enable ? 'enabled' : 'disabled' })
    ElMessage.success(enable ? '已启用' : '已禁用')
    fetchProxies()
  } catch (e: any) {
    ElMessage.error(e.message || '操作失败')
  }
}

async function delProxy(proxy: Proxy) {
  const ok = await confirm({ message: `确定删除反向代理「${proxy.name}」？` })
  if (!ok) return
  try {
    await deleteProxy(proxy.id)
    ElMessage.success('删除成功')
    fetchProxies()
  } catch (e: any) {
    ElMessage.error(e.message || '删除失败')
  }
}
defineExpose({ fetchProxies })
</script>
