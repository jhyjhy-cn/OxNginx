<template>
  <OnDialog v-model="dialogVisible" :title="`${$t('sites.editSite')}[${siteName}]`" width="60%" height="70%">
    <el-tabs v-model="activeTab" tab-position="left" class="edit-tabs">
      <!-- 1. 域名管理 -->
      <el-tab-pane :label="$t('sites.tabDomain')" name="domain">
        <el-form label-width="80px">
          <el-form-item :label="$t('sites.domain')">
            <div style="display: flex; gap: 8px; width: 100%">
              <el-input
                v-model="domainInput"
                type="textarea"
                :autosize="{ minRows: 2, maxRows: 6 }"
                :placeholder="$t('sites.domainInputHint')"
                style="flex: 1"
              />
              <el-button type="primary" style="align-self: flex-end" @click="addDomains">
                {{ $t('sites.addDomain') }}
              </el-button>
            </div>
          </el-form-item>
        </el-form>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px">
          <el-button v-if="domainSelected.length > 0" type="danger" size="small" @click="deleteSelectedDomains">
            {{ $t('common.delete') }} ({{ domainSelected.length }})
          </el-button>
          <span v-else />
          <span style="font-size: 12px; color: #909399">{{ $t('sites.domainCount', { n: domains.length }) }}</span>
        </div>
        <el-table :data="domainsDisplay" style="width: 100%" max-height="380" @selection-change="(val: DomainItem[]) => domainSelected = val">
          <el-table-column type="selection" width="45" />
          <el-table-column :label="$t('sites.domain')">
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
      </el-tab-pane>

      <!-- 2. 伪静态 -->
      <el-tab-pane :label="$t('sites.tabRewrite')" name="rewrite">
        <div v-for="(rule, i) in rewriteRules" :key="i" class="rule-row">
          <el-input v-model="rule.pattern" placeholder="^/old/(.*)$" style="flex: 2" @change="debouncedSave" />
          <el-input v-model="rule.replacement" placeholder="/new/$1" style="flex: 2" @change="debouncedSave" />
          <el-select v-model="rule.flag" style="width: 110px" @change="debouncedSave">
            <el-option label="last" value="last" />
            <el-option label="break" value="break" />
            <el-option label="redirect" value="redirect" />
            <el-option label="permanent" value="permanent" />
          </el-select>
          <el-button type="danger" :icon="Delete" circle size="small" @click="rewriteRules.splice(i, 1); debouncedSave()" />
        </div>
        <el-button type="primary" plain size="small" @click="rewriteRules.push({ pattern: '', replacement: '', flag: 'last' })">
          <el-icon><Plus /></el-icon> {{ $t('sites.addRule') }}
        </el-button>
      </el-tab-pane>

      <!-- 3. 配置文件 -->
      <el-tab-pane :label="$t('sites.tabConfig')" name="config">
        <div class="config-tab-content">
          <div class="config-hint">
            <p>提示：Ctrl+F 搜索关键字，Ctrl+S 保存，Ctrl+H 查找替换</p>
            <p>此处为站点主配置文件，若您不了解配置规则，请勿随意修改</p>
          </div>
          <div ref="configEditorRef" class="config-editor-box" />
          <div style="display: flex; gap: 8px; margin-top: 8px">
            <el-button type="primary" size="small" :loading="configSaving" @click="saveSiteConfig">{{ $t('common.save') }}</el-button>
            <el-button size="small" @click="loadSiteConfig">{{ $t('common.refresh') }}</el-button>
          </div>
        </div>
      </el-tab-pane>

      <!-- 4. SSL证书 -->
      <el-tab-pane :label="$t('sites.tabSsl')" name="ssl">
        <el-form label-width="80px">
          <el-form-item :label="$t('sites.enableSsl')">
            <el-switch v-model="editForm.ssl" @change="debouncedSave" />
          </el-form-item>
          <template v-if="editForm.ssl">
            <el-form-item :label="$t('sites.certPath')">
              <el-input v-model="editForm.certificate_path" placeholder="/opt/oxnginx/ssl/fullchain.cer" @change="debouncedSave" />
            </el-form-item>
            <el-form-item :label="$t('sites.keyPath')">
              <el-input v-model="editForm.key_path" placeholder="/opt/oxnginx/ssl/private.key" @change="debouncedSave" />
            </el-form-item>
          </template>
        </el-form>
      </el-tab-pane>

      <!-- 5. 反向代理 -->
      <el-tab-pane :label="$t('sites.tabProxy')" name="proxy">
        <el-form label-width="80px">
          <el-form-item :label="$t('sites.proxyPass')">
            <el-input v-model="editForm.proxy_pass" placeholder="http://127.0.0.1:8080" @change="debouncedSave" />
          </el-form-item>
          <el-form-item :label="$t('sites.rootPath')">
            <el-input v-model="editForm.root_path" placeholder="/opt/oxnginx/wwwroot" @change="debouncedSave" />
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 6. 重定向 -->
      <el-tab-pane :label="$t('sites.tabRedirect')" name="redirect">
        <el-alert :title="$t('sites.redirectHint')" type="info" :closable="false" style="margin-bottom: 12px" />
        <div v-for="(rule, i) in redirectRules" :key="i" class="rule-row">
          <el-input v-model="rule.domain" :placeholder="$t('sites.redirectDomain')" style="flex: 2" @change="debouncedSave" />
          <el-input v-model="rule.target" :placeholder="$t('sites.redirectTarget')" style="flex: 2" @change="debouncedSave" />
          <el-select v-model="rule.redirect_type" style="width: 90px" @change="debouncedSave">
            <el-option label="301" :value="301" />
            <el-option label="302" :value="302" />
          </el-select>
          <el-button type="danger" :icon="Delete" circle size="small" @click="redirectRules.splice(i, 1); debouncedSave()" />
        </div>
        <el-button type="primary" plain size="small" @click="redirectRules.push({ domain: '', target: '', redirect_type: 301 })">
          <el-icon><Plus /></el-icon> {{ $t('sites.addRule') }}
        </el-button>
      </el-tab-pane>

      <!-- 7. 防盗链 -->
      <el-tab-pane :label="$t('sites.tabHotlink')" name="hotlink">
        <el-form label-width="100px">
          <el-form-item :label="$t('sites.hotlinkEnable')">
            <el-switch v-model="hotlink.enabled" @change="debouncedSave" />
          </el-form-item>
          <template v-if="hotlink.enabled">
            <el-form-item :label="$t('sites.hotlinkDomains')">
              <el-input v-model="hotlink.domainsStr" type="textarea" :autosize="{ minRows: 3, maxRows: 6 }" :placeholder="$t('sites.hotlinkDomainsHint')" @change="debouncedSave" />
            </el-form-item>
            <el-form-item :label="$t('sites.hotlinkCode')">
              <el-select v-model="hotlink.return_code" @change="debouncedSave">
                <el-option label="403 Forbidden" :value="403" />
                <el-option label="404 Not Found" :value="404" />
              </el-select>
            </el-form-item>
          </template>
        </el-form>
      </el-tab-pane>

      <!-- 8. 网站日志 -->
      <el-tab-pane :label="$t('sites.tabLog')" name="log">
        <el-form label-width="100px">
          <el-form-item :label="$t('sites.logAccessPath')">
            <el-input v-model="editForm.log_access_path" :placeholder="$t('sites.logAccessPathHint')" @change="debouncedSave" />
          </el-form-item>
          <el-form-item :label="$t('sites.logErrorPath')">
            <el-input v-model="editForm.log_error_path" :placeholder="$t('sites.logErrorPathHint')" @change="debouncedSave" />
          </el-form-item>
        </el-form>
        <el-divider />
        <div style="display: flex; gap: 8px; margin-bottom: 8px">
          <el-button size="small" :loading="logLoading" @click="loadSiteLog('access')">{{ $t('sites.accessLog') }}</el-button>
          <el-button size="small" :loading="logLoading" @click="loadSiteLog('error')">{{ $t('sites.errorLog') }}</el-button>
        </div>
        <pre v-if="siteLog" class="log-output">{{ siteLog }}</pre>
        <el-empty v-else :description="$t('sites.clickToLoadLog')" />
      </el-tab-pane>
    </el-tabs>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Delete } from '@element-plus/icons-vue'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'
