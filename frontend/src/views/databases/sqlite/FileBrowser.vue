<template>
  <div class="file-browser">
    <div class="path-bar">
      <el-button size="small" @click="goUp" :disabled="!parentDir">↑</el-button>
      <el-input v-model="currentPath" size="small" @keyup.enter="onChangePath">
        <template #append>
          <el-button size="small" @click="onChangePath">Go</el-button>
        </template>
      </el-input>
    </div>
    <el-table :data="items" v-loading="loading" @row-dblclick="onRowDblClick" height="360">
      <el-table-column :label="$t('common.name')" min-width="200">
        <template #default="{ row }">
          <el-icon v-if="row.is_dir"><Folder /></el-icon>
          <el-icon v-else><Document /></el-icon>
          <span style="margin-left: 6px">{{ row.name }}</span>
        </template>
      </el-table-column>
      <el-table-column :label="$t('common.action')" width="120" align="right">
        <template #default="{ row }">
          <el-button v-if="!row.is_dir" type="primary" size="small" @click="onPick(row)">
            {{ $t('common.confirm') }}
          </el-button>
          <el-button v-else type="primary" link size="small" @click="enter(row)">
            {{ $t('common.confirm') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>
    <div v-if="!items.length && !loading" class="empty">{{ $t('sys.files.emptyDir') }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Folder, Document } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import { listFiles, listRoots } from '@/api/files-mgr'

const props = defineProps<{ path: string }>()
const emit = defineEmits<{ 'update:path': [v: string]; pick: [path: string] }>()

const { t: _t } = useI18n()
const currentPath = ref(props.path || '')
const items = ref<Array<{ name: string; path: string; is_dir: boolean }>>([])
const loading = ref(false)

const parentDir = computed(() => {
  if (!currentPath.value) return ''
  const sep = currentPath.value.includes('\\') ? '\\' : '/'
  const idx = currentPath.value.lastIndexOf(sep)
  return idx > 0 ? currentPath.value.slice(0, idx) : ''
})

watch(
  () => props.path,
  (v) => {
    currentPath.value = v
    load()
  },
  { immediate: true }
)

async function load() {
  loading.value = true
  try {
    if (!currentPath.value) {
      // 根目录
      const roots = (await listRoots()) || []
      items.value = roots.map((p) => ({ name: p, path: p, is_dir: true }))
    } else {
      const data = (await listFiles({ path: currentPath.value } as any)) || []
      items.value = (data as any[]).map((it) => ({
        name: it.name,
        path: it.path,
        is_dir: !!it.is_dir,
      }))
    }
    emit('update:path', currentPath.value)
  } catch {
    items.value = []
  } finally {
    loading.value = false
  }
}

function onChangePath() {
  load()
}

function goUp() {
  if (parentDir.value) {
    currentPath.value = parentDir.value
    load()
  }
}

function enter(row: { is_dir: boolean; path: string }) {
  if (row.is_dir) {
    currentPath.value = row.path
    load()
  } else {
    onPick(row)
  }
}

function onRowDblClick(row: { is_dir: boolean; path: string }) {
  enter(row)
}

function onPick(row: { path: string }) {
  emit('pick', row.path)
}
</script>

<style scoped>
.file-browser {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.path-bar {
  display: flex;
  gap: 8px;
}
.empty {
  text-align: center;
  color: var(--el-text-color-secondary);
  padding: 16px 0;
}
</style>
