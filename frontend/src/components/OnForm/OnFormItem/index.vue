<template>
  <el-form-item
    v-if="visible"
    :prop="props.prop"
    :label="labelText(label)"
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
          :multiple="props.multiple"
        >
          <template v-if="props.options && hasOptionsSlot">
            <template v-if="props.type === 'select'">
              <component
                :is="'el-option'"
                v-for="opt in props.options"
                :key="opt.value"
                :label="isI18nKey(opt.label) ? t(opt.label) : opt.label"
                :value="opt.value"
                :disabled="opt.disabled"
              />
            </template>
            <template v-else-if="props.type === 'radio'">
              <component
                :is="'el-radio'"
                v-for="opt in props.options"
                :key="opt.value"
                :value="opt.value"
                :disabled="opt.disabled"
              >{{ isI18nKey(opt.label) ? t(opt.label) : opt.label }}</component>
            </template>
            <template v-else-if="props.type === 'checkbox'">
              <component
                :is="'el-checkbox'"
                v-for="opt in props.options"
                :key="opt.value"
                :label="opt.value"
                :disabled="opt.disabled"
              >{{ isI18nKey(opt.label) ? t(opt.label) : opt.label }}</component>
            </template>
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
    multiple?: boolean;
    rows?: number;
    min?: number;
    max?: number;
    format?: string;
    defaultValue?: any;
    visible?: boolean;
    viewFormat?: FormField["viewFormat"];
    labelWidth?: string | number;
    autocomplete?: string;
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

const componentMap: Record<string, string> = {
  input: "el-input",
  textarea: "el-input",
  password: "el-input",
  select: "el-select",
  radio: "el-radio-group",
  checkbox: "el-checkbox-group",
  switch: "el-switch",
  date: "el-date-picker",
  daterange: "el-date-picker",
  datetime: "el-date-picker",
  time: "el-time-picker",
  number: "el-input-number",
};

const currentComponent = computed(
  () => componentMap[props.type || "input"] || "el-input"
);

function isI18nKey(val?: string): boolean {
  return !!val && val.includes(".");
}

function labelText(val?: string): string {
  if (!val) return "";
  return isI18nKey(val) ? t(val) : val;
}

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
      p["active-value"] = 1;
      p["inactive-value"] = 0;
      break;
    case "date":
      p.type = "date";
      p.format = props.format || "YYYY-MM-DD";
      p.valueFormat = "YYYY-MM-DD";
      break;
    case "daterange":
      p.type = "daterange";
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
  if (props.autocomplete && (props.type === "input" || props.type === "textarea" || props.type === "password")) {
    p.autocomplete = props.autocomplete;
  }
  return p;
});

const mergedRules = computed(() => {
  const rules: any[] = [];
  if (props.required) {
    const labelTextValue = labelText(props.label || props.prop || "");
    rules.push({
      required: true,
      message: `请输入${labelTextValue}`,
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

const labelWidth = computed(() => {
  if (
    props.labelWidth === 0 ||
    props.labelWidth === "0" ||
    props.labelWidth === "0px"
  )
    return "0";
  return props.labelWidth;
});

const visible = computed(() => props.visible !== false);

const isViewMode = computed(() => props.viewMode === true);

const viewValue = computed(() => {
  if (props.viewFormat) {
    return props.viewFormat(props.modelValue, props as FormField);
  }
  if (props.options && props.modelValue !== undefined) {
    const opt = props.options.find((o) => o.value === props.modelValue);
    return labelText(opt?.label) || opt?.label || props.modelValue;
  }
  if (props.modelValue === null || props.modelValue === undefined) return "-";
  return String(props.modelValue);
});

const modelValue = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

const hasOptionsSlot = computed(() => {
  return ["select", "radio", "checkbox"].includes(props.type || "");
});

const computedPlaceholder = computed(() => {
  if (props.placeholder) return labelText(props.placeholder);
  if (props.label) return t("common.input") + labelText(props.label);
  return "";
});
</script>

<style scoped>
.el-image {
  border-radius: 4px;
}
</style>
