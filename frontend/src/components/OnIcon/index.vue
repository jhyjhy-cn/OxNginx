<template>
  <!-- Element Plus 图标 -->
  <el-icon v-if="isEpIcon" :size="sizeNum" :color="color" :class="className" :style="style">
    <component :is="epIconName" />
  </el-icon>
  <!-- SVG 组件 -->
  <el-icon v-else-if="svgComponent" :size="sizeNum" :color="color" :class="className" :style="style" class="on-icon-svg-wrapper">
    <component :is="svgComponent" />
  </el-icon>
  <!-- SVG Sprite（vite-plugin-svg-icons） -->
  <svg
    v-else
    aria-hidden="true"
    class="on-icon-svg"
    :class="className"
    :style="svgStyle"
  >
    <use :xlink:href="symbolId" />
  </svg>
</template>

<script setup lang="ts">
import { type Component, computed } from 'vue'
import * as EpIcons from '@element-plus/icons-vue'

/**
 * 通用图标组件
 * - Element Plus 图标：<OnIcon name="Setting" :size="18" color="#fff" />
 * - SVG 文件名引用：<OnIcon svgName="translate" :size="18" />
 * - SVG 组件传入：<OnIcon :svg="MySvgComponent" :size="18" />
 * - SVG Sprite：<OnIcon name="svg:my-icon" :size="24" />
 *
 * SVG 文件放在 src/assets/svgs/ 目录，按文件名自动扫描，无需手动导入
 */

// 自动扫描 src/assets/svgs/ 下所有 SVG，构建 文件名 → 组件 的映射
const svgModules = import.meta.glob('@/assets/svgs/*.svg', { eager: true }) as Record<string, { default: Component }>
const svgMap: Record<string, Component> = {}
for (const [path, mod] of Object.entries(svgModules)) {
  // 从路径提取文件名（不含扩展名）：/src/assets/svgs/translate.svg → translate
  const name = path.split('/').pop()!.replace(/\.svg$/, '')
  svgMap[name] = mod.default
}

const props = withDefaults(defineProps<{
  /** Element Plus 图标名称 */
  name?: string
  /** SVG 文件名（不含路径和扩展名），自动从 src/assets/svgs/ 查找 */
  svgName?: string
  /** SVG Vue 组件（手动传入） */
  svg?: Component
  /** 尺寸，数字为 px，字符串直接作为 CSS 值 */
  size?: number | string
  /** 颜色 */
  color?: string
  /** 自定义 class */
  className?: string
  /** 自定义 style */
  style?: string
  /** SVG Sprite 前缀 */
  prefix?: string
}>(), {
  name: '',
  svgName: '',
  size: 16,
  color: '',
  className: '',
  style: '',
  prefix: 'icon',
})

/** 最终的 SVG 组件（svgName 自动查找优先，其次手动传入的 svg） */
const svgComponent = computed(() => {
  if (props.svgName) return svgMap[props.svgName] ?? null
  return props.svg ?? null
})

/** 是否为 Element Plus 图标 */
const isEpIcon = computed(() => !!props.name && !props.name.startsWith('svg:'))

/** Element Plus 图标组件名称（支持 kebab-case → PascalCase） */
const epIconName = computed(() => {
  const raw = props.name
  return raw in EpIcons
    ? raw
    : raw.replace(/(^|-)(\w)/g, (_, _sep, c: string) => c.toUpperCase())
})

/** 尺寸转数字 */
const sizeNum = computed(() => typeof props.size === 'number' ? props.size : parseInt(props.size) || 16)

/** SVG Sprite symbolId */
const symbolId = computed(() => {
  const raw = props.name.replace(/^svg:/, '')
  return `#${props.prefix}-${raw}`
})

/** SVG Sprite style */
const svgStyle = computed(() => {
  const s = typeof props.size === 'number' ? `${props.size}px` : props.size
  return `width:${s};height:${s};color:${props.color};${props.style}`
})
</script>

<style>
.on-icon-svg {
  display: inline-block;
  vertical-align: -0.15em;
  fill: currentColor;
  overflow: hidden;
}
/* SVG 组件内 path 使用 currentColor 而非硬编码 fill */
.on-icon-svg-wrapper svg path {
  fill: currentColor;
}
</style>
