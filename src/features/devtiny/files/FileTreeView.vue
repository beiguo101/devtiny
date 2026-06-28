<template>
  <div class="tree-list">
    <FileTreeNode
      v-for="node in nodes"
      :key="node.relativePath"
      :node="node"
      :selected-path="selectedPath"
      @select="$emit('select', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import type { FileTreeNode as FileTreeNodeType } from '../../../core/filesystem/types'
import FileTreeNode from './FileTreeNode.vue'

defineProps<{
  nodes: FileTreeNodeType[]
  selectedPath: string
}>()

defineEmits<{
  select: [node: FileTreeNodeType]
}>()
</script>
