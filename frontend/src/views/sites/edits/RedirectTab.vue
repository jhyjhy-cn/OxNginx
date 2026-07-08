<template>
  <div style="display: flex; gap: 8px; margin-bottom: 12px">
    <el-button type="primary" size="small" @click="openRedirectForm()">添加重定向</el-button>
    <el-button size="small" @click="add404Redirect">404重定向</el-button>
    <el-button size="small" @click="fetchRedirects">刷新</el-button>
    <el-button v-if="redirectSelected.length > 0" type="danger" size="small" @click="batchDeleteRedirects">
      删除 ({{ redirectSelected.length }})
    </el-button>
  </div>
  <el-table :data="redirectRules" style="width: 100%" @selection-change="(val: RedirectRule[]) => (redirectSelected = val)">
    <el-table-column type="selection" width="45" />
    <el-table-column label="被重定向" min-width="150" show-overflow-tooltip>
      <template #default="{ row }">
        {{ row.domains.join(', ') }}
      </template>
    </el-table-column>
    <el-table-column label="重定向类型" width="100">
      <template #default="{ row }">
        <el-tag size="small">{{ row.redirect_type === 'type' ? '域名类型' : '路径类型' }}</el-tag>
      </template>
    </el-table-column>
    <el-table-column label="重定向到" min-width="180" show-overflow-tooltip>
      <template #default="{ row }">
        {{ row.target_url }}
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
          @change="(val: boolean) => toggleRedirect(row, val)"
        />
      </template>
    </el-table-column>
    <el-table-column label="操作" width="180" fixed="right">
      <template #default="{ row, $index }">
        <el-button type="primary" link size="small" @click="openRedirectConfigFile(row)">配置文件</el-button>
        <el-button type="primary" link size="small" @click="openRedirectForm(row, $index)">编辑</el-button>
        <el-button type="danger" link size="small" @click="deleteRedirect($index)">删除</el-button>
      </template>
    </el-table-column>
  </el-table>

  <RedirectFormDialog v-model:visible="redirectFormVisible" :domains="domains" :rule="redirectFormTarget" @saved="onRedirectSaved" />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTabStore } from '@/stores/tabs'
import { useFilesStore } from '@/stores/files'
import RedirectFormDialog from '../RedirectFormDialog.vue'
import type { RedirectRule } from '../types'

const router = useRouter()

const props = defineProps<{
  siteId: number
  siteName: string
  domains: string[]
  /** editForm.redirect_rules 的原始 JSON 字符串 */
  redirectRulesJson: string
}>()

const emit = defineEmits<{
  /** 更新 editForm.redirect_rules */
  'update:redirectRulesJson': [value: string]
  saved: []
}>()

const redirectRules = ref<RedirectRule[]>([])
const redirectSelected = ref<RedirectRule[]>([])
const redirectFormVisible = ref(false)
const redirectFormTarget = ref<RedirectRule | null>(null)
const redirectEditIndex = ref(-1)

/** 从 JSON 字符串解析重定向规则（兼容旧格式） */
function fetchRedirects() {
  try {
    const raw = JSON.parse(props.redirectRulesJson || '[]')
    redirectRules.value = raw.map((r: any) => {
      if ('domain' in r && 'target' in r) {
        // 旧格式转换
        return {
          enabled: true,
          keep_params: false,
          redirect_type: 'path',
          redirect_method: r.redirect_type || 301,
          domains: [r.domain],
          target_url: r.target,
          status: 'enabled',
        } as RedirectRule
      }
      return r as RedirectRule
    })
  } catch {
    redirectRules.value = []
  }
}

function emitAndSave() {
  emit('update:redirectRulesJson', JSON.stringify(redirectRules.value))
  emit('saved')
}

function openRedirectForm(rule?: RedirectRule, index?: number) {
  redirectFormTarget.value = rule || null
  redirectEditIndex.value = index ?? -1
  redirectFormVisible.value = true
}

function onRedirectSaved(rule: RedirectRule) {
  if (redirectEditIndex.value >= 0) {
    redirectRules.value.splice(redirectEditIndex.value, 1, rule)
  } else {
    redirectRules.value.push(rule)
  }
  emitAndSave()
}

function toggleRedirect(rule: RedirectRule, enable: boolean) {
  rule.status = enable ? 'enabled' : 'disabled'
  rule.enabled = enable
  emitAndSave()
}

function deleteRedirect(index: number) {
  redirectRules.value.splice(index, 1)
  emitAndSave()
}

function batchDeleteRedirects() {
  const toDelete = new Set(redirectSelected.value)
  redirectRules.value = redirectRules.value.filter((r) => !toDelete.has(r))
  redirectSelected.value = []
  emitAndSave()
}

function add404Redirect() {
  openRedirectForm({
    enabled: true,
    keep_params: false,
    redirect_type: 'path',
    redirect_method: 302,
    domains: ['404'],
    target_url: '',
    status: 'enabled',
  })
}

function openRedirectConfigFile(_rule: RedirectRule) {
  if (props.siteName) {
    const path = `/opt/oxnginx/nginx/conf/sites-enabled/${props.siteName}.conf`
    const tabStore = useTabStore()
    const filesStore = useFilesStore()
    tabStore.addTab({ path: '/files', title: 'menu.files', closable: true })
    const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '')
    const existing = filesStore.tabs.find((t) => t.path.replace(/\\/g, '/').replace(/\/+$/, '') === normalized)
    if (existing) {
      filesStore.setActiveTab(existing.id)
    } else {
      filesStore.addTab(path)
    }
    router.push('/files')
  }
}

defineExpose({ fetchRedirects })
</script>
