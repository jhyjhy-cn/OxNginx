<template>
  <div class="on-pagination">
    <el-pagination
      v-model:current-page="page"
      v-model:page-size="size"
      :total="total"
      :page-sizes="pageSizes"
      layout="total, sizes, prev, pager, next, jumper"
      @current-change="onCurrentChange"
      @size-change="onSizeChange"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    currentPage: number
    pageSize: number
    total: number
    pageSizes?: number[]
  }>(),
  {
    pageSizes: () => [10, 20, 50, 100],
  }
)

const emit = defineEmits<{
  'update:currentPage': [value: number]
  'update:pageSize': [value: number]
  change: [page: number, size: number]
}>()

const page = computed({
  get: () => props.currentPage,
  set: (v) => emit('update:currentPage', v),
})

const size = computed({
  get: () => props.pageSize,
  set: (v) => emit('update:pageSize', v),
})

// ponytail: 取 el-pagination 事件回传的新值。不能读 page.value/size.value ——
// 它们是 prop 的 computed,父级(OnTable)不走 v-model 回写时读到的永远是旧页码
function onCurrentChange(val: number) {
  emit('change', val, size.value)
}

function onSizeChange(val: number) {
  emit('change', 1, val)
}
</script>

<style scoped>
.on-pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}
</style>
