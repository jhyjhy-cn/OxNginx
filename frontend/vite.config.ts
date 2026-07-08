import { defineConfig } from 'vite'
import { resolve } from 'path'
import { fileURLToPath } from 'url'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import tailwindcss from '@tailwindcss/vite'
import vueDevTools from 'vite-plugin-vue-devtools'
import svgLoader from 'vite-svg-loader'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

// vite 8 完全 ESM，__dirname 不再可用 —— ponytail: 用 fileURLToPath 推导
const __dirname = fileURLToPath(new URL('./', import.meta.url))

export default defineConfig({
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
    extensions: ['.mjs', '.js', '.ts', '.jsx', '.tsx', '.json', '.vue'],
  },
  css: {
    // element-plus 2.13 需要 dart-sass modern API —— ponytail: 否则 build 报 deprecation
    preprocessorOptions: {
      scss: { api: 'modern-compiler' as const },
    },
  },
  plugins: [
    vue(),
    vueDevTools(),
    vueJsx(),
    tailwindcss(),
    svgLoader({ defaultImport: 'component' }),
    AutoImport({ resolvers: [ElementPlusResolver()] }),
    Components({ resolvers: [ElementPlusResolver()] }),
  ],
  optimizeDeps: {
    // ponytail: monaco 顶层 entry，走预构建避免首次 import dev 卡顿
    include: ['monaco-editor/esm/vs/editor/editor.api'],
  },
  server: {
    open: true,
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:9000',
        changeOrigin: true,
        ws: true,
      },
    },
  },
  build: {
    outDir: '../backend/target/debug/static',
    emptyOutDir: true,
    target: 'baseline-widely-available',
    chunkSizeWarningLimit: 4000,
    rollupOptions: {
      output: {
        // ponytail: rolldown 8 仅接受 manualChunks 函数形式；按模块 id 分到 monaco/element-plus/vendor
        manualChunks(id: string) {
          if (id.includes('node_modules/monaco-editor')) return 'monaco-editor'
          if (
            id.includes('node_modules/element-plus') ||
            id.includes('node_modules/@element-plus')
          ) {
            return 'element-plus'
          }
          if (
            id.includes('node_modules/vue') ||
            id.includes('node_modules/vue-router') ||
            id.includes('node_modules/pinia') ||
            id.includes('node_modules/axios')
          ) {
            return 'vendor'
          }
        },
      },
    },
  },
})