import { monaco } from '@/utils/monaco-env'
import type { DomainItem, RewriteRule, RedirectRule, HotlinkCfg } from './types'

const { t } = useI18n()

const props = defineProps<{
  visible: boolean
  siteId: number | null
  siteName: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: []
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

// ---- 通用编辑表单 ----
const activeTab = ref('domain')
const editForm = reactive({
  name: '',
  server_name: '',
  ssl: false,
  certificate_path: '',
  key_path: '',
  proxy_pass: '',
  root_path: '',
  remark: '',
  expire_time: '',
  rewrite_rules: '',
  redirect_rules: '',
  hotlink_config: '',
  log_access_path: '',
  log_error_path: '',
})

// ---- 域名管理 ----
const domainInput = ref('')
const domains = ref<string[]>([])
const domainSelected = ref<DomainItem[]>([])
const domainsDisplay = computed(() => domains.value.map((d) => ({ domain: d })))

function openDomain(domain: string) {
  window.open('http://' + domain, '_blank')
}

function addDomains() {
  const lines = domainInput.value.split('\n').map((l) => l.trim()).filter(Boolean)
  let added = false
  for (const d of lines) {
    if (!domains.value.includes(d)) {
      domains.value.push(d)
      added = true
    }
  }
  domainInput.value = ''
  if (added) saveDomains()
}

function deleteDomain(domain: string) {
  domains.value = domains.value.filter((d) => d !== domain)
  saveDomains()
}

function deleteSelectedDomains() {
  const toDelete = new Set(domainSelected.value.map((d) => d.domain))
  domains.value = domains.value.filter((d) => !toDelete.has(d))
  domainSelected.value = []
  saveDomains()
}

function extractPort(domains: string): string {
  const first = domains.split('\n')[0]?.trim() || ''
  const ipv6Match = first.match(/^\[.+?\]:(\d+)$/)
  if (ipv6Match) return ipv6Match[1]
  const portMatch = first.match(/:(\d+)$/)
  if (portMatch) return portMatch[1]
  return '80'
}

async function saveDomains() {
  if (!props.siteId) return
  try {
    const server_name = domains.value.join(' ')
    const listen = extractPort(domains.value[0] || '80')
    await api.put(`/api/sites/${props.siteId}`, { server_name, listen })
    ElMessage.success(t('common.success'))
    emit('saved')
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('sites.operationFailed'))
  }
}

