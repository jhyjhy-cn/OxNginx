<template>
  <el-form-item
    v-if="visible"
    :prop="props.prop"
    :label="t(label || '')"
    :label-width="labelWidth"
    :rules="mergedRules"
    v-bind="$attrs"
  >
    <!-- 查看模式 -->
    <template v-if="isViewMode">
      <slot name="view">
        <span>{{ viewValue }}</span>
      </slot>
    </template>

    <!-- 编辑模式 -->
    <template v-else>
      <slot>
        <component
          :is="currentComponent"
          v-model="modelValue"
          v-bind="componentProps"
          :disabled="props.disabled || props.readonly"
          :placeholder="computedPlaceholder"
          :clearable="props.clearable"
        >
          <template v-if="props.options && hasOptionsSlot">
            <component
              :is="'el-option'"
              v-for="opt in props.options"
              :key="opt.value"
              :label="t(opt.label)"
              :value="opt.value"
              :disabled="opt.disabled"
            />
          </template>
        </component>
      </slot>
    </template>
  </el-form-item>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { FormField } from "../types";

defineOptions({ name: "OnFormItem", inheritAttrs: false });

const { t } = useI18n();

const props = withDefaults(
  defineProps<{
    prop: string;
    label?: string;
    type?: FormField["type"];
    modelValue?: any;
    disabled?: boolean;
    readonly?: boolean;
    viewMode?: boolean;
    required?: boolean;
    rules?: FormField["rules"];
    placeholder?: string;
    clearable?: boolean;
    options?: FormField["options"];
    showPassword?: boolean;
    rows?: number;
    min?: number;
    max?: number;
    format?: string;
    defaultValue?: any;
    visible?: boolean;
    viewFormat?: FormField["viewFormat"];
    labelWidth?: string | number;
  }>(),
  {
    type: "input",
    clearable: true,
    visible: true,
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: any];
}>();

// 计算组件类型映射
const componentMap: Record<string, string> = {
  input: "el-input",
  textarea: "el-input",
  password: "el-input",
  select: "el-select",
  radio: "el-radio-group",
  checkbox: "el-checkbox-group",
  switch: "el-switch",
  date: "el-date-picker",
  datetime: "el-date-picker",
  time: "el-time-picker",
  number: "el-input-number",
};

const currentComponent = computed(
  () => componentMap[props.type || "input"] || "el-input"
);

// 组件属性
const componentProps = computed(() => {
  const p: Record<string, any> = {};
  switch (props.type) {
    case "textarea":
      p.type = "textarea";
      p.rows = props.rows || 3;
      break;
    case "password":
      p.type = "password";
      p.showPassword = props.showPassword !== false;
      break;
    case "select":
      p.filterable = true;
      break;
    case "radio":
    case "checkbox":
      p.placeholder = undefined;
      break;
    case "switch":
      p.placeholder = undefined;
      break;
    case "date":
      p.type = "date";
      p.format = props.format || "YYYY-MM-DD";
      p.valueFormat = "YYYY-MM-DD";
      break;
    case "datetime":
      p.type = "datetime";
      p.format = props.format || "YYYY-MM-DD HH:mm:ss";
      p.valueFormat = "YYYY-MM-DD HH:mm:ss";
      break;
    case "time":
      p.format = props.format || "HH:mm:ss";
      p.valueFormat = "HH:mm:ss";
      break;
    case "number":
      p.min = props.min;
      p.max = props.max;
      p.precision = 0;
      break;
  }
  return p;
});

// 合并校验规则
const mergedRules = computed(() => {
  const rules: any[] = [];
  if (props.required) {
    const labelText = t(props.label || props.prop || "");
    rules.push({
      required: true,
      message: `请输入${labelText}`,
      trigger: "blur",
    });
  }
  if (props.rules) {
    const rawRules = Array.isArray(props.rules) ? props.rules : [props.rules];
    for (const rule of rawRules) {
      if (rule && typeof rule === "object") {
        if (rule.message) {
          const msg = t(String(rule.message));
          rules.push({ ...rule, message: msg !== rule.message ? msg : rule.message });
        } else {
          rules.push(rule);
        }
      } else {
        rules.push(rule);
      }
    }
  }
  return rules.length > 0 ? rules : undefined;
});

// 标签宽度
const labelWidth = computed(() => {
  if (
    props.labelWidth === 0 ||
    props.labelWidth === "0" ||
    props.labelWidth === "0px"
  )
    return "0";
  return props.labelWidth;
});

// 是否可见
const visible = computed(() => props.visible !== false);

// 查看模式
const isViewMode = computed(() => props.viewMode === true);

// 查看值格式化
const viewValue = computed(() => {
  if (props.viewFormat) {
    return props.viewFormat(props.modelValue, props as FormField);
  }
  if (props.options && props.modelValue !== undefined) {
    const opt = props.options.find((o) => o.value === props.modelValue);
    return t(opt?.label ?? "") || opt?.label || props.modelValue;
  }
  if (props.modelValue === null || props.modelValue === undefined) return "-";
  return String(props.modelValue);
});

// v-model
const modelValue = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

// 是否需要 options slot
const hasOptionsSlot = computed(() => {
  return ["select", "radio", "checkbox"].includes(props.type || "");
});

// 占位符：优先用传入值，其次 "请输入" + label
const computedPlaceholder = computed(() => {
  if (props.placeholder) return t(props.placeholder);
  if (props.label) return t("common.input") + t(props.label);
  return "";
});
</script>

<style scoped>
.el-form-item {
  margin-bottom: 18px;
}

.el-input,
.el-select,
.el-date-editor,
.el-time-picker,
.el-input-number {
  width: 100%;
}
</style>
