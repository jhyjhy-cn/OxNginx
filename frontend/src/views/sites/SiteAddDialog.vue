<template>
  <OnDialog v-model="dialogVisible" :title="$t('sys.sites.addSite')" width="600px">
    <OnForm ref="formRef" :model="form">
      <OnFormGrid :model="form" :fields="fieldsName" />

      <!-- ponytail: 域名输入需要 autosize + @input 自动提取站点名,OnFormItem 暂不支持,保留原生行 -->
      <el-form-item :label="$t('sys.sites.domain')" prop="server_name" :rules="serverNameRules">
        <el-input
          v-model="form.server_name"
          type="textarea"
          :autosize="{ minRows: 6, maxRows: 8 }"
          :placeholder="domainPlaceholder"
          @input="onDomainsInput"
        />
      </el-form-item>

      <OnFormGrid :model="form" :fields="fieldsSsl" />

      <!-- ponytail: 根目录需要 append 浏览按钮,OnFormGrid 暂无字段级插槽,保留原生行 -->
      <el-form-item :label="$t('sys.sites.rootPath')">
        <el-input v-model="form.root_path" :placeholder="$t('sys.sites.rootPathHint')" clearable>
          <template #append>
            <el-button @click="pickerVisible = true">{{ $t('common.browse') }}</el-button>
          </template>
        </el-input>
      </el-form-item>

      <OnFormGrid :model="form" :fields="fieldsTail" />
    </OnForm>
    <template #footer>
      <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="submit">{{ $t('common.confirm') }}</el-button>
    </template>
  </OnDialog>

  <OnFilePicker v-model="pickerVisible" mode="folder" :initial-path="form.root_path" @pick="onPick" />
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import OnDialog from '@/components/OnDialog/index.vue'
import OnForm from '@/components/OnForm/OnForm/index.vue'
import OnFormGrid from '@/components/OnForm/OnFormGrid/index.vue'
import OnFilePicker from '@/components/OnFilePicker/index.vue'
import type { FormField } from '@/components/OnForm/types'
import { createSite } from '@/api/sites'

const { t } = useI18n()

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  created: []
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

const formRef = ref<InstanceType<typeof OnForm>>()
const submitting = ref(false)
const form = reactive({
  name: '',
  server_name: '',
  ssl: 0, // OnFormItem 的 switch 固定 active-value=1/inactive-value=0
  certificate_path: '',
  key_path: '',
  proxy_pass: '',
  root_path: '',
  remark: '',
  expire_time: '',
})

const serverNameRules = [{ required: true, message: t('sys.sites.enterDomain'), trigger: 'blur' }]

// ---- 表单字段（统一 OnFormGrid 声明式渲染）----
const fieldsName = computed<FormField[]>(() => [
  { prop: 'name', label: 'sys.sites.name', type: 'input', required: true, placeholder: 'sys.sites.enterSiteName' },
])

const fieldsSsl = computed<FormField[]>(() => [
  { prop: 'ssl', label: 'sys.sites.enableSsl', type: 'switch' },
  {
    prop: 'certificate_path',
    label: 'sys.sites.certPath',
    type: 'input',
    visible: form.ssl === 1,
    placeholder: 'sys.sites.certPathPlaceholder',
  },
  {
    prop: 'key_path',
    label: 'sys.sites.keyPath',
    type: 'input',
    visible: form.ssl === 1,
    placeholder: 'sys.sites.keyPathPlaceholder',
  },
  { prop: 'proxy_pass', label: 'sys.sites.proxyPass', type: 'input', placeholder: 'sys.sites.proxyPassPlaceholder' },
])

const fieldsTail = computed<FormField[]>(() => [
  { prop: 'remark', label: 'sys.sites.remark', type: 'textarea', rows: 2, placeholder: 'sys.sites.remarkHint' },
  { prop: 'expire_time', label: 'sys.sites.expireTime', type: 'datetime', placeholder: 'sys.sites.permanent' },
])

const domainPlaceholder = computed(
  () => `${t('sys.sites.domainHint')}\n${t('sys.sites.domainFormatIp')}\n${t('sys.sites.domainFormatPort')}\n${t('sys.sites.domainFormatIpv6')}`
)

function extractPort(domains: string): string {
  const first = domains.split('\n')[0]?.trim() || ''
  const ipv6Match = first.match(/^\[.+?\]:(\d+)$/)
  if (ipv6Match) return ipv6Match[1]
  const portMatch = first.match(/:(\d+)$/)
  if (portMatch) return portMatch[1]
  return '80'
}

function onDomainsInput() {
  if (!form.name) {
    const firstLine = form.server_name.split('\n')[0]?.trim()
    if (firstLine) {
      form.name = firstLine.replace(/:\d+\]?$|:\d+$/, '').replace(/^\[|]$/g, '')
    }
  }
}

// ---- 根目录选择（复用统一 OnFilePicker）----
const pickerVisible = ref(false)

function onPick(paths: string[]) {
  form.root_path = paths[0] || ''
}

// 打开时重置表单
watch(
  () => props.visible,
  (val) => {
    if (val) {
      form.name = ''
      form.server_name = ''
      form.ssl = 0
      form.certificate_path = ''
      form.key_path = ''
      form.proxy_pass = ''
      form.root_path = ''
      form.remark = ''
      form.expire_time = ''
    }
  }
)

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const data = {
      name: form.name,
      server_name: form.server_name.replace(/\n/g, ' ').trim(),
      listen: extractPort(form.server_name),
      ssl: form.ssl === 1,
      certificate_path: form.certificate_path || null,
      key_path: form.key_path || null,
      proxy_pass: form.proxy_pass || null,
      root_path: form.root_path || null,
      remark: form.remark || null,
      expire_time: form.expire_time || null,
    }
    await createSite(data)
    ElMessage.success(t('sys.sites.createSuccess'))
    dialogVisible.value = false
    emit('created')
  } catch (error: any) {
    ElMessage.error(error.message || t('sys.sites.operationFailed'))
  } finally {
    submitting.value = false
  }
}
</script>
