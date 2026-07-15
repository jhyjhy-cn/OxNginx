<template>
  <el-dialog
    v-model="visible"
    :title="$t('dbm.menu.dbAdd')"
    width="560px"
    :close-on-click-modal="false"
    @open="onOpen"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
      <el-form-item :label="$t('common.type')">
        <el-radio-group v-model="mode">
          <el-radio value="existing">{{ $t('dbm.addExisting') }}</el-radio>
          <el-radio value="new">{{ $t('dbm.createNew') }}</el-radio>
        </el-radio-group>
      </el-form-item>

      <el-form-item :label="$t('common.name')" prop="name">
        <el-input v-model="form.name" maxlength="64" />
      </el-form-item>

      <el-form-item :label="$t('dbm.colRootPath')" :prop="mode === 'existing' ? 'path' : ''">
        <el-input v-model="form.path" :placeholder="placeholder" clearable>
          <template #append>
            <el-button @click="onBrowse">{{ $t('dbm.browse') }}</el-button>
          </template>
        </el-input>
        <div class="hint">{{ $t('dbm.pathHint') }}</div>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="visible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="onSubmit">{{ $t('common.confirm') }}</el-button>
    </template>
  </el-dialog>

  <!-- 浏览文件:复用 /api/files/list 展示当前路径下子项,点击选择 -->
  <el-dialog v-model="browseVisible" :title="$t('dbm.browse')" width="640px">
    <FileBrowser v-model:path="browsePath" @pick="onPick" />
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { ElMessage, type FormInstance, type FormRules } from 'element-plus'
import { useI18n } from 'vue-i18n'
import { createDatabase } from '@/api/databases'
import FileBrowser from './FileBrowser.vue'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [v: boolean]; saved: [] }>()

const { t } = useI18n()
const visible = computed({ get: () => props.modelValue, set: (v) => emit('update:modelValue', v) })

interface FormState {
  name: string
  path: string
}
const form = reactive<FormState>({ name: '', path: '' })
const formRef = ref<FormInstance>()
const submitting = ref(false)
const mode = ref<'existing' | 'new'>('existing')

const placeholder = computed(() => t('dbm.pathPlaceholder'))

const rules = computed<FormRules<FormState>>(() => ({
  name: [{ required: true, message: t('dbm.required'), trigger: 'blur' }],
  path: mode.value === 'existing' ? [{ required: true, message: t('dbm.required'), trigger: 'blur' }] : [],
}))

function onOpen() {
  form.name = ''
  form.path = ''
  mode.value = 'existing'
}

const browseVisible = ref(false)
const browsePath = ref('')

function onBrowse() {
  browsePath.value = form.path || ''
  browseVisible.value = true
}

function onPick(file: string) {
  form.path = file
  browseVisible.value = false
}

async function onSubmit() {
  if (!formRef.value) return
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    // 新建模式:路径空时让后端默认 server/sqlite/{name}.db
    const payload: Record<string, unknown> = {
      type: 'sqlite',
      name: form.name,
      enabled: true,
    }
    if (mode.value === 'existing') {
      payload.db_name = form.path // 现有文件路径
    } else {
      // 新建:空路径由后端默认,非空就用
      if (form.path.trim()) payload.db_name = form.path.trim()
      else payload.db_name = null
    }
    await createDatabase(payload as any)
    ElMessage.success(t('common.success'))
    visible.value = false
    emit('saved')
  } catch (e: any) {
    ElMessage.error(e?.message || t('common.failed'))
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-top: 4px;
}
</style>
