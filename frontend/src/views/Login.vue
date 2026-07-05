<template>
  <div class="login-page">
    <!-- 左侧内容区域 -->
    <div class="left-section">
      <div class="brand-logo">
        <div class="logo-icon">N</div>
        <span>OxNginx</span>
      </div>

      <!-- 动画人物区域 -->
      <div class="characters-area">
        <div class="characters-container" ref="charactersContainer">
          <!-- 紫色高个子 -->
          <div
            class="character purple-character"
            :style="{
              height: (isTyping || (form.password.length > 0 && !showPassword)) ? '440px' : '400px',
              transform: (form.password.length > 0 && showPassword)
                ? 'skewX(0deg)'
                : (isTyping || (form.password.length > 0 && !showPassword))
                  ? `skewX(${purpleSkew - 10}deg) translateX(40px)`
                  : `skewX(${purpleSkew}deg)`
            }"
          >
            <div
              class="eyes"
              :style="{
                left: (form.password.length > 0 && showPassword) ? '25px' : `${55 + purpleFaceX}px`,
                top: (form.password.length > 0 && showPassword) ? '40px' : `${45 + purpleFaceY}px`
              }"
            >
              <div class="eye" :class="{ blinking: isPurpleBlinking }">
                <div class="pupil" :style="getPupilStyle('purple')"></div>
              </div>
              <div class="eye" :class="{ blinking: isPurpleBlinking }">
                <div class="pupil" :style="getPupilStyle('purple')"></div>
              </div>
            </div>
          </div>

          <!-- 黑色中等个子 -->
          <div
            class="character black-character"
            :style="{
              transform: (form.password.length > 0 && showPassword)
                ? 'skewX(0deg)'
                : isLookingAtEachOther
                  ? `skewX(${blackSkew * 1.5 + 8}deg) translateX(15px)`
                  : `skewX(${blackSkew}deg)`
            }"
          >
            <div
              class="eyes"
              :style="{
                left: (form.password.length > 0 && showPassword) ? '18px' : `${32 + blackFaceX}px`,
                top: (form.password.length > 0 && showPassword) ? '35px' : `${40 + blackFaceY}px`
              }"
            >
              <div class="eye" :class="{ blinking: isBlackBlinking }">
                <div class="pupil" :style="getPupilStyle('black')"></div>
              </div>
              <div class="eye" :class="{ blinking: isBlackBlinking }">
                <div class="pupil" :style="getPupilStyle('black')"></div>
              </div>
            </div>
          </div>

          <!-- 橙色半圆 -->
          <div
            class="character orange-character"
            :style="{ transform: `skewX(${orangeSkew}deg)` }"
          >
            <div
              class="pupil-only-eyes"
              :style="{
                left: (form.password.length > 0 && showPassword) ? '60px' : `${95 + orangeFaceX}px`,
                top: (form.password.length > 0 && showPassword) ? '95px' : `${105 + orangeFaceY}px`
              }"
            >
              <div class="pupil-only" :style="getPupilOnlyStyle()"></div>
              <div class="pupil-only" :style="getPupilOnlyStyle()"></div>
            </div>
          </div>

          <!-- 黄色圆顶 -->
          <div
            class="character yellow-character"
            :style="{ transform: `skewX(${yellowSkew}deg)` }"
          >
            <div
              class="pupil-only-eyes"
              :style="{
                left: (form.password.length > 0 && showPassword) ? '22px' : `${55 + yellowFaceX}px`,
                top: (form.password.length > 0 && showPassword) ? '45px' : `${55 + yellowFaceY}px`
              }"
            >
              <div class="pupil-only" :style="getPupilOnlyStyle()"></div>
              <div class="pupil-only" :style="getPupilOnlyStyle()"></div>
            </div>
            <div
              class="mouth-line"
              :style="{
                left: (form.password.length > 0 && showPassword) ? '18px' : `${48 + yellowFaceX}px`,
                top: (form.password.length > 0 && showPassword) ? '105px' : `${110 + yellowFaceY}px`
              }"
            ></div>
          </div>
        </div>
      </div>

      <!-- 底部链接 -->
      <div class="footer-links">
        <a href="#">{{ $t('login.agreement') }}</a>
        <a href="#">{{ $t('login.privacy') }}</a>
        <a href="#">{{ $t('login.contactUs') }}</a>
        <span class="footer-icp">{{ $t('login.icp') }}</span>
      </div>

      <div class="decoration-grid"></div>
      <div class="decoration-blur decoration-blur-1"></div>
      <div class="decoration-blur decoration-blur-2"></div>
    </div>

    <!-- 右侧登录区域 -->
    <div class="right-section">
      <!-- 右上角工具栏 -->
      <div class="login-toolbar">
        <el-color-picker
          :model-value="settingsStore.themeColor"
          @change="handleColorChange"
          :predefine="presetColors"
          size="small"
        />
        <el-dropdown @command="handleLanguageChange" trigger="click">
          <OnIcon svgName="translate" :size="18" class="toolbar-icon" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="zh-CN" :class="{ active: settingsStore.locale === 'zh-CN' }">中文</el-dropdown-item>
              <el-dropdown-item command="en-US" :class="{ active: settingsStore.locale === 'en-US' }">English</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-icon class="toolbar-icon" :size="18" @click="settingsStore.toggleDarkMode($event)">
          <Moon v-if="!settingsStore.darkMode" />
          <Sunny v-else />
        </el-icon>
      </div>

      <div class="login-wrapper">
        <div class="login-header">
          <h1>{{ $t('login.title') }}</h1>
          <p>{{ $t('login.subtitle') }}</p>
        </div>

        <el-form
          ref="formRef"
          :model="form"
          :rules="rules"
          class="login-form"
          @submit.prevent="handleLogin"
        >
          <el-form-item prop="username">
            <label class="form-label">{{ $t('login.username') }}</label>
            <el-input
              v-model="form.username"
              size="large"
              auto-complete="off"
              :placeholder="$t('login.enterUsername')"
              @focus="onUsernameFocus"
              @blur="onUsernameBlur"
            />
          </el-form-item>

          <el-form-item prop="password">
            <label class="form-label">{{ $t('login.password') }}</label>
            <el-input
              v-model="form.password"
              :type="showPassword ? 'text' : 'password'"
              size="large"
              auto-complete="off"
              :placeholder="$t('login.enterPassword')"
              @keyup.enter="handleLogin"
            >
              <template #suffix>
                <el-icon class="password-toggle" @click="showPassword = !showPassword">
                  <View v-if="!showPassword" />
                  <Hide v-else />
                </el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-button
              :loading="loading"
              type="primary"
              size="large"
              class="login-button"
              @click="handleLogin"
            >
              {{ loading ? $t('login.loginBtn') + '...' : $t('login.loginBtn') }}
            </el-button>
          </el-form-item>
        </el-form>

        <div v-if="needSetup" class="setup-tip">
          <el-divider>{{ $t('login.firstUse') }}</el-divider>
          <el-button type="success" size="large" class="setup-button" @click="handleSetup">
            {{ $t('login.initAdmin') }}
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { ElMessage } from 'element-plus'
import { View, Hide, Moon, Sunny } from '@element-plus/icons-vue'
import { useSettingsStore } from '@/stores/settings'
import api from '@/api'

