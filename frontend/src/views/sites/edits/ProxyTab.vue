<template>
  <div style="display: flex; gap: 8px; margin-bottom: 12px">
    <el-button type="primary" size="small" @click="openProxyForm()">添加</el-button>
    <el-button size="small" @click="fetchProxies">刷新</el-button>
  </div>
  <el-table :data="proxyList" style="width: 100%" v-loading="proxyLoading">
    <el-table-column prop="name" label="名称" width="120" />
    <el-table-column prop="proxy_dir" label="代理目录" width="120" />
    <el-table-column prop="target_url" label="目标URL" min-width="180" show-overflow-tooltip />
    <el-table-column label="缓存" width="100">
      <template #default="{ row }">
        <el-tag :type="row.cache === 1 ? 'success' : 'info'" size="small">
          {{ row.cache === 1 ? '已开启' : '已关闭' }}
        </el-tag>
      </template>
    </el-table-column>
    <el-table-column label="状态" width="100">
      <template #default="{ row }">
        <el-switch
          :model-value="row.status === 'enabled'"
          inline-prompt
          active-text="启"
          inactive-text="停"
          size="small"
          @change="(val: boolean) => toggleProxy(row, val)"
        />
      </template>
    </el-table-column>
    <el-table-column label="操作" width="180" fixed="right">
      <template #default="{ row }">
        <el-button type="primary" link size="small" @click="openProxyForm(row)">编辑</el-button>
        <el-button type="danger" link size="small" @click="deleteProxy(row)">删除</el-button>
      </template>
    </el-table-column>
  </el-table>

  <ProxyFormDialog v-model:visible="proxyFormVisible" :site-id="siteId" :proxy="proxyFormTarget" @saved="fetchProxies" />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'
import ProxyFormDialog from '../ProxyFormDialog.vue'
import type { ReverseProxy } from '../types'

const props = defineProps<{
  siteId: number
}>()

const proxyList = ref<ReverseProxy[]>([])
const proxyLoading = ref(false)
const proxyFormVisible = ref(false)
const proxyFormTarget = ref<ReverseProxy | null>(null)

async function fetchProxies() {
  proxyLoading.value = true
  try {
    const res = await api.get(`/api/sites/${props.siteId}/proxies`)
    if (res.data.code === 0) {
      proxyList.value = res.data.data || []
    }
  } catch {
    proxyList.value = []
  } finally {
    proxyLoading.value = false
  }
}

function openProxyForm(proxy?: ReverseProxy) {
  proxyFormTarget.value = proxy || null
  proxyFormVisible.value = true
}

async function toggleProxy(proxy: ReverseProxy, enable: boolean) {
  try {
    await api.put(`/api/proxies/${proxy.id}`, { status: enable ? 'enabled' : 'disabled' })
    ElMessage.success(enable ? '已启用' : '已禁用')
    fetchProxies()
  } catch (e: any) {
    ElMessage.error(e.response?.data?.message || '操作失败')
  }
}

async function deleteProxy(proxy: ReverseProxy) {
  try {
    await ElMessageBox.confirm(`确定删除反向代理「${proxy.name}」？`, '提示', { type: 'warning' })
    await api.delete(`/api/proxies/${proxy.id}`)
    ElMessage.success('删除成功')
    fetchProxies()
  } catch (e: any) {
    if (e !== 'cancel') ElMessage.error(e.response?.data?.message || '删除失败')
  }
}

// tab 切换到 proxy 时由父组件触发
defineExpose({ fetchProxies })
</script>
