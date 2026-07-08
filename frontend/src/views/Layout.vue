<template>
  <el-container class="layout-container">
    <!-- 侧边栏 -->
    <el-aside width="220px" class="aside">
      <div class="logo">
        <img src="@/assets/imgs/logo.png" alt="OxNginx" class="logo-image" />
        <h2>OxNginx</h2>
      </div>

      <el-menu
        :default-active="route.path"
        router
        class="sidebar-menu"
        background-color="#001529"
        text-color="#ffffffb3"
        active-text-color="#ffffff"
      >
        <el-menu-item index="/dashboard">
          <el-icon><Odometer /></el-icon>
          <span>{{ $t('menu.dashboard') }}</span>
        </el-menu-item>

        <el-menu-item index="/sites">
          <el-icon><Grid /></el-icon>
          <span>{{ $t('menu.sites') }}</span>
        </el-menu-item>

        <el-menu-item index="/ssl">
          <el-icon><Lock /></el-icon>
          <span>{{ $t('menu.ssl') }}</span>
        </el-menu-item>

        <el-menu-item index="/upstreams">
          <el-icon><Connection /></el-icon>
          <span>{{ $t('menu.upstreams') }}</span>
        </el-menu-item>

        <el-menu-item index="/access">
          <el-icon><Key /></el-icon>
          <span>{{ $t('menu.access') }}</span>
        </el-menu-item>

        <el-menu-item index="/templates">
          <el-icon><Files /></el-icon>
          <span>{{ $t('menu.templates') }}</span>
        </el-menu-item>

        <el-menu-item index="/logs">
          <el-icon><Document /></el-icon>
          <span>{{ $t('menu.logs') }}</span>
        </el-menu-item>

        <el-menu-item index="/config">
          <el-icon><Edit /></el-icon>
          <span>{{ $t('menu.config') }}</span>
        </el-menu-item>

        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <span>{{ $t('menu.settings') }}</span>
        </el-menu-item>
      </el-menu>
    </el-aside>

    <!-- 主内容区 -->
    <el-container>
      <!-- 顶栏 -->
      <el-header class="header">
        <div class="header-left">
          <el-breadcrumb separator="/">
            <el-breadcrumb-item :to="{ path: '/' }">{{ $t('layout.home') }}</el-breadcrumb-item>
            <el-breadcrumb-item>{{ route.meta.title ? $t(route.meta.title as string) : '' }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>

        <div class="header-right">
          <!-- 语言切换 -->
          <el-dropdown @command="handleLocaleChange" trigger="click">
            <span class="lang-switch">
              {{ settingsStore.locale === 'zh-CN' ? '中' : 'EN' }}
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="zh-CN" :class="{ active: settingsStore.locale === 'zh-CN' }">中文</el-dropdown-item>
                <el-dropdown-item command="en-US" :class="{ active: settingsStore.locale === 'en-US' }">English</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>

          <!-- 主题色切换 -->
          <el-popover placement="bottom" :width="320" trigger="click">
            <template #reference>
              <el-icon :size="18" class="header-action"><Brush /></el-icon>
            </template>
            <el-color-picker-panel
              :model-value="settingsStore.themeColor"
              :predefine="themeColors"
              color-format="hex"
              @change="handleThemeChange"
            />
          </el-popover>

          <!-- 暗黑模式切换 -->
          <el-icon :size="18" class="header-action" @click="handleDarkToggle">
            <Moon v-if="!settingsStore.darkMode" />
            <Sunny v-else />
          </el-icon>

          <!-- 用户菜单 -->
          <el-dropdown @command="handleCommand">
            <span class="user-info">
              <el-icon><User /></el-icon>
              {{ authStore.username }}
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="change-username">{{ $t('layout.changeUsername') }}</el-dropdown-item>
                <el-dropdown-item command="change-password">{{ $t('layout.changePassword') }}</el-dropdown-item>
                <el-dropdown-item divided command="logout">{{ $t('layout.logout') }}</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>

      <!-- 内容 -->
      <el-main class="main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>

  <!-- 修改密码弹窗 -->
  <OnDialog v-model="pwdDialogVisible" :title="$t('layout.changePassword')" width="400px" :maximizable="false" :destroy-on-close="true">
    <el-form ref="pwdFormRef" :model="pwdForm" :rules="pwdRules" label-width="80px">
      <el-form-item :label="$t('layout.oldPassword')" prop="oldPassword">
        <el-input v-model="pwdForm.oldPassword" type="password" show-password :placeholder="$t('layout.enterOldPassword')" />
      </el-form-item>
      <el-form-item :label="$t('layout.newPassword')" prop="newPassword">
        <el-input v-model="pwdForm.newPassword" type="password" show-password :placeholder="$t('layout.enterNewPassword')" />
      </el-form-item>
      <el-form-item :label="$t('layout.confirmPassword')" prop="confirmPassword">
        <el-input v-model="pwdForm.confirmPassword" type="password" show-password :placeholder="$t('layout.enterConfirmPassword')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="pwdDialogVisible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="pwdLoading" @click="submitChangePassword">{{ $t('layout.confirmChange') }}</el-button>
    </template>
  </OnDialog>

  <!-- 修改账号弹窗 -->
  <OnDialog v-model="userDialogVisible" :title="$t('layout.changeUsername')" width="400px" :maximizable="false" :destroy-on-close="true">
    <el-form ref="userFormRef" :model="userForm" :rules="userRules" label-width="80px">
      <el-form-item :label="$t('layout.currentUsername')">
        <el-input :model-value="authStore.username" disabled />
      </el-form-item>
      <el-form-item :label="$t('layout.password')" prop="password">
        <el-input v-model="userForm.password" type="password" show-password :placeholder="$t('layout.enterPassword')" />
      </el-form-item>
      <el-form-item :label="$t('layout.newUsername')" prop="newUsername">
        <el-input v-model="userForm.newUsername" :placeholder="$t('layout.enterNewUsername')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="userDialogVisible = false">{{ $t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="userLoading" @click="submitChangeUsername">{{ $t('layout.confirmChange') }}</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const settingsStore = useSettingsStore()
const { locale } = useI18n()

// 同步 i18n locale 与持久化 store
locale.value = settingsStore.locale
watch(
  () => settingsStore.locale,
  (val) => {
    locale.value = val
  }
)

// 主题色预设
const themeColors = ['#409EFF', '#67C23A', '#E6A23C', '#F56C6C', '#9B59B6', '#00BCD4']

function handleLocaleChange(lang: string) {
  settingsStore.setLocale(lang as 'zh-CN' | 'en-US')
}

function handleThemeChange(color: string) {
  settingsStore.setThemeColor(color)
}

function handleDarkToggle() {
  settingsStore.toggleDarkMode()
}

function handleCommand(command: string) {
  if (command === 'logout') {
    authStore.logout()
    router.push('/login')
  } else if (command === 'change-password') {
    openPwdDialog()
  } else if (command === 'change-username') {
    openUserDialog()
  }
}

// ===== 修改密码 =====
const pwdDialogVisible = ref(false)
const pwdLoading = ref(false)
const pwdFormRef = ref<FormInstance>()
const pwdForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: '',
})

