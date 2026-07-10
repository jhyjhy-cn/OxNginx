<template>
  <OnDialog v-model="dialogVisible" :title="$t('sys.sites.addSite')" width="600px">
    <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
      <el-form-item :label="$t('sys.sites.name')" prop="name">
        <el-input v-model="form.name" :placeholder="$t('sys.sites.enterSiteName')" />
      </el-form-item>
      <el-form-item :label="$t('sys.sites.domain')" prop="server_name">
        <el-input
          v-model="form.server_name"
          type="textarea"
          :autosize="{ minRows: 6, maxRows: 8 }"
          :placeholder="domainPlaceholder"
          @input="onDomainsInput"
        />
      </el-form-item>
      <el-form-item :label="$t('sys.sites.enableSsl')">
        <el-switch v-model="form.ssl" />
      </el-form-item>
      <template v-if="form.ssl">
        <el-form-item :label="$t('sys.sites.certPath')">
          <el-input v-model="form.certificate_path" placeholder="/opt/oxnginx/ssl/fullchain.cer" />
        </el-form-item>
        <el-form-item :label="$t('sys.sites.keyPath')">
          <el-input v-model="form.key_path" placeholder="/opt/oxnginx/ssl/private.key" />
        </el-form-item>
      </template>
      <el-form-item :label="$t('sys.sites.proxyPass')">
        <el-input v-model="form.proxy_pass" placeholder="http://127.0.0.1:8080" />
      </el-form-item>
      <el-form-item :label="$t('sys.sites.rootPath')">
        <el-input v-model="form.root_path" :placeholder="$t('sys.sites.rootPathHint')" />
      </el-form-item>
      <el-form-item :label="$t('sys.sites.remark')">
        <el-input v-model="form.remark" type="textarea" :autosize="{ minRows: 2, maxRows: 4 }" :placeholder="$t('sys.sites.remarkHint')" />
      </el-form-item>
      <el-form-item :label="$t('sys.sites.expireTime')">
        <el-date-picker
          v-model="form.expire_time"
          type="datetime"
          :placeholder="$t('sys.sites.permanent')"
          format="YYYY-MM-DD HH:mm:ss"
          value-format="YYYY-MM-DD HH:mm:ss"
          clearable
          style="width: 100%"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="submit">{{ $t('common.confirm') }}</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import type { FormInstance } from 'element-plus'
import OnDialog from '@/components/OnDialog/index.vue'
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

const formRef = ref<FormInstance>()
const submitting = ref(false)
const form = reactive({
  name: '',
  server_name: '',
  ssl: false,
  certificate_path: '',
  key_path: '',
  proxy_pass: '',
  root_path: '',
  remark: '',
  expire_time: '',
})

const rules = {
  name: [{ required: true, message: t('sys.sites.enterSiteName'), trigger: 'blur' }],
  server_name: [{ required: true, message: t('sys.sites.enterDomain'), trigger: 'blur' }],
}

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

// 打开时重置表单
watch(
  () => props.visible,
  (val) => {
    if (val) {
      form.name = ''
      form.server_name = ''
      form.ssl = false
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
      ssl: form.ssl,
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
