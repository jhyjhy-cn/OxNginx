<template>
  <div class="on-table">
    <!-- 操作栏 -->
    <div class="on-table__toolbar">
      <div class="on-table__toolbar-left">
        <slot name="toolbar-left" />
      </div>
      <div class="on-table__toolbar-right">
        <slot name="toolbar-right" />
        <el-button :icon="Refresh" :title="t('common.refresh')" @click="handleReload" />
        <OnTableColumnConfig :columns="localColumns" @update:columns="handleColumnsUpdate" />
      </div>
    </div>

    <!-- 表格 -->
    <el-table
      ref="tableRef"
      v-bind="$attrs"
      :data="props.data"
      :height="tableHeight"
      :max-height="maxHeight"
      :stripe="tableOptions.stripe"
      :border="tableOptions.border"
      :highlight-current-row="tableOptions.highlightCurrentRow"
      :size="props.options?.size"
      :show-header="props.options?.showHeader"
      :row-key="props.options?.rowKey"
      :default-sort="props.options?.defaultSort"
      :span-method="props.options?.spanMethod"
      :show-summary="props.options?.showSummary"
      :summary-method="getSummaries"
      v-loading="props.loading"
      @selection-change="emit('selectionChange', $event)"
      @sort-change="handleSortChange"
      @row-click="handleRowClick"
    >
      <!-- 遍历列 -->
      <template v-for="col in visibleColumns" :key="col.prop">
        <OnTableColumn :column="col" @command="(cmd, row) => emit('command', cmd, row)">
          <!-- 自定义插槽透传 -->
          <template v-if="col.slot && $slots[col.slot]" #[col.slot]="scope">
            <slot :name="col.slot" v-bind="scope" />
          </template>
        </OnTableColumn>
      </template>
    </el-table>

    <!-- 分页 -->
    <OnPagination
      v-if="showPagination"
      :current-page="currentPage"
      :page-size="pageSize"
      :total="total"
      @change="handlePageChange"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onBeforeUnmount } from "vue";
import { useI18n } from "vue-i18n";
import { Refresh } from "@element-plus/icons-vue";
import type { TableColumn, TableOptions } from "./types";
import OnTableColumn from "./OnTableColumn/index.vue";
import OnTableColumnConfig from "./OnTableColumnConfig/index.vue";
import OnPagination from "../OnPagination/index.vue";

const { t } = useI18n();

defineOptions({ name: "OnTable", inheritAttrs: false });

const props = withDefaults(
  defineProps<{
    data?: any[];
    columns: TableColumn[];
    options?: TableOptions;
    loading?: boolean;
    pagination?: boolean | { total?: number; currentPage?: number; pageSize?: number };
  }>(),
  {
    loading: false,
    pagination: true,
  }
);

// 默认选项
const tableOptions = computed(() => ({
  border: true,
  stripe: true,
  highlightCurrentRow: true,
  ...props.options,
}));

const emit = defineEmits<{
  (e: "selectionChange", selection: any[]): void;
  (e: "sortChange", sort: { prop: string; order: "ascending" | "descending" }): void;
  (e: "rowClick", row: any, column: any, event: Event): void;
  (e: "command", command: string | number, row: any): void;
  (e: "pageChange", page: number, size: number): void;
  (e: "reload"): void;
  (e: "update:columns", columns: TableColumn[]): void;
}>();

// 列管理
const localColumns = ref<TableColumn[]>([...props.columns]);

watch(
  () => props.columns,
  (cols) => {
    localColumns.value = [...cols];
  }
);

function handleColumnsUpdate(columns: TableColumn[]) {
  localColumns.value = columns;
  emit("update:columns", columns);
}

function handleReload() {
  emit("reload");
}

// 分页状态
const pagination = computed(() => {
  if (typeof props.pagination === "boolean") {
    return { show: props.pagination, total: 0, currentPage: 1, pageSize: 20 };
  }
  return { show: true, ...props.pagination };
});

const showPagination = computed(() => pagination.value.show);
const total = computed(() => pagination.value.total || 0);
const currentPage = ref(pagination.value.currentPage || 1);
const pageSize = ref(pagination.value.pageSize || 20);

// 表格高度
const tableHeight = computed(() => {
  const h = props.options?.height;
  if (h === "auto") return undefined;
  if (h) return h;
  return "50vh";
});

// 自动高度（使用 max-height）
const maxHeight = computed(() => {
  if (props.options?.height === "auto") return autoHeight.value;
  return props.options?.maxHeight;
});

// 可见列
const visibleColumns = computed(() => {
  return localColumns.value.filter((col) => {
    if (typeof col.visible === "function") {
      return col.visible();
    }
    return col.visible !== false;
  });
});

// 合计列
const sumColumns = computed(() => {
  return visibleColumns.value.filter((col) => col.sum);
});

function getSummaries(param: any) {
  const { columns, data } = param;
  const sums: any[] = [];

  columns.forEach((column: any, index: number) => {
    if (index === 0) {
      sums[index] = "合计";
      return;
    }

    const col = sumColumns.value.find((c) => c.prop === column.property);
    if (col) {
      const values = data.map((row: any) => Number(row[column.property]) || 0);
      const sum = values.reduce((a: number, b: number) => a + b, 0);
      sums[index] = sum.toFixed(2);
    } else {
      sums[index] = "";
    }
  });

  return sums;
}

// 自动高度
const tableRef = ref();
const autoHeight = ref<number>();

function updateAutoHeight() {
  if (props.options?.height !== "auto" || !tableRef.value) return;

  const el = tableRef.value.$el as HTMLElement;
  if (!el) return;

  const rect = el.getBoundingClientRect();
  const offsetTop = rect.top;
  autoHeight.value = window.innerHeight - offsetTop - 85; // 85 = 分页高度 + margin
}

onMounted(() => {
  if (props.options?.height === "auto") {
    updateAutoHeight();
    window.addEventListener("resize", updateAutoHeight);
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateAutoHeight);
});

function handleSortChange(sort: any) {
  emit("sortChange", sort);
}

function handleRowClick(row: any, column: any, event: Event) {
  emit("rowClick", row, column, event);
}

function handlePageChange() {
  emit("pageChange", currentPage.value, pageSize.value);
}

// 暴露方法
defineExpose({
  tableRef,
  currentPage,
  pageSize,
});
</script>

<style scoped>
.on-table {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.on-table__toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.on-table__toolbar-left,
.on-table__toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.on-table :deep(.el-table) {
  height: auto;
  flex: 1;
}

.on-table :deep(.el-table__body-wrapper) {
  flex: 1;
  overflow-y: auto;
}
</style>
