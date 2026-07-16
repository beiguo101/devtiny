<template>
  <div class="dt-tree-branch">
    <button
      class="dt-tree-row"
      :class="{ selected: entry.kind === 'file' && selectedPath === entry.path }"
      type="button"
      @click="activate"
    >
      <IconChevronDown v-if="entry.kind === 'directory' && expanded" />
      <IconChevronRight v-else-if="entry.kind === 'directory'" />
      <IconFileCode v-else />
      <span>{{ entry.name }}</span>
      <span v-if="entry.file?.status !== 'clean'" class="dt-tree-status" :class="`is-${entry.file?.status}`">
        {{ statusLetter(entry.file?.status) }}
      </span>
    </button>
    <div v-if="entry.kind === 'directory' && expanded" class="dt-tree-children">
      <MockFileTreeNode
        v-for="child in entry.children"
        :key="child.id"
        :entry="child"
        :selected-path="selectedPath"
        @select="$emit('select', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { IconChevronDown, IconChevronRight, IconFileCode } from '@tabler/icons-vue'
import type { ChangeStatus, FileTreeEntry } from '../mock/types'

const props = defineProps<{ entry: FileTreeEntry; selectedPath: string }>()
const emit = defineEmits<{ select: [path: string] }>()
const expanded = ref(props.entry.path === 'src' || props.entry.path === 'src/components')

function activate() {
  if (props.entry.kind === 'directory') expanded.value = !expanded.value
  else emit('select', props.entry.path)
}

function statusLetter(status?: ChangeStatus) {
  return status ? ({ modified: 'M', added: 'A', deleted: 'D', renamed: 'R', clean: '' })[status] : ''
}
</script>
