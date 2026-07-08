<template>
  <el-form label-width="100px">
    <el-form-item :label="$t('sites.logAccessPath')">
      <el-input v-model="editForm.log_access_path" :placeholder="$t('sites.logAccessPathHint')" @change="$emit('save')" />
    </el-form-item>
    <el-form-item :label="$t('sites.logErrorPath')">
      <el-input v-model="editForm.log_error_path" :placeholder="$t('sites.logErrorPathHint')" @change="$emit('save')" />
    </el-form-item>
  </el-form>
  <el-divider />
  <div style="display: flex; gap: 8px; margin-bottom: 8px">
    <el-button size="small" :loading="logLoading" @click="loadSiteLog('access')">{{ $t('sites.accessLog') }}</el-button>
    <el-button size="small" :loading="logLoading" @click="loadSiteLog('error')">{{ $t('sites.errorLog') }}</el-button>
  </div>
  <pre v-if="siteLog" class="log-output">{{ siteLog }}</pre>
  <el-empty v-else :description="$t('sites.clickToLoadLog')" />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import api from '@/api'

const { t } = useI18n()

// editForm 由父组件传入，直接修改（对象引用）
const props = defineProps<{
  editForm: {
    log_access_path: string
    log_error_path: string
  }
}>()

defineEmits<{
  save: []
}>()

const logLoading = ref(false)
const siteLog = ref('')

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
</script>

<style scoped>
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