const pwdRules: FormRules = {
  oldPassword: [{ required: true, message: '请输入旧密码', trigger: 'blur' }],
  newPassword: [{ required: true, message: '请输入新密码', trigger: 'blur' }],
  confirmPassword: [
    { required: true, message: '请再次输入新密码', trigger: 'blur' },
    {
      validator: (_rule: any, value: string, callback: Function) => {
        if (value !== pwdForm.newPassword) {
          callback(new Error('两次输入的密码不一致'))
        } else {
          callback()
        }
      },
      trigger: 'blur',
    },
  ],
}

function openPwdDialog() {
  pwdForm.oldPassword = ''
  pwdForm.newPassword = ''
  pwdForm.confirmPassword = ''
  pwdDialogVisible.value = true
}

async function submitChangePassword() {
  const valid = await pwdFormRef.value?.validate().catch(() => false)
  if (!valid) return

  pwdLoading.value = true
  try {
    const res = await api.post('/api/change-password', {
      old_password: pwdForm.oldPassword,
      new_password: pwdForm.newPassword,
    })
    if (res.data.code === 0) {
      ElMessage.success('密码修改成功，请重新登录')
      pwdDialogVisible.value = false
      authStore.logout()
      router.push('/login')
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (err: any) {
    ElMessage.error(err.response?.data?.message || '修改密码失败')
  } finally {
    pwdLoading.value = false
  }
}

// ===== 修改账号 =====
const userDialogVisible = ref(false)
const userLoading = ref(false)
const userFormRef = ref<FormInstance>()
const userForm = reactive({
  password: '',
  newUsername: '',
})

const userRules: FormRules = {
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }],
  newUsername: [{ required: true, message: '请输入新用户名', trigger: 'blur' }],
}

function openUserDialog() {
  userForm.password = ''
  userForm.newUsername = ''
  userDialogVisible.value = true
}

async function submitChangeUsername() {
  const valid = await userFormRef.value?.validate().catch(() => false)
  if (!valid) return

  userLoading.value = true
  try {
    const res = await api.post('/api/change-username', {
      password: userForm.password,
      new_username: userForm.newUsername,
    })
    if (res.data.code === 0) {
      authStore.updateUser(res.data.data.token, res.data.data.username)
      ElMessage.success('账号修改成功')
      userDialogVisible.value = false
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (err: any) {
    ElMessage.error(err.response?.data?.message || '修改账号失败')
  } finally {
    userLoading.value = false
  }
}
</script>

<style scoped>
.layout-container {
  height: 100vh;
}

.aside {
  background-color: #001529;
  overflow: hidden;
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.logo h2 {
  margin: 0;
  font-size: 20px;
}

.logo-image {
  width: 36px;
  height: 36px;
  margin-right: 10px;
  border-radius: 6px;
  object-fit: contain;
}

.sidebar-menu {
  border-right: none;
}

.header {
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
}

.header-left {
  display: flex;
  align-items: center;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-action {
  cursor: pointer;
  color: var(--el-text-color-regular);
  transition: color 0.2s;
  display: flex;
  align-items: center;
}

.header-action:hover {
  color: var(--el-color-primary);
}

.lang-switch {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  color: var(--el-text-color-regular);
  background: var(--el-fill-color-light);
  transition: all 0.2s;
  user-select: none;
}

.lang-switch:hover {
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.user-info {
  display: flex;
  align-items: center;
  cursor: pointer;
  color: var(--el-text-color-primary);
}

.user-info .el-icon {
  margin-right: 4px;
}

.main {
  background: var(--el-bg-color-page);
  padding: 20px;
}

/* 暗黑模式适配 */
:global(html.dark) .lang-switch:hover {
  color: var(--el-color-primary);
  background: rgba(var(--el-color-primary-rgb), 0.15);
}

:global(html.dark) .main {
  background: #0a0a0a;
}
</style>
