<template>
  <el-form label-width="80px">
    <el-form-item :label="$t('sites.enableSsl')">
      <el-switch v-model="editForm.ssl" @change="$emit('save')" />
    </el-form-item>
    <template v-if="editForm.ssl">
      <el-form-item :label="$t('sites.certPath')">
        <el-input
          v-model="editForm.certificate_path"
          placeholder="/opt/oxnginx/ssl/fullchain.cer"
          @change="$emit('save')"
        />
      </el-form-item>
      <el-form-item :label="$t('sites.keyPath')">
        <el-input
          v-model="editForm.key_path"
          placeholder="/opt/oxnginx/ssl/private.key"
          @change="$emit('save')"
        />
      </el-form-item>
    </template>
  </el-form>
</template>

<script setup lang="ts">
// editForm 由父组件传入，直接修改（对象引用）
defineProps<{
  editForm: {
    ssl: boolean;
    certificate_path: string;
    key_path: string;
  };
}>();

defineEmits<{
  save: [];
}>();
</script>
