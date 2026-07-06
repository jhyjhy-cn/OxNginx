<template>
  <div class="top-bar-right">
    <!-- 主题色 -->
    <el-color-picker
      :model-value="settingsStore.themeColor"
      @change="handleColorChange"
      :predefine="presetColors"
      size="small"
    />
    <!-- 语言切换 -->
    <el-dropdown @command="handleLanguageChange" trigger="click">
      <OnIcon svgName="translate" :size="18" class="action-icon" />
      <template #dropdown>
        <el-dropdown-menu>
          <el-dropdown-item command="zh-CN" :class="{ active: settingsStore.locale === 'zh-CN' }">中文</el-dropdown-item>
          <el-dropdown-item command="en-US" :class="{ active: settingsStore.locale === 'en-US' }">English</el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
    <!-- 暗黑模式切换 -->
    <el-icon class="action-icon" :size="18" @click="settingsStore.toggleDarkMode($event)">
      <Moon v-if="!settingsStore.darkMode" />
      <Sunny v-else />
    </el-icon>
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
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'

const { t, locale } = useI18n()
const router = useRouter()
const authStore = useAuthStore()
const settingsStore = useSettingsStore()

const presetColors = ['#409EFF', '#536dfe', '#9c27b0', '#00bfa5', '#ff5722', '#e91e63']

function handleColorChange(color: string | null) {
  if (color) settingsStore.setThemeColor(color)
}

const emit = defineEmits<{
  openThemeDrawer: []
  changeUsername: []
  changePassword: []
}>()

function handleLanguageChange(lang: 'zh-CN' | 'en-US') {
  settingsStore.setLocale(lang)
  locale.value = lang
}

function handleUserCommand(cmd: string) {
  if (cmd === 'logout') {
    authStore.logout()
    router.push('/login')
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
:deep(.el-dropdown-menu__item.active) {
  color: var(--el-color-primary);
  font-weight: 600;
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
