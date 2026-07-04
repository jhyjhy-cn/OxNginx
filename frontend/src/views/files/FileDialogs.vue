<template>
  <!-- 新建文件/文件夹 -->
  <OnDialog v-model="createDialog.visible" :title="createDialog.isDir ? t('files.createFolder') : t('files.createFile')" width="420px">
    <el-form label-width="80px">
      <el-form-item :label="createDialog.isDir ? t('files.folderName') : t('files.fileName')">
        <el-input v-model="createDialog.name" :placeholder="createDialog.isDir ? t('files.enterFolderName') : t('files.enterFileName')" @keyup.enter="$emit('submit-create')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="createDialog.visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="$emit('submit-create')">{{ t('common.confirm') }}</el-button>
    </template>
  </OnDialog>

  <!-- 重命名 -->
  <OnDialog v-model="renameDialog.visible" :title="t('files.rename')" width="420px">
    <el-form label-width="80px">
      <el-form-item :label="t('files.name')">
        <el-input v-model="renameDialog.newName" :placeholder="t('files.enterNewName')" @keyup.enter="$emit('submit-rename')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="renameDialog.visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="$emit('submit-rename')">{{ t('common.confirm') }}</el-button>
    </template>
  </OnDialog>

  <!-- 复制/移动 -->
  <OnDialog v-model="moveDialog.visible" :title="moveDialog.isCopy ? t('files.copyTo') : t('files.moveTo')" width="500px">
    <el-form label-width="80px">
      <el-form-item :label="t('files.destination')">
        <el-input v-model="moveDialog.destination" :placeholder="t('files.enterDestination')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="moveDialog.visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="$emit('submit-move')">{{ t('common.confirm') }}</el-button>
    </template>
  </OnDialog>

  <!-- 压缩 -->
  <OnDialog v-model="compressDialog.visible" :title="t('files.compress')" width="460px">
    <el-form label-width="80px">
      <el-form-item :label="t('files.compressName')">
        <el-input v-model="compressDialog.name" :placeholder="t('files.enterCompressName')" />
      </el-form-item>
      <el-form-item :label="t('files.compressFormat')">
        <el-radio-group v-model="compressDialog.format">
          <el-radio value="zip">.zip</el-radio>
          <el-radio value="tar.gz">.tar.gz</el-radio>
        </el-radio-group>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="compressDialog.visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="$emit('submit-compress')">{{ t('common.confirm') }}</el-button>
    </template>
  </OnDialog>

  <!-- 权限 -->
  <OnDialog v-model="chmodDialog.visible" :title="t('files.changePermission')" width="400px">
    <el-form label-width="80px">
      <el-form-item :label="t('files.permissionMode')">
        <el-input v-model="chmodDialog.mode" :placeholder="t('files.enterPermission')" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="chmodDialog.visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="$emit('submit-chmod')">{{ t('common.confirm') }}</el-button>
    </template>
  </OnDialog>

  <!-- 编辑文件 -->
  <OnDialog v-model="editDialog.visible" :title="t('files.editFile')" width="80%" maximizable>
    <div class="editor-wrapper">
      <textarea v-model="editDialog.content" class="file-editor" spellcheck="false" />
    </div>
    <template #footer>
      <el-button @click="editDialog.visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" @click="$emit('submit-edit')">{{ t('files.saveFile') }}</el-button>
    </template>
  </OnDialog>

  <!-- 属性 -->
  <OnDialog v-model="propDialog.visible" title="属性" width="480px">
    <el-descriptions :column="1" border size="small">
      <el-descriptions-item label="名称">{{ propDialog.item?.name }}</el-descriptions-item>
      <el-descriptions-item label="类型">{{ propDialog.item?.is_dir ? '文件夹' : '文件' }}{{ propDialog.item?.extension ? ' (.' + propDialog.item.extension + ')' : '' }}</el-descriptions-item>
      <el-descriptions-item label="路径">{{ propDialog.item?.path }}</el-descriptions-item>
      <el-descriptions-item label="大小">
        <template v-if="propDialog.item?.is_dir">
          <span v-if="propDialog.item?._size !== undefined">{{ formatSize(propDialog.item._size) }}</span>
          <el-button v-else link type="primary" size="small" :loading="propDialog.item?._calcLoading" @click="propDialog.item && $emit('calc-size', propDialog.item)">计算</el-button>
        </template>
        <template v-else>{{ formatSize(propDialog.item?.size || 0) }}</template>
      </el-descriptions-item>
      <el-descriptions-item label="权限/所有者">{{ propDialog.item?.permissions || '-' }}{{ propDialog.item?.owner ? ' / ' + propDialog.item.owner : '' }}</el-descriptions-item>
      <el-descriptions-item label="修改时间">{{ propDialog.item?.modified || '-' }}</el-descriptions-item>
    </el-descriptions>
    <template #footer>
      <el-button type="primary" @click="propDialog.visible = false">确定</el-button>
    </template>
  </OnDialog>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import OnDialog from '@/components/OnDialog/index.vue'
import type { FileItem } from './useFileManager'

defineProps<{
  createDialog: { visible: boolean; isDir: boolean; name: string }
  renameDialog: { visible: boolean; path: string; newName: string }
  moveDialog: { visible: boolean; source: string; destination: string; isCopy: boolean }
  compressDialog: { visible: boolean; paths: string[]; name: string; format: string }
  chmodDialog: { visible: boolean; path: string; mode: string }
  editDialog: { visible: boolean; path: string; content: string }
  propDialog: { visible: boolean; item: FileItem | null }
  formatSize: (bytes: number) => string
}>()

defineEmits<{
  'submit-create': []
  'submit-rename': []
  'submit-move': []
  'submit-compress': []
  'submit-chmod': []
  'submit-edit': []
  'calc-size': [row: FileItem]
}>()

const { t } = useI18n()
</script>

<style scoped>
.editor-wrapper { height: 60vh; }
.file-editor {
  width: 100%; height: 100%;
  border: 1px solid var(--el-border-color); border-radius: 4px; padding: 12px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace; font-size: 13px; line-height: 1.6;
  resize: none; outline: none;
  background: var(--el-bg-color); color: var(--el-text-color-primary); tab-size: 4;
}
.file-editor:focus { border-color: var(--el-color-primary); }
</style>