const { t, locale } = useI18n()
const router = useRouter()
const authStore = useAuthStore()
const settingsStore = useSettingsStore()

const presetColors = ['#409EFF', '#536dfe', '#9c27b0', '#00bfa5', '#ff5722', '#e91e63']

function handleColorChange(color: string | null) {
  if (color) settingsStore.setThemeColor(color)
}

function handleLanguageChange(lang: 'zh-CN' | 'en-US') {
  settingsStore.setLocale(lang)
  locale.value = lang
}

const formRef = ref()
const loading = ref(false)
const needSetup = ref(false)
const showPassword = ref(false)

const form = reactive({
  username: '',
  password: '',
})

const rules = {
  username: [{ required: true, message: () => t('login.enterUsername'), trigger: 'blur' }],
  password: [{ required: true, message: () => t('login.enterPassword'), trigger: 'blur' }],
}

// 动画状态
const charactersContainer = ref<HTMLElement | null>(null)
const isTyping = ref(false)
const isLookingAtEachOther = ref(false)
const isPurpleBlinking = ref(false)
const isBlackBlinking = ref(false)

const mouseX = ref(0)
const mouseY = ref(0)

const purpleSkew = ref(0)
const purpleFaceX = ref(0)
const purpleFaceY = ref(0)
const blackSkew = ref(0)
const blackFaceX = ref(0)
const blackFaceY = ref(0)
const orangeSkew = ref(0)
const orangeFaceX = ref(0)
const orangeFaceY = ref(0)
const yellowSkew = ref(0)
const yellowFaceX = ref(0)
const yellowFaceY = ref(0)

