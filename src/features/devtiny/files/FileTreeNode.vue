<template>
  <div>
    <button
      class="tree-node"
      :class="{ selected: selectedPath === node.relativePath }"
      type="button"
      @click="handleClick"
    >
      <span class="tree-icon">
        <IconChevronDown v-if="node.kind === 'directory' && expanded" />
        <IconChevronRight v-else-if="node.kind === 'directory'" />
        <component :is="fileIcon" v-else />
      </span>
      <span class="tree-name">{{ node.name }}</span>
    </button>
    <div v-if="expanded && node.children.length" class="tree-children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.relativePath"
        :node="child"
        :selected-path="selectedPath"
        @select="$emit('select', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  IconBrandDocker,
  IconBrandPython,
  IconChevronDown,
  IconChevronRight,
  IconFile,
  IconFileCode,
  IconFileDescription,
  IconSettings
} from '@tabler/icons-vue'
import type { FileTreeNode } from '../../../core/filesystem/types'

const props = defineProps<{
  node: FileTreeNode
  selectedPath: string
}>()

const emit = defineEmits<{
  select: [node: FileTreeNode]
}>()

const expanded = ref(false)

const fileIcon = computed(() => {
  const name = props.node.name.toLowerCase()
  if (name === 'dockerfile' || name.includes('docker-compose') || name.endsWith('.yml') || name.endsWith('.yaml')) {
    return IconBrandDocker
  }
  if (name.endsWith('.py')) return IconBrandPython
  if (name.endsWith('.md') || name.endsWith('.txt')) return IconFileDescription
  if (name.startsWith('.') || name.endsWith('.json') || name.endsWith('.toml')) return IconSettings
  if (name.endsWith('.js') || name.endsWith('.ts') || name.endsWith('.html') || name.endsWith('.css')) {
    return IconFileCode
  }
  return IconFile
})

function handleClick() {
  if (props.node.kind === 'directory') {
    expanded.value = !expanded.value
  }
  emit('select', props.node)
}
</script>
