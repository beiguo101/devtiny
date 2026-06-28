<template>
  <div class="changes-panel">
    <div v-if="!changes.length" class="empty-inline">{{ t('noChanges') }}</div>

    <label v-for="change in changes" :key="change.relativePath" class="change-row">
      <input v-model="selectedPaths" type="checkbox" :value="change.relativePath" @change="emitSelection" />
      <button class="change-button" type="button" :title="change.relativePath" @click="$emit('select', change)">
        <span class="status-dot" :class="change.status">{{ statusLabel(change.status) }}</span>
        <span>{{ fileName(change.relativePath) }}</span>
      </button>
    </label>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { statusLabel, t } from '../../../app/i18n'
import type { GitFileSelection } from '../../../core/commands/types'
import type { GitChangeFile } from './types'

const props = defineProps<{
  changes: GitChangeFile[]
  selectedPaths: string[]
}>()

const emit = defineEmits<{
  select: [change: GitChangeFile]
  selectionChange: [files: GitFileSelection[]]
}>()

const selectedPaths = ref<string[]>([])

const selected = computed(() =>
  props.changes
    .filter((change) => selectedPaths.value.includes(change.relativePath))
    .map((change) => ({
      relativePath: change.relativePath,
      status: change.status,
      indexStatus: change.indexStatus,
      worktreeStatus: change.worktreeStatus
    }))
)

watch(
  () => props.changes,
  () => {
    selectedPaths.value = selectedPaths.value.filter((path) =>
      props.changes.some((change) => change.relativePath === path)
    )
    emitSelection()
  }
)

watch(
  () => props.selectedPaths,
  (paths) => {
    selectedPaths.value = paths.filter((path) =>
      props.changes.some((change) => change.relativePath === path)
    )
  },
  { immediate: true }
)

function emitSelection() {
  emit('selectionChange', selected.value)
}

function fileName(relativePath: string) {
  return relativePath.split('/').filter(Boolean).pop() || relativePath
}
</script>
