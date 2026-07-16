<template>
  <div class="dt-diff" :aria-label="`${file.name} 的变化内容`">
    <div v-if="file.binary || file.previewUnavailable" class="dt-diff-unavailable">
      <IconPhotoOff />
      <strong>无法预览文本变化</strong>
      <p>{{ file.previewUnavailable || '该文件内容暂时无法预览。' }}</p>
      <span>{{ file.name }} · {{ file.size }}</span>
    </div>
    <div v-else-if="!file.diff.length" class="dt-diff-unavailable">
      <IconFileOff />
      <strong>没有可显示的 Diff</strong>
      <p>该文件当前没有文本行变化。</p>
    </div>
    <div v-else class="dt-diff-scroll" role="table">
      <div class="dt-diff-file-head" role="row">
        <span>{{ file.oldPath ? `${file.oldPath} → ` : '' }}{{ file.path }}</span>
        <span><b>+{{ file.additions }}</b> / <em>-{{ file.deletions }}</em></span>
      </div>
      <div
        v-for="(line, index) in file.diff"
        :key="`${index}-${line.oldLine}-${line.newLine}`"
        class="dt-diff-line"
        :class="`is-${line.kind}`"
        role="row"
      >
        <span class="dt-line-number">{{ line.kind === 'hunk' ? '' : line.oldLine ?? '' }}</span>
        <span class="dt-line-number">{{ line.kind === 'hunk' ? '' : line.newLine ?? '' }}</span>
        <span class="dt-line-marker">{{ marker(line.kind) }}</span>
        <code>{{ line.content }}</code>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { IconFileOff, IconPhotoOff } from '@tabler/icons-vue'
import type { DiffLineKind, MockProjectFile } from '../mock/types'

defineProps<{ file: MockProjectFile }>()

function marker(kind: DiffLineKind) {
  if (kind === 'added') return '+'
  if (kind === 'deleted') return '−'
  return ''
}
</script>
