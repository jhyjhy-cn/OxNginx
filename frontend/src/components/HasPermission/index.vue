<template>
  <slot v-if="hasPerm" />
</template>

<script setup lang="ts">
/**
 * 权限按钮组件 —— 按权限码控制 slot 是否渲染
 *
 * 用法:
 *   <HasPermission code="sys:site:add">
 *     <el-button @click="add">新增</el-button>
 *   </HasPermission>
 *
 * 约定:
 *   - props.code: 权限码字符串,与后端 sys_menus.permission 一致
 *   - super_admin (username=='admin' 或 roles 含 super_admin) 一律放行
 *   - 仅用于 UI 隐藏,后端仍需独立鉴权
 */
import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'

const props = defineProps<{
  code: string
}>()

const authStore = useAuthStore()
const hasPerm = computed(() => authStore.hasPermission(props.code))
</script>