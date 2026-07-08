<template>
  <component
    :is="activeLayout"
    @open-theme-drawer="showThemeDrawer = true"
    @change-username="showUsernameDialog = true"
    @change-password="showPasswordDialog = true"
  />

  <!-- 主题配置抽屉 -->
  <ThemeDrawer :visible="showThemeDrawer" @close="showThemeDrawer = false" />

  <!-- 修改密码弹窗 -->
  <el-dialog v-model="showPasswordDialog" :title="t('layout.changePassword')" width="400px" append-to-body>
    <el-form :model="passwordForm" :rules="passwordRules" ref="passwordFormRef" label-width="100px">
      <el-form-item :label="t('layout.oldPassword')" prop="oldPassword">
        <el-input v-model="passwordForm.oldPassword" type="password" show-password :placeholder="t('layout.enterOldPassword')" />
      </el-form-item>
      <el-form-item :label="t('layout.newPassword')" prop="newPassword">
        <el-input v-model="passwordForm.newPassword" type="password" show-password :placeholder="t('layout.enterNewPassword')" />
      </el-form-item>
      <el-form-item :label="t('layout.confirmPassword')" prop="confirmPassword">
        <el-input v-model="passwordForm.confirmPassword" type="password" show-password :placeholder="t('layout.enterConfirmPassword')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="showPasswordDialog = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="submitPassword" :loading="passwordLoading">{{ t('layout.confirmChange') }}</el-button>
    </template>
  </el-dialog>

  <!-- 修改账号弹窗 -->
  <el-dialog v-model="showUsernameDialog" :title="t('layout.changeUsername')" width="400px" append-to-body>
    <el-form :model="usernameForm" :rules="usernameRules" ref="usernameFormRef" label-width="100px">
      <el-form-item :label="t('layout.currentUsername')">
        <el-input :model-value="authStore.username" disabled />
      </el-form-item>
      <el-form-item :label="t('layout.password')" prop="password">
        <el-input v-model="usernameForm.password" type="password" show-password :placeholder="t('layout.enterPassword')" />
      </el-form-item>
      <el-form-item :label="t('layout.newUsername')" prop="newUsername">
        <el-input v-model="usernameForm.newUsername" :placeholder="t('layout.enterNewUsername')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="showUsernameDialog = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="submitUsername" :loading="usernameLoading">{{ t('layout.confirmChange') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import api from '@/api/index'
import SidebarTreeLayout from './SidebarTreeLayout.vue'
import TopTreeLayout from './TopTreeLayout.vue'
import ThemeDrawer from './ThemeDrawer.vue'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()

// 根据 layoutMode 动态选择布局组件
const activeLayout = computed(() => {
  return settingsStore.layoutMode === 'top-tree' ? TopTreeLayout : SidebarTreeLayout
})

// 主题抽屉
const showThemeDrawer = ref(false)

// 修改密码
const showPasswordDialog = ref(false)
const passwordLoading = ref(false)
const passwordFormRef = ref()
const passwordForm = reactive({ oldPassword: '', newPassword: '', confirmPassword: '' })
const passwordRules = computed(() => ({
  oldPassword: [{ required: true, message: t('layout.enterOldPassword'), trigger: 'blur' }],
  newPassword: [{ required: true, message: t('layout.enterNewPassword'), trigger: 'blur' }],
  confirmPassword: [
    { required: true, message: t('layout.enterConfirmPassword'), trigger: 'blur' },
    {
      validator: (_rule: unknown, value: string, callback: (error?: Error) => void) =>
        value === passwordForm.newPassword ? callback() : callback(new Error(t('layout.passwordMismatch'))),
      trigger: 'blur',
    },
  ],
}))

async function submitPassword() {
  const valid = await passwordFormRef.value?.validate().catch(() => false)
  if (!valid) return
  passwordLoading.value = true
  try {
    await api.post('/api/user/password', { old_password: passwordForm.oldPassword, new_password: passwordForm.newPassword })
    ElMessage({ type: 'success', message: t('layout.passwordChanged') })
    showPasswordDialog.value = false
    authStore.logout()
  } catch {
    ElMessage({ type: 'error', message: t('layout.passwordChangeFailed') })
  } finally {
    passwordLoading.value = false
  }
}

// 修改账号
const showUsernameDialog = ref(false)
const usernameLoading = ref(false)
const usernameFormRef = ref()
const usernameForm = reactive({ password: '', newUsername: '' })
const usernameRules = computed(() => ({
  password: [{ required: true, message: t('layout.enterPassword'), trigger: 'blur' }],
  newUsername: [{ required: true, message: t('layout.enterNewUsername'), trigger: 'blur' }],
}))

async function submitUsername() {
  const valid = await usernameFormRef.value?.validate().catch(() => false)
  if (!valid) return
  usernameLoading.value = true
  try {
    const { data } = await api.post('/api/user/username', { password: usernameForm.password, new_username: usernameForm.newUsername })
    if (data.code === 0) {
      ElMessage({ type: 'success', message: t('layout.usernameChanged') })
      showUsernameDialog.value = false
      authStore.updateUser(data.data.token, usernameForm.newUsername)
    } else {
      ElMessage({ type: 'error', message: data.message || t('layout.usernameChangeFailed') })
    }
  } catch {
    ElMessage({ type: 'error', message: t('layout.usernameChangeFailed') })
  } finally {
    usernameLoading.value = false
  }
}

// 弹窗关闭时重置表单
watch(showPasswordDialog, (v) => {
  if (!v) {
    passwordForm.oldPassword = ''
    passwordForm.newPassword = ''
    passwordForm.confirmPassword = ''
    passwordFormRef.value?.resetFields()
  }
})
watch(showUsernameDialog, (v) => {
  if (!v) {
    usernameForm.password = ''
    usernameForm.newUsername = ''
    usernameFormRef.value?.resetFields()
  }
})
</script>
