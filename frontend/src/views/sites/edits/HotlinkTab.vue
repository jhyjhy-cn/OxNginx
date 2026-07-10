<template>
  <el-form label-width="100px">
    <el-form-item :label="$t('sys.sites.hotlinkEnable')">
      <el-switch v-model="hotlink.enabled" @change="$emit('save')" />
    </el-form-item>
    <template v-if="hotlink.enabled">
      <el-form-item :label="$t('sys.sites.hotlinkDomains')">
        <el-input
          v-model="hotlink.domainsStr"
          type="textarea"
          :autosize="{ minRows: 3, maxRows: 6 }"
          :placeholder="$t('sys.sites.hotlinkDomainsHint')"
          @change="$emit('save')"
        />
      </el-form-item>
      <el-form-item :label="$t('sys.sites.hotlinkCode')">
        <el-select v-model="hotlink.return_code" @change="$emit('save')">
          <el-option label="403 Forbidden" :value="403" />
          <el-option label="404 Not Found" :value="404" />
        </el-select>
      </el-form-item>
    </template>
  </el-form>
</template>

<script setup lang="ts">
import type { HotlinkCfg } from '../types'

// hotlink 由父组件传入，直接修改（对象引用）
defineProps<{
  hotlink: HotlinkCfg
}>()

defineEmits<{
  save: []
}>()
</script>
