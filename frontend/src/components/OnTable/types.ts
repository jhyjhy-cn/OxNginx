import type { VNodeChild } from "vue";

// 列类型
export type ColumnType = "selection" | "index" | "expand" | "image" | "date";

// 对齐方式
export type Align = "left" | "center" | "right";

// 固定列
export type Fixed = "left" | "right";

// 按钮配置
export interface ColumnButton {
  name: string | { zh: string; en: string }; // 支持 i18n
  command: string | number;
  type?: "primary" | "success" | "warning" | "danger" | "info";
  size?: "large" | "default" | "small";
  disabled?: boolean | ((row: any) => boolean);
}

// 列配置
export interface TableColumn {
  type?: ColumnType; // selection/index/expand
  label?: string; // 支持 i18n key
  prop?: string;
  slot?: string; // 自定义插槽名
  width?: number | string;
  minWidth?: number | string;
  align?: Align;
  fixed?: Fixed;

  // 日期格式化
  dateFormat?: string;

  // 图片
  imageSize?: number;

  // 按钮组
  buttons?: ColumnButton[];

  // 渲染函数
  render?: (row: any, index: number) => VNodeChild;

  // 排序
  sortable?: boolean | "custom";

  // 多级表头
  children?: TableColumn[];

  // 显示控制
  visible?: boolean | ((row?: any) => boolean);

  // 合计
  sum?: boolean;

  // tooltip
  showOverflowTooltip?: boolean;
}

// 表格选项
export interface TableOptions {
  data?: any[];
  height?: string | number | "auto";
  maxHeight?: string | number;
  stripe?: boolean;
  border?: boolean;
  size?: "large" | "default" | "small";
  showHeader?: boolean;
  showSummary?: boolean;
  rowKey?: string;
  defaultSort?: { prop: string; order: "ascending" | "descending" };
  showPagination?: boolean;
  spanMethod?: (data: { row: any; column: number; rowIndex: number; columnIndex: number }) => [number, number] | void;
}

// 表格事件
export interface TableEmits {
  (e: "selection-change", selection: any[]): void;
  (e: "sort-change", sort: { prop: string; order: "ascending" | "descending" }): void;
  (e: "row-click", row: any, column: any, event: Event): void;
  (e: "command", command: string | number, row: any): void;
}
