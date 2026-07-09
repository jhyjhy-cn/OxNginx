<template>
  <el-form
    ref="formRef"
    :model="props.model"
    :label-width="labelWidth"
    :label-position="props.labelPosition"
    :disabled="props.disabled"
    v-bind="$attrs"
  >
    <slot />
  </el-form>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { FormInstance } from 'element-plus'
import type { OnFormProps } from '../types'

defineOptions({ name: 'OnForm', inheritAttrs: false })

const props = withDefaults(defineProps<OnFormProps>(), {
  labelWidth: '100px',
  labelPosition: 'right',
  disabled: false,
})

const formRef = ref<FormInstance>()

const labelWidth = computed(() => {
  if (props.labelWidth === 'auto') return undefined
  return String(props.labelWidth)
})

async function validate() {
  return formRef.value?.validate()
}

function validateField(prop: string) {
  return formRef.value?.validateField(prop)
}

function resetFields() {
  formRef.value?.resetFields()
}

function clearValidate() {
  formRef.value?.clearValidate()
}

defineExpose({
  formRef,
  validate,
  validateField,
  resetFields,
  clearValidate,
})
</script>