const pupilOffsets = ref({ purple: { x: 0, y: 0 }, black: { x: 0, y: 0 }, orange: { x: 0, y: 0 }, yellow: { x: 0, y: 0 } })

let blinkTimeout: ReturnType<typeof setTimeout> | null = null

function getPupilStyle(character: string) {
  const offset = pupilOffsets.value[character as keyof typeof pupilOffsets.value]
  let x = offset.x, y = offset.y
  if (form.password.length > 0 && showPassword.value) { x = -3; y = -3 }
  else if (isLookingAtEachOther.value) { x = 2; y = 3 }
  return { transform: `translate(${x}px, ${y}px)` }
}

function getPupilOnlyStyle() {
  const offset = pupilOffsets.value.orange
  let x = offset.x, y = offset.y
  if (form.password.length > 0 && showPassword.value) { x = -4; y = -3 }
  return { transform: `translate(${x}px, ${y}px)` }
}

function calculatePositions() {
  if (!charactersContainer.value) return
  const rect = charactersContainer.value.getBoundingClientRect()
  const centerX = rect.left + rect.width / 2
  const centerY = rect.top + rect.height / 3
  const deltaX = mouseX.value - centerX
  const deltaY = mouseY.value - centerY

  purpleFaceX.value = Math.max(-12, Math.min(12, deltaX / 25))
  purpleFaceY.value = Math.max(-8, Math.min(8, deltaY / 35))
  purpleSkew.value = Math.max(-5, Math.min(5, -deltaX / 100))
  blackFaceX.value = Math.max(-10, Math.min(10, deltaX / 25))
  blackFaceY.value = Math.max(-6, Math.min(6, deltaY / 35))
  blackSkew.value = Math.max(-5, Math.min(5, -deltaX / 100))
  orangeFaceX.value = Math.max(-10, Math.min(10, deltaX / 25))
  orangeFaceY.value = Math.max(-6, Math.min(6, deltaY / 35))
  orangeSkew.value = Math.max(-5, Math.min(5, -deltaX / 100))
  yellowFaceX.value = Math.max(-10, Math.min(10, deltaX / 25))
  yellowFaceY.value = Math.max(-6, Math.min(6, deltaY / 35))
  yellowSkew.value = Math.max(-5, Math.min(5, -deltaX / 100))

  const pupilX = Math.max(-3, Math.min(3, deltaX / 80))
  const pupilY = Math.max(-3, Math.min(3, deltaY / 80))
  pupilOffsets.value = {
    purple: { x: pupilX, y: pupilY },
    black: { x: pupilX, y: pupilY },
    orange: { x: pupilX, y: pupilY },
    yellow: { x: pupilX, y: pupilY },
  }
}

function handleMouseMove(e: MouseEvent) {
  mouseX.value = e.clientX
  mouseY.value = e.clientY
  calculatePositions()
}

function schedulePurpleBlink() {
  const delay = Math.random() * 4000 + 3000
  blinkTimeout = setTimeout(() => {
    isPurpleBlinking.value = true
    setTimeout(() => { isPurpleBlinking.value = false; schedulePurpleBlink() }, 150)
  }, delay)
}

function scheduleBlackBlink() {
  const delay = Math.random() * 4000 + 3000
  setTimeout(() => {
    isBlackBlinking.value = true
    setTimeout(() => { isBlackBlinking.value = false; scheduleBlackBlink() }, 150)
  }, delay)
}