// ---- 伪静态 / 重定向 / 防盗链 ----
const rewriteRules = ref<RewriteRule[]>([])
const redirectRules = ref<RedirectRule[]>([])
const hotlink = reactive<HotlinkCfg>({ enabled: false, domainsStr: '', return_code: 403 })

// ---- 配置文件编辑器 ----
const configEditorRef = ref<HTMLElement>()
const configSaving = ref(false)
let configEditor: monaco.editor.IStandaloneCodeEditor | null = null

// ---- 日志 ----
const logLoading = ref(false)
const siteLog = ref('')

// ---- 初始化：打开弹窗时加载数据 ----
watch(() => props.visible, async (val) => {
  if (!val || !props.siteId) return
  activeTab.value = 'domain'
  siteLog.value = ''

  try {
    const res = await api.get(`/api/sites/${props.siteId}`)
    if (res.data.code !== 0) return
    const site = res.data.data
    Object.assign(editForm, {
      name: site.name,
      server_name: site.server_name,
      ssl: !!site.ssl,
      certificate_path: site.certificate_path || '',
      key_path: site.key_path || '',
      proxy_pass: site.proxy_pass || '',
      root_path: site.root_path || '',
      remark: site.remark || '',
      expire_time: site.expire_time || '',
      rewrite_rules: site.rewrite_rules || '',
      redirect_rules: site.redirect_rules || '',
      hotlink_config: site.hotlink_config || '',
      log_access_path: site.log_access_path || '',
      log_error_path: site.log_error_path || '',
    })
    domains.value = site.server_name.split(' ').map((d: string) => d.trim()).filter(Boolean)
    domainInput.value = ''
    domainSelected.value = []

    try { rewriteRules.value = JSON.parse(editForm.rewrite_rules || '[]') } catch { rewriteRules.value = [] }
    try { redirectRules.value = JSON.parse(editForm.redirect_rules || '[]') } catch { redirectRules.value = [] }
    try {
      const hc = JSON.parse(editForm.hotlink_config || '{}')
      hotlink.enabled = hc.enabled || false
      hotlink.domainsStr = (hc.allowed_domains || []).join('\n')
      hotlink.return_code = hc.return_code || 403
    } catch {
      hotlink.enabled = false
      hotlink.domainsStr = ''
      hotlink.return_code = 403
    }
  } catch { /* ignore */ }
})

