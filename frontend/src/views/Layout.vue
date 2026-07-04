<template>
  <el-container class="layout-container">
    <!-- 侧边栏 -->
    <el-aside width="220px" class="aside">
      <div class="logo">
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
          <span>仪表盘</span>
        </el-menu-item>

        <el-menu-item index="/sites">
          <el-icon><Grid /></el-icon>
          <span>站点管理</span>
        </el-menu-item>

        <el-menu-item index="/ssl">
          <el-icon><Lock /></el-icon>
          <span>SSL证书</span>
        </el-menu-item>

        <el-menu-item index="/upstreams">
          <el-icon><Connection /></el-icon>
          <span>负载均衡</span>
        </el-menu-item>

        <el-menu-item index="/access">
          <el-icon><Key /></el-icon>
          <span>访问控制</span>
        </el-menu-item>

        <el-menu-item index="/templates">
          <el-icon><Files /></el-icon>
          <span>配置模板</span>
        </el-menu-item>

        <el-menu-item index="/logs">
          <el-icon><Document /></el-icon>
          <span>日志</span>
        </el-menu-item>

        <el-menu-item index="/config">
          <el-icon><Edit /></el-icon>
          <span>配置编辑</span>
        </el-menu-item>

        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <span>设置</span>
        </el-menu-item>
      </el-menu>
    </el-aside>

    <!-- 主内容区 -->
    <el-container>
      <!-- 顶栏 -->
      <el-header class="header">
        <div class="header-left">
          <el-breadcrumb separator="/">
            <el-breadcrumb-item :to="{ path: '/' }">首页</el-breadcrumb-item>
            <el-breadcrumb-item>{{ route.meta.title }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>

        <div class="header-right">
          <el-dropdown @command="handleCommand">
            <span class="user-info">
              <el-icon><User /></el-icon>
              {{ authStore.username }}
              <el-icon class="el-icon--right"><ArrowDown /></el-icon>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="change-username">修改账号</el-dropdown-item>
                <el-dropdown-item command="change-password">修改密码</el-dropdown-item>
                <el-dropdown-item divided command="logout">退出登录</el-dropdown-item>
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
  <OnDialog v-model="pwdDialogVisible" title="修改密码" width="400px" :maximizable="false" :destroy-on-close="true">
    <el-form ref="pwdFormRef" :model="pwdForm" :rules="pwdRules" label-width="80px">
      <el-form-item label="旧密码" prop="oldPassword">
        <el-input v-model="pwdForm.oldPassword" type="password" show-password placeholder="请输入旧密码" />
      </el-form-item>
      <el-form-item label="新密码" prop="newPassword">
        <el-input v-model="pwdForm.newPassword" type="password" show-password placeholder="请输入新密码" />
      </el-form-item>
      <el-form-item label="确认密码" prop="confirmPassword">
        <el-input v-model="pwdForm.confirmPassword" type="password" show-password placeholder="请再次输入新密码" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="pwdDialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="pwdLoading" @click="submitChangePassword">确认修改</el-button>
    </template>
  </OnDialog>

  <!-- 修改账号弹窗 -->
  <OnDialog v-model="userDialogVisible" title="修改账号" width="400px" :maximizable="false" :destroy-on-close="true">
    <el-form ref="userFormRef" :model="userForm" :rules="userRules" label-width="80px">
      <el-form-item label="当前账号">
        <el-input :model-value="authStore.username" disabled />
      </el-form-item>
      <el-form-item label="密码" prop="password">
        <el-input v-model="userForm.password" type="password" show-password placeholder="请输入密码验证身份" />
      </el-form-item>
      <el-form-item label="新账号" prop="newUsername">
        <el-input v-model="userForm.newUsername" placeholder="请输入新用户名" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="userDialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="userLoading" @click="submitChangeUsername">确认修改</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

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
  newPassword: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
  ],
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
      // 更新本地 token 和用户名
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

.sidebar-menu {
  border-right: none;
}

.header {
  background: white;
  border-bottom: 1px solid #e8e8e8;
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
}

.user-info {
  display: flex;
  align-items: center;
  cursor: pointer;
  color: #333;
}

.user-info .el-icon {
  margin-right: 4px;
}

.main {
  background: #f0f2f5;
  padding: 20px;
}
</style>
