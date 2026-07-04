<template>
  <div class="login-container">
    <el-card class="login-card">
      <template #header>
        <div class="card-header">
          <h2>OxNginx Manager</h2>
          <p>{{ $t('login.subtitle') }}</p>
        </div>
      </template>

      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="0"
        @submit.prevent="handleLogin"
      >
        <el-form-item prop="username">
          <el-input
            v-model="form.username"
            :placeholder="$t('login.username')"
            prefix-icon="User"
            size="large"
          />
        </el-form-item>

        <el-form-item prop="password">
          <el-input
            v-model="form.password"
            type="password"
            :placeholder="$t('login.password')"
            prefix-icon="Lock"
            size="large"
            show-password
            @keyup.enter="handleLogin"
          />
        </el-form-item>

        <el-form-item>
          <el-button
            type="primary"
            size="large"
            :loading="loading"
            style="width: 100%"
            @click="handleLogin"
          >
            {{ $t('login.loginBtn') }}
          </el-button>
        </el-form-item>
      </el-form>

      <div v-if="needSetup" class="setup-tip">
        <el-divider>{{ $t('login.firstUse') }}</el-divider>
        <el-button type="success" @click="handleSetup">
          {{ $t('login.initAdmin') }}
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { ElMessage } from 'element-plus'
import api from '@/api'

const { t } = useI18n()
const router = useRouter()
const authStore = useAuthStore()

const formRef = ref()
const loading = ref(false)
const needSetup = ref(false)

const form = reactive({
  username: '',
  password: '',
})

const rules = {
  username: [{ required: true, message: () => t('login.enterUsername'), trigger: 'blur' }],
  password: [{ required: true, message: () => t('login.enterPassword'), trigger: 'blur' }],
}

onMounted(() => {
  // 检查是否需要初始化
  checkSetup()
})

async function checkSetup() {
  try {
    const response = await api.get('/api/setup/status')
    if (response.data.code === 0 && response.data.data?.need_setup) {
      needSetup.value = true
    }
  } catch {
    // 接口异常时不显示初始化按钮，避免误判
    needSetup.value = false
  }
}

async function handleLogin() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  loading.value = true
  try {
    await authStore.login(form.username, form.password)
    ElMessage.success(t('login.loginSuccess'))
    router.push('/')
  } catch (error: any) {
    ElMessage.error(error.message || t('login.loginFailed'))
  } finally {
    loading.value = false
  }
}

async function handleSetup() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return

  loading.value = true
  try {
    const response = await api.post('/api/setup', {
      username: form.username,
      password: form.password,
    })
    if (response.data.code === 0) {
      ElMessage.success(t('login.initSuccess'))
      needSetup.value = false
    } else {
      ElMessage.error(response.data.message)
    }
  } catch (error: any) {
    ElMessage.error(error.message || t('login.initFailed'))
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.login-card {
  width: 400px;
}

.card-header {
  text-align: center;
}

.card-header h2 {
  margin: 0;
  color: #303133;
}

.card-header p {
  margin: 8px 0 0;
  color: #909399;
  font-size: 14px;
}

.setup-tip {
  text-align: center;
  margin-top: 16px;
}
</style>