// ---- 配置文件编辑器初始化 ----
watch(activeTab, (tab) => {
  if (tab === 'config' && configEditorRef.value && props.siteId) {
    nextTick(() => {
      if (!configEditor) {
        configEditor = monaco.editor.create(configEditorRef.value!, {
          value: '',
          language: 'nginx',
          theme: 'vs-dark',
          minimap: { enabled: false },
          fontSize: 13,
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
          automaticLayout: true,
          tabSize: 4,
        })
        configEditor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
          saveSiteConfig()
        })
      }
      loadSiteConfig()
    })
  }
})

async function loadSiteConfig() {
  if (!props.siteId) return
  try {
    const res = await api.get(`/api/config/file/${props.siteName}`)
    if (res.data.code === 0 && configEditor) {
      configEditor.setValue(res.data.data?.content || '')
    }
  } catch { /* ignore */ }
}

async function saveSiteConfig() {
  if (!props.siteId || !configEditor) return
  configSaving.value = true
  try {
    const res = await api.put(`/api/config/file/${props.siteName}`, { content: configEditor.getValue() })
    if (res.data.code === 0) {
      ElMessage.success(t('common.success'))
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('common.failed'))
  } finally {
    configSaving.value = false
  }
}

// ---- 日志 ----
async function loadSiteLog(type: 'access' | 'error') {
  logLoading.value = true
  try {
    const res = await api.get(`/api/log/${type}`)
    if (res.data.code === 0) {
      siteLog.value = (res.data.data?.lines || []).join('\n')
    }
  } catch {
    siteLog.value = t('sites.logLoadFailed')
  } finally {
    logLoading.value = false
  }
}

// ---- 防抖保存 ----
let saveTimer: ReturnType<typeof setTimeout> | null = null
function debouncedSave() {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => saveAllSettings(), 500)
}

async function saveAllSettings() {
  if (!props.siteId) return
  try {
    const data = {
      ssl: editForm.ssl,
      certificate_path: editForm.certificate_path || null,
      key_path: editForm.key_path || null,
      proxy_pass: editForm.proxy_pass || null,
      root_path: editForm.root_path || null,
      rewrite_rules: JSON.stringify(rewriteRules.value),
      redirect_rules: JSON.stringify(redirectRules.value),
      hotlink_config: JSON.stringify({
        enabled: hotlink.enabled,
        allowed_domains: hotlink.domainsStr.split('\n').map((d) => d.trim()).filter(Boolean),
        return_code: hotlink.return_code,
      }),
      log_access_path: editForm.log_access_path || null,
      log_error_path: editForm.log_error_path || null,
    }
    await api.put(`/api/sites/${props.siteId}`, data)
    ElMessage.success(t('common.success'))
    emit('saved')
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('sites.operationFailed'))
  }
}
</script>

<style scoped>
.edit-tabs {
  height: 100%;
  display: flex;
}
.edit-tabs :deep(.el-tabs__header) {
  min-width: 120px;
}
.edit-tabs :deep(.el-tabs__content) {
  padding: 0 16px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
.edit-tabs :deep(.el-tab-pane) {
  height: 100%;
}
.rule-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.config-tab-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.config-editor-box {
  width: 100%;
  flex: 1;
  min-height: 300px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 4px;
}
.config-hint {
  margin-bottom: 8px;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.8;
}
.log-output {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
  font-size: 12px;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  max-height: 380px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}
</style>
