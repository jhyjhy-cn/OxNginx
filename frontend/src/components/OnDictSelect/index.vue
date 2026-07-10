<template>
  <el-select
    :model-value="normalized"
    :multiple="multiple"
    :clearable="clearable"
    :disabled="disabled"
    :placeholder="placeholder"
    filterable
    class="on-dict-select"
    @change="handleChange"
  >
    <el-option
      v-for="item in options"
      :key="item.value"
      :label="item.label"
      :value="item.value"
      :disabled="item.status === 'disabled'"
    />
  </el-select>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { listDicts } from '@/api/sys/dicts'
import type { Dict, DictItem } from '@/api/sys/dicts/type'

const props = withDefaults(
  defineProps<{
    /** 字典 code（对应 sys_dicts.code） */
    code: string
    vModel?: string | number | string[] | number[]
    /** 多选模式 */
    multiple?: boolean
    /** 可清空 */
    clearable?: boolean
    /** 禁用 */
    disabled?: boolean
    /** 占位符，支持 i18n key 或原文 */
    placeholder?: string
  }>(),
  {
    vModel: '',
    multiple: false,
    clearable: true,
    disabled: false,
    placeholder: 'common.input',
  }
)

const emit = defineEmits<{
  (e: 'update:vModel', v: string | number | string[] | number[]): void
}>()

// ponytail: 一次拉全表 + 前端按 code 过滤，避免每个 select 都打 /dicts/{id} 接口
const dictCache = new Map<string, DictItem[]>()
const allDicts = ref<Dict[]>([])

onMounted(async () => {
  if (dictCache.has(props.code)) return
  try {
    const list = await listDicts()
    for (const d of list || []) {
      if (d.code && d.items) dictCache.set(d.code, d.items)
    }
    allDicts.value = list || []
  } catch {}
})

const options = computed(() => {
  const items = dictCache.get(props.code) || []
  return items
    .filter((i) => i.status !== 'disabled')
    .sort((a, b) => (a.sort ?? 0) - (b.sort ?? 0))
})

// 字典 value 是字符串；外部 v-model 可能 number；统一转字符串
const normalized = computed(() => {
  if (props.vModel === null || props.vModel === undefined) return props.multiple ? [] : ''
  if (Array.isArray(props.vModel)) return props.vModel.map(String)
  return String(props.vModel)
})

const handleChange = (val: string | string[]) => {
  if (props.multiple) {
    emit('update:vModel', Array.isArray(val) ? val : [])
  } else {
    // 单选：尽量还原原始类型；外部是 number → 转回 number
    const raw = Array.isArray(val) ? val[0] : val
    const isNum = typeof props.vModel === 'number'
    emit('update:vModel', raw === '' || raw === undefined ? '' : isNum ? Number(raw) : raw)
  }
}
</script>

<style scoped>
.on-dict-select {
  width: 100%;
}
</style>
