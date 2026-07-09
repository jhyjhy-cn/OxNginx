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

function onCurrentChange() {
  emit('change', page.value, size.value)
}

function onSizeChange() {
  emit('change', 1, size.value)
}
</script>

<style scoped>
.on-pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}
</style>
