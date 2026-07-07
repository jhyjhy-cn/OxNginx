# components 目录规范

本目录统一存放**项目级可复用业务组件**。Element Plus（`el-*`）与第三方组件不在此规范范围内。

## 目录结构

每个组件**必须是一个目录**，目录名 = 组件名（PascalCase），入口文件固定为 `index.vue`：

```
components/
├── OnDialog/         # 弹窗组件
│   └── index.vue
├── OnIcon/           # 图标组件 (EP / SVG)
│   └── index.vue
└── HasPermission/    # 权限按钮组件
    └── index.vue
```

**禁止**写成单文件：`components/Foo.vue` ❌ → `components/Foo/index.vue` ✅

这样做的原因：
- 组件附带子文件（README、demo、测试）时不会污染外层目录
- 引用路径统一为 `@/components/<Name>/index.vue`，无论是否带子文件都一致
- 配合 IDE 跳转 / 重构更稳定

## 引用方式

```ts
// 一律 import 默认导出，路径写全
import OnDialog from '@/components/OnDialog/index.vue'
import HasPermission from '@/components/HasPermission/index.vue'
```

模板中：

```vue
<OnDialog v-model="visible" title="标题">
  <p>内容</p>
  <template #footer>
    <el-button @click="visible = false">取消</el-button>
  </template>
</OnDialog>

<HasPermission code="sys:site:add">
  <el-button>新增站点</el-button>
</HasPermission>
```

## 命名约定

| 类型 | 命名 | 前缀 | 示例 |
|------|------|------|------|
| 弹窗 | PascalCase | `On` | OnDialog, OnDrawer, OnPopover |
| 图标 | PascalCase | `On` | OnIcon, OnEmoji |
| 权限 / 角色 | PascalCase | — | HasPermission, HasRole |
| 业务封装 | PascalCase | 业务域 | FileTree, CertPicker |

- `On*` 前缀用于**包装 Element Plus / 第三方 UI**的统一封装（弹窗、抽屉、Popover、Icon 等）
- `Has*` 前缀用于**布尔权限 / 能力判断**类组件
- 业务封装直接用业务名

## 内部规范

1. **入口文件**：`index.vue`，`<script setup lang="ts">`
2. **Props / Emits**：用 `withDefaults(defineProps<{...}>(), {...})` + `defineEmits<{...}>()` 的 TS 风格
3. **样式**：`<style scoped>`，避免污染全局；必须全局覆盖时另起 `<style>` 不带 scoped 并加注释说明
4. **i18n**：所有用户可见文案用 `{{ $t('xxx') }}` 或 `t('xxx')`，不写死中文
5. **依赖**：仅依赖 `vue` / `element-plus` / `@/stores/*` / `@/utils/*`，不引入具体业务模块

## 新增组件 checklist

- [ ] 目录名 = 组件名（PascalCase）
- [ ] 入口 `index.vue` 用 `<script setup lang="ts">`
- [ ] Props / Emits 写 TS 类型
- [ ] 引用方式：`@/components/<Name>/index.vue`
- [ ] 文案走 i18n
- [ ] 必要时补本目录 README

## 现有组件速查

| 组件 | 用途 | Props |
|------|------|-------|
| `OnDialog` | 增强版 el-dialog (拖拽/最大化/主题色) | `modelValue, title, width, height, maximizable, closeOnClickModal, destroyOnClose` |
| `OnIcon` | 统一图标 (Element Plus / SVG / Sprite) | `name, svgName, svg, size, color, className, style, prefix` |
| `HasPermission` | 权限码包装 slot | `code` |