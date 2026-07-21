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
    <!--
      ponytail: 曾经用 <component :is="currentComponent"> 按类型字符串动态渲染,
      但 element-plus 走 unplugin 按需注册(非全局),动态字符串解析不到组件,输入框整个不渲染。
      改为静态标签逐个分支,unplugin 才能静态解析 + 自动注入样式。
    -->
    <template v-else>
      <slot>
        <!-- input / textarea / password 都是 el-input -->
        <el-input
          v-if="inputLike"
          v-model="modelValue"
          :type="inputType"
          :rows="props.type === 'textarea' ? props.rows || 3 : undefined"
          :show-password="props.type === 'password' ? props.showPassword !== false : undefined"
          :autocomplete="props.autocomplete"
          :disabled="props.disabled || props.readonly"
          :placeholder="computedPlaceholder"
          :clearable="props.clearable"
        />
        <el-select
          v-else-if="props.type === 'select'"
          v-model="modelValue"
          filterable
          :multiple="props.multiple"
          :disabled="props.disabled || props.readonly"
          :placeholder="computedPlaceholder"
          :clearable="props.clearable"
        >
          <el-option
            v-for="opt in props.options"
            :key="opt.value"
            :label="labelText(opt.label)"
            :value="opt.value"
            :disabled="opt.disabled"
          />
        </el-select>
        <el-radio-group
          v-else-if="props.type === 'radio'"
          v-model="modelValue"
          :disabled="props.disabled || props.readonly"
        >
          <el-radio v-for="opt in props.options" :key="opt.value" :value="opt.value" :disabled="opt.disabled">{{
            labelText(opt.label)
          }}</el-radio>
        </el-radio-group>
        <el-checkbox-group
          v-else-if="props.type === 'checkbox'"
          v-model="modelValue"
          :disabled="props.disabled || props.readonly"
        >
          <el-checkbox v-for="opt in props.options" :key="opt.value" :label="opt.value" :disabled="opt.disabled">{{
            labelText(opt.label)
          }}</el-checkbox>
        </el-checkbox-group>
        <el-switch
          v-else-if="props.type === 'switch'"
          v-model="modelValue"
          :active-value="1"
          :inactive-value="0"
          :disabled="props.disabled || props.readonly"
        />
        <el-date-picker
          v-else-if="datePickerType"
          v-model="modelValue"
          :type="datePickerType"
          :format="dateFormat"
          :value-format="dateFormat"
          :disabled="props.disabled || props.readonly"
          :placeholder="computedPlaceholder"
          :clearable="props.clearable"
        />
        <el-time-picker
          v-else-if="props.type === 'time'"
          v-model="modelValue"
          format="HH:mm:ss"
          value-format="HH:mm:ss"
          :disabled="props.disabled || props.readonly"
          :placeholder="computedPlaceholder"
          :clearable="props.clearable"
        />
        <el-input-number
          v-else-if="props.type === 'number'"
          v-model="modelValue"
          :min="props.min"
          :max="props.max"
          :precision="0"
          :disabled="props.disabled || props.readonly"
        />
        <el-input
          v-else
          v-model="modelValue"
          :disabled="props.disabled || props.readonly"
          :placeholder="computedPlaceholder"
          :clearable="props.clearable"
        />
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

// ---- 控件分支计算（模板用静态标签，见上方注释）----
// input / textarea / password 统一是 el-input
const inputLike = computed(() => ["input", "textarea", "password"].includes(props.type || "input"))

const inputType = computed(() => {
  if (props.type === "textarea") return "textarea"
  if (props.type === "password") return "password"
  return undefined
})

// date / daterange / datetime 统一是 el-date-picker
const datePickerType = computed(() => {
  if (props.type === "date" || props.type === "daterange" || props.type === "datetime") return props.type
  return ""
})

const dateFormat = computed(() => {
  if (props.format) return props.format
  return props.type === "datetime" ? "YYYY-MM-DD HH:mm:ss" : "YYYY-MM-DD"
})

function isI18nKey(val?: string): boolean {
  return !!val && val.includes(".");
}

function labelText(val?: string): string {
  if (!val) return "";
  return isI18nKey(val) ? t(val) : val;
}

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
