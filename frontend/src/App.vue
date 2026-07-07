<template>
  <el-config-provider :locale="epLocale">
    <router-view />
  </el-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import en from 'element-plus/es/locale/lang/en'

const { locale } = useI18n()

const epLocale = computed(() => locale.value === 'zh-CN' ? zhCn : en)
</script>

<style>
@font-face {
  font-family: 'Maple Mono NF CN';
  src: url('/fonts/MapleMono-NF-CN-Regular.ttf') format('truetype');
  font-weight: normal;
  font-style: normal;
  font-display: swap;
}

:root {
  --el-bg-color-page: #f3f5f8;
  --font-mono: 'Maple Mono NF CN', Consolas, 'Courier New', monospace;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

code, pre, .monaco-editor, .xterm {
  font-family: var(--font-mono) !important;
}
</style>
