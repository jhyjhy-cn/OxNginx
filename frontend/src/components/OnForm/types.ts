import type { FormItemRule } from "element-plus";
import type { Component } from "vue";

// 表单项类型
export type ItemType =
  | "input"
  | "textarea"
  | "password"
  | "select"
  | "radio"
  | "checkbox"
  | "switch"
  | "date"
  | "daterange"
  | "datetime"
  | "time"
  | "number";

// 选项配置
export interface ItemOption {
  label: string;
  value: any;
  disabled?: boolean;
}

// 表单字段配置
export interface FormField {
  // 基础
  prop: string;
  label?: string; // 支持 i18n key 或原文
  type?: ItemType;
  span?: number; // 栅格跨度

  // 值
  defaultValue?: any;
  placeholder?: string; // 支持 i18n key 或原文

  // 状态
  disabled?: boolean;
  readonly?: boolean;
  clearable?: boolean;

  // 校验
  required?: boolean;
  rules?: FormItemRule | FormItemRule[];

  // 组件特定
  options?: ItemOption[]; // select/radio/checkbox 用
  multiple?: boolean; // select 多选
  showPassword?: boolean;
  rows?: number;
  min?: number;
  max?: number;
  format?: string;
  autocomplete?: string; // off / on / new-password，input/textarea/password 有效

  // 高级
  visible?: boolean;
  component?: Component | string;
  slots?: Record<string, any>;

  // 查看模式
  viewMode?: boolean;
  viewFormat?: (value: any, field: FormField) => any;
}

// OnForm 属性
export interface OnFormProps {
  model: Record<string, any>;
  fields?: FormField[];
  labelWidth?: string | number;
  labelPosition?: "left" | "right" | "top";
  disabled?: boolean;
  readonly?: boolean;
  viewMode?: boolean;
  span?: number;
  gutter?: number;
}

// OnFormGrid 属性
export interface OnFormGridProps {
  fields: FormField[];
  model: Record<string, any>;
  span?: number;
  gutter?: number;
  viewMode?: boolean;
}
