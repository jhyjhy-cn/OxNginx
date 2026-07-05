<template>
  <OnDialog v-model="dialogVisible" :title="$t('sites.deleteSite')" width="420px" :maximizable="false">
    <div style="margin-bottom: 16px">
      <p>{{ $t('sites.confirmDeleteSite') }} <strong>{{ site?.name }}</strong> ?</p>
    </div>
    <el-checkbox v-model="options.deleteRecord">{{ $t('sites.deleteSiteRecord') }}</el-checkbox>
    <el-checkbox v-model="options.deleteFiles" style="margin-top: 12px">
      {{ $t('sites.deleteSiteFiles', { path: site?.root_path || $t('common.none') }) }}
    </el-checkbox>
    <template #footer>
      <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="danger" @click="confirm">{{ $t('sites.confirmDelete') }}</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { reactive, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'
import type { Site } from './types'

const { t } = useI18n()

const props = defineProps<{
  visible: boolean
  site: Site | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  deleted: []
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

const options = reactive({
  deleteRecord: true,
  deleteFiles: false,
})

watch(() => props.visible, (val) => {
  if (val) {
    options.deleteRecord = true
    options.deleteFiles = false
  }
})

async function confirm() {
  if (!props.site) return
  try {
    await api.delete(`/api/sites/${props.site.id}`, {
      data: {
        delete_record: options.deleteRecord,
        delete_files: options.deleteFiles,
      },
    })
    ElMessage.success(t('sites.deleteSuccess'))
    dialogVisible.value = false
    emit('deleted')
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('sites.deleteFailed'))
  }
}
</script>