function onUsernameFocus() {
  isTyping.value = true
  isLookingAtEachOther.value = true
  setTimeout(() => { isLookingAtEachOther.value = false }, 800)
}

function onUsernameBlur() {
  isTyping.value = false
}

// 业务逻辑
async function checkSetup() {
  try {
    const response = await api.get('/api/setup/status')
    if (response.data.code === 0 && response.data.data?.need_setup) {
      needSetup.value = true
    }
  } catch {
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

onMounted(() => {
  checkSetup()
  document.addEventListener('mousemove', handleMouseMove)
  schedulePurpleBlink()
  scheduleBlackBlink()
  calculatePositions()
})

onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove)
  if (blinkTimeout) clearTimeout(blinkTimeout)
})
</script>

<style scoped>
.login-page {
  min-height: 100vh;
  display: grid;
  grid-template-columns: 1fr 1fr;
}

@media (max-width: 1024px) {
  .login-page {
    grid-template-columns: 1fr;
  }
}

.left-section {
  position: relative;
  display: none;
  flex-direction: column;
  justify-content: space-between;
  padding: 48px;
  background: linear-gradient(135deg, #1a1a2e 0%, color-mix(in srgb, #1a1a2e 70%, var(--el-color-primary) 30%) 100%);
  color: white;
  overflow: hidden;
}

@media (min-width: 1024px) {
  .left-section {
    display: flex;
  }
}

.brand-logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  position: relative;
  z-index: 20;
}

.logo-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
}

.characters-area {
  flex: 1;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  position: relative;
  z-index: 20;
  padding-bottom: 40px;
}

.characters-container {
  position: relative;
  width: 550px;
  height: 420px;
}

.character {
  position: absolute;
  bottom: 0;
  transition: all 0.7s ease-in-out;
  transform-origin: bottom center;
}

.purple-character {
  left: 60px;
  width: 200px;
  height: 400px;
  background-color: #6C3FF5;
  border-radius: 14px 14px 0 0;
  z-index: 1;
}

.purple-character .eyes {
  position: absolute;
  display: flex;
  gap: 28px;
  transition: all 0.7s ease-in-out;
}

.purple-character .eye {
  width: 26px;
  height: 26px;
  background: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: height 0.15s ease-in-out;
}

.purple-character .eye.blinking {
  height: 3px;
}

.purple-character .pupil {
  width: 10px;
  height: 10px;
  background: #2D2D2D;
  border-radius: 50%;
  transition: transform 0.1s ease-out;
}

.black-character {
  left: 250px;
  width: 130px;
  height: 310px;
  background-color: #2D2D2D;
  border-radius: 10px 10px 0 0;
  z-index: 2;
}

.black-character .eyes {
  position: absolute;
  display: flex;
  gap: 22px;
  transition: all 0.7s ease-in-out;
}

.black-character .eye {
  width: 22px;
  height: 22px;
  background: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: height 0.15s ease-in-out;
}

.black-character .eye.blinking {
  height: 3px;
}

.black-character .pupil {
  width: 9px;
  height: 9px;
  background: #2D2D2D;
  border-radius: 50%;
  transition: transform 0.1s ease-out;
}

.orange-character {
  left: 0;
  width: 260px;
  height: 220px;
  background-color: #FF9B6B;
  border-radius: 130px 130px 0 0;
  z-index: 3;
}

.orange-character .pupil-only-eyes {
  position: absolute;
  display: flex;
  gap: 30px;
  transition: all 0.2s ease-out;
}

.orange-character .pupil-only {
  width: 18px;
  height: 18px;
  background: #2D2D2D;
  border-radius: 50%;
  transition: transform 0.1s ease-out;
}

.yellow-character {
  right: 30px;
  width: 150px;
  height: 250px;
  background-color: #E8D754;
  border-radius: 75px 75px 0 0;
  z-index: 4;
}

.yellow-character .pupil-only-eyes {
  position: absolute;
  display: flex;
  gap: 22px;
  transition: all 0.2s ease-out;
}

.yellow-character .pupil-only {
  width: 18px;
  height: 18px;
  background: #2D2D2D;
  border-radius: 50%;
  transition: transform 0.1s ease-out;
}

