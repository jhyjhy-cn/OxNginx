<template>
  <OnDialog v-model="dialogVisible" :title="isEdit ? '编辑重定向' : '添加重定向'" width="520px" :maximizable="false">
    <el-form ref="formRef" :model="form" :rules="rules" label-width="100px">
      <el-form-item label="开启重定向">
        <el-switch v-model="form.enabled" active-text="启" inactive-text="停" />
      </el-form-item>
      <el-form-item label="保留URL参数">
        <el-switch v-model="form.keep_params" active-text="是" inactive-text="否" />
      </el-form-item>
      <el-form-item label="重定向类型" prop="redirect_type">
        <el-select v-model="form.redirect_type" style="width: 100%">
          <el-option label="域名类型" value="type" />
          <el-option label="路径类型" value="path" />
        </el-select>
      </el-form-item>
      <el-form-item label="重定向方式" prop="redirect_method">
        <el-select v-model="form.redirect_method" style="width: 100%">
          <el-option label="301 永久重定向" :value="301" />
          <el-option label="302 临时重定向" :value="302" />
        </el-select>
      </el-form-item>
      <el-form-item label="重定向域名" prop="domains">
        <el-select
          v-model="form.domains"
          multiple
          filterable
          allow-create
          default-first-option
          placeholder="选择或输入域名"
          style="width: 100%"
        >
          <el-option v-for="d in domainOptions" :key="d" :label="d" :value="d" />
        </el-select>
      </el-form-item>
      <el-form-item label="目标URL" prop="target_url">
        <el-input v-model="form.target_url" placeholder="https://example.com/new-path" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="submitting" @click="submit">确定</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import type { FormInstance } from 'element-plus'
import OnDialog from '@/components/OnDialog/index.vue'
import type { RedirectRule } from './types'

const props = defineProps<{
  visible: boolean
  domains: string[] // 站点已有域名列表，供选择
  rule?: RedirectRule | null // 编辑时传入
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  saved: [rule: RedirectRule]
}>()

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

const isEdit = computed(() => !!props.rule)
const formRef = ref<FormInstance>()
const submitting = ref(false)

const domainOptions = computed(() => props.domains || [])

const form = reactive({
  enabled: true,
  keep_params: false,
  redirect_type: 'type' as string,
  redirect_method: 301 as number,
  domains: [] as string[],
  target_url: '',
})

const rules = {
  redirect_type: [{ required: true, message: '请选择重定向类型', trigger: 'change' }],
  redirect_method: [{ required: true, message: '请选择重定向方式', trigger: 'change' }],
  domains: [{ required: true, type: 'array' as const, min: 1, message: '请至少选择一个域名', trigger: 'change' }],
  target_url: [{ required: true, message: '请输入目标URL', trigger: 'blur' }],
}

watch(
  () => props.visible,
  (val) => {
    if (val && props.rule) {
      form.enabled = props.rule.enabled
      form.keep_params = props.rule.keep_params
      form.redirect_type = props.rule.redirect_type
      form.redirect_method = props.rule.redirect_method
      form.domains = [...props.rule.domains]
      form.target_url = props.rule.target_url
    } else if (val) {
      form.enabled = true
      form.keep_params = false
      form.redirect_type = 'type'
      form.redirect_method = 301
      form.domains = []
      form.target_url = ''
    }
  }
)

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitting.value = true
  try {
    const rule: RedirectRule = {
      enabled: form.enabled,
      keep_params: form.keep_params,
      redirect_type: form.redirect_type,
      redirect_method: form.redirect_method,
      domains: [...form.domains],
      target_url: form.target_url,
      status: form.enabled ? 'enabled' : 'disabled',
    }
    emit('saved', rule)
    dialogVisible.value = false
  } finally {
    submitting.value = false
  }
}
</script>
