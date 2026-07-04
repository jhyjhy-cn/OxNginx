<template>
  <div class="top-bar-right">
    <!-- 主题设置按钮 -->
    <el-icon class="action-icon" :size="18" @click="$emit('openThemeDrawer')"><Setting /></el-icon>
    <!-- 用户菜单 -->
    <el-dropdown @command="handleUserCommand" trigger="click">
      <span class="user-trigger">
        <el-icon :size="18"><User /></el-icon>
        <span class="username">{{ authStore.username }}</span>
        <el-icon :size="12"><ArrowDown /></el-icon>
      </span>
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item command="changeUsername"><el-icon><User /></el-icon>{{ t('layout.changeUsername') }}</el-dropdown-item>
          <el-dropdown-item command="changePassword"><el-icon><Lock /></el-icon>{{ t('layout.changePassword') }}</el-dropdown-item>
          <el-dropdown-item divided command="logout"><el-icon><SwitchButton /></el-icon>{{ t('layout.logout') }}</el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'

const { t } = useI18n()
const authStore = useAuthStore()

const emit = defineEmits<{
  openThemeDrawer: []
  changeUsername: []
  changePassword: []
}>()

function handleUserCommand(cmd: string) {
  if (cmd === 'logout') {
    authStore.logout()
  } else if (cmd === 'changeUsername') {
    emit('changeUsername')
  } else if (cmd === 'changePassword') {
    emit('changePassword')
  }
}
</script>

<style scoped>
.top-bar-right {
  display: flex;
  align-items: center;
  gap: 16px;
}
.action-icon {
  cursor: pointer;
  color: var(--el-text-color-regular);
}
.action-icon:hover {
  color: var(--el-color-primary);
}
.user-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  color: var(--el-text-color-regular);
  font-size: 14px;
}
.username {
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