.yellow-character .mouth-line {
  position: absolute;
  width: 70px;
  height: 5px;
  background: #2D2D2D;
  border-radius: 3px;
  transition: all 0.2s ease-out;
}

.decoration-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(255, 255, 255, 0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.05) 1px, transparent 1px);
  background-size: 20px 20px;
}

.decoration-blur {
  position: absolute;
  border-radius: 50%;
  filter: blur(60px);
}

.decoration-blur-1 {
  top: 25%;
  right: 25%;
  width: 256px;
  height: 256px;
  background: rgba(255, 255, 255, 0.1);
}

.decoration-blur-2 {
  bottom: 25%;
  left: 25%;
  width: 384px;
  height: 384px;
  background: rgba(255, 255, 255, 0.05);
}

.right-section {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  background: #fafafa;
}

@media (min-width: 1024px) {
  .right-section {
    padding: 32px 48px;
  }
}

.login-wrapper {
  width: 100%;
  max-width: 400px;
}

.login-header {
  text-align: center;
  margin-bottom: 40px;
}

.login-header h1 {
  font-size: 30px;
  font-weight: 700;
  color: #1a1a2e;
  margin: 0 0 8px;
}

.login-header p {
  font-size: 14px;
  color: #6b7280;
  margin: 0;
}

.login-form .form-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
  margin-bottom: 8px;
}

.login-form :deep(.el-form-item) {
  margin-bottom: 20px;
}

.login-form :deep(.el-input__wrapper) {
  height: 48px;
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(0, 0, 0, 0.1);
  padding: 0 16px;
  transition: all 0.3s ease;
}

.login-form :deep(.el-input__wrapper:hover) {
  border-color: var(--el-color-primary);
}

.login-form :deep(.el-input__wrapper.is-focus) {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--el-color-primary) 10%, transparent);
}

.login-form :deep(.el-input__inner) {
  height: 48px;
  font-size: 15px;
}

.login-form :deep(.el-input__inner::placeholder) {
  color: #9ca3af;
}

.password-toggle {
  cursor: pointer;
  color: #9ca3af;
  transition: color 0.3s ease;
}

.password-toggle:hover {
  color: var(--el-color-primary);
}

.login-button {
  width: 100%;
  height: 48px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  border: none;
  transition: all 0.3s ease;
}

.setup-tip {
  text-align: center;
  margin-top: 16px;
}

.setup-button {
  width: 100%;
  border-radius: 8px;
}

/* 底部链接 */
.footer-links {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.6);
  position: relative;
  z-index: 20;
}

.footer-links a {
  color: rgba(255, 255, 255, 0.6);
  text-decoration: none;
  transition: color 0.3s;
}

.footer-links a:hover {
  color: rgba(255, 255, 255, 1);
}

.footer-icp {
  color: rgba(255, 255, 255, 0.4);
}

/* 右上角工具栏 */
.login-toolbar {
  position: absolute;
  top: 20px;
  right: 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 10;
}

.toolbar-icon {
  cursor: pointer;
  color: var(--el-text-color-regular);
  transition: color 0.3s;
}

.toolbar-icon:hover {
  color: var(--el-color-primary);
}

:deep(.el-dropdown-menu__item.active) {
  color: var(--el-color-primary);
  font-weight: 600;
}

/* ===== 暗黑模式 ===== */
:global(.dark) .login-page {
  background: #141414;
}

.right-section {
  transition: background 0.3s;
}

:global(.dark) .right-section {
  background: #1a1a1a;
}

:global(.dark) .login-header h1 {
  color: #e5eaf3;
}

:global(.dark) .login-header p {
  color: #a3a6ad;
}

:global(.dark) .login-form .form-label {
  color: #cfd3dc;
}

:global(.dark) .login-form :deep(.el-input__wrapper) {
  background: #262727;
  border-color: #4c4d4f;
  box-shadow: none;
}

:global(.dark) .login-form :deep(.el-input__inner) {
  color: #e5eaf3;
}

:global(.dark) .login-form :deep(.el-input__inner::placeholder) {
  color: #6b6f76;
}
</style>
