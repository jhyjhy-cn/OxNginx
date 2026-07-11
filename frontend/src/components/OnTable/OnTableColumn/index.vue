<template>
  <!-- 选择列 -->
  <el-table-column v-if="column.type === 'selection'" type="selection" :width="column.width || 48" :fixed="column.fixed" />

  <!-- 索引列 -->
  <el-table-column v-else-if="column.type === 'index'" type="index" :label="labelText(column.label)" :width="column.width" :fixed="column.fixed" />

  <!-- 展开列 -->
  <el-table-column v-else-if="column.type === 'expand'" type="expand" :width="column.width" :fixed="column.fixed" />

  <!-- 多级表头 -->
  <el-table-column v-else-if="column.children?.length" :label="labelText(column.label)" :width="column.width" :align="column.align" :fixed="column.fixed">
    <OnTableColumn v-for="child in column.children" :key="child.prop" :column="child" />
  </el-table-column>

  <!-- 普通列 -->
  <el-table-column v-else v-bind="column">
    <!-- 自定义表头 -->
    <template #header>
      <slot name="header">
        <span class="on-table-header-label">{{ labelText(column.label) }}</span>
      </slot>
    </template>

    <!-- 单元格内容 -->
    <template #default="{ row, $index }">
      <!-- 图片 -->
      <el-image v-if="column.type === 'image'" :src="row[column.prop!]" :preview-src-list="[row[column.prop!]]" fit="cover" :style="imageStyle" preview-teleported />

      <!-- 日期 -->
      <template v-else-if="column.type === 'date'">
        {{ formatDate(row[column.prop!], column.dateFormat) }}
      </template>

      <!-- 按钮组 -->
      <el-button-group v-else-if="column.buttons?.length">
        <el-button
          v-for="(btn, idx) in column.buttons"
          :key="idx"
          :size="btn.size || 'small'"
          :type="btn.type"
          :disabled="isBtnDisabled(btn, row)"
          @click.stop="emit('command', btn.command, row)"
        >
          {{ tBtn(btn.name) }}
        </el-button>
      </el-button-group>

      <!-- 自定义插槽 -->
      <slot v-else-if="column.slot" :name="column.slot" :row="row" :index="$index" />

      <!-- 渲染函数 -->
      <component v-else-if="column.render" :is="column.render" :row="row" :index="$index" />

      <!-- 默认 -->
      <span v-else>{{ row[column.prop!] }}</span>
    </template>
  </el-table-column>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { TableColumn, ColumnButton } from "../types";

defineOptions({ name: "OnTableColumn" });

const { t } = useI18n();

const props = defineProps<{
  column: TableColumn;
}>();

const emit = defineEmits<{
  command: [command: string | number, row: any];
}>();

const imageStyle = computed(() => {
  const size = props.column.imageSize || 40;
  return { width: `${size}px`, height: `${size}px` };
});

function formatDate(value: any, _format?: string) {
  if (!value) return "-";
  const str = String(value);
  if (str.length === 10) {
    const d = new Date(Number(value) * 1000);
    return d.toLocaleDateString("zh-CN");
  } else if (str.length === 13) {
    return new Date(value).toLocaleDateString("zh-CN");
  }
  try {
    return new Date(value).toLocaleDateString("zh-CN");
  } catch {
    return value;
  }
}

function labelText(label?: string) {
  if (!label) return "";
  return label.includes(".") ? t(label) : label;
}

function tBtn(name: string | { zh: string; en: string }) {
  if (typeof name === "object") {
    return name.zh;
  }
  return labelText(name);
}

function isBtnDisabled(btn: ColumnButton, row: any) {
  if (typeof btn.disabled === "function") {
    return btn.disabled(row);
  }
  return btn.disabled;
}
</script>

<style scoped>
.on-table-header-label {
  white-space: nowrap;
}

.el-image {
  border-radius: 4px;
}
</style>
