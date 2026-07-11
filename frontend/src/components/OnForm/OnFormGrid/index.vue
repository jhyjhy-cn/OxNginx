<template>
  <el-row :gutter="props.gutter" class="on-form-grid">
    <slot name="prepend" />
    <el-col
      v-for="field in visibleFields"
      :key="field.prop"
      :span="field.span || props.span"
    >
      <OnFormItem
        v-model="props.model[field.prop]"
        :prop="field.prop"
        :label="field.label"
        :type="field.type"
        :disabled="field.disabled"
        :readonly="field.readonly"
        :view-mode="field.viewMode ?? props.viewMode"
        :required="field.required"
        :rules="field.rules"
        :placeholder="field.placeholder"
        :clearable="field.clearable"
        :options="field.options"
        :show-password="field.showPassword"
        :multiple="field.multiple"
        :rows="field.rows"
        :min="field.min"
        :max="field.max"
        :format="field.format"
        :visible="field.visible"
        :view-format="field.viewFormat"
        :autocomplete="field.autocomplete"
      />
    </el-col>
    <slot name="append" />
  </el-row>
</template>

<script setup lang="ts">
import { computed, provide, ref } from "vue";
import type { OnFormGridProps } from "../types";
import OnFormItem from "../OnFormItem/index.vue";

defineOptions({ name: "OnFormGrid" });

const props = withDefaults(defineProps<OnFormGridProps>(), {
  span: 24,
  gutter: 18,
  viewMode: false,
});

const formRef = ref();

const visibleFields = computed(() => {
  return props.fields.filter((f) => f.visible !== false);
});

provide("onForm", { formRef });
</script>

<style scoped>
.on-form-grid {
  width: 100%;
}
</style>
