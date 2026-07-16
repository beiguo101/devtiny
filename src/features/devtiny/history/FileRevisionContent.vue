<template>
  <section class="dt-revision-content">
    <header>
      <div>
        <button type="button" class="dt-back-current" @click="$emit('close')"><IconArrowLeft />返回当前内容</button>
        <span class="dt-eyebrow">该提交的具体差异 · {{ entry.shortCommit }}</span>
        <h2>{{ entry.subject }}</h2>
        <p>{{ relativePath }} · {{ formatDate(entry.date) }}</p>
      </div>
      <div class="dt-revision-actions">
        <button class="dt-button dt-button-secondary" type="button" :disabled="busy" @click="saveRevision"><IconDownload />另存为</button>
        <button class="dt-button dt-button-danger" type="button" :disabled="busy" @click="restoreRevision"><IconHistoryToggle />恢复此记录</button>
      </div>
    </header>
    <div v-if="error" class="dt-history-message is-error">{{ error }}</div>
    <div v-if="message" class="dt-history-message">{{ message }}</div>
    <div v-if="loading" class="dt-loading-state dt-revision-loading"><span class="dt-loading-mark"></span><strong>正在读取历史内容…</strong></div>
    <div v-else-if="content?.isBinary" class="dt-empty-compact"><IconPhotoOff /><strong>这是二进制历史版本</strong><span>可另存为或恢复，但无法显示文本内容。</span></div>
    <StructuredDiffViewer v-else-if="revisionFile" :file="revisionFile" />
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { confirm, save } from '@tauri-apps/plugin-dialog'
import { IconArrowLeft, IconDownload, IconHistoryToggle, IconPhotoOff } from '@tabler/icons-vue'
import { readFileRevision, restoreFileRevision, saveFileRevisionAs } from '../changes/api'
import type { FileHistoryEntry, FileRevisionContent } from '../changes/types'
import StructuredDiffViewer from '../changes/StructuredDiffViewer.vue'
import { parseUnifiedDiff } from '../changes/diffParser'
import type { MockProjectFile } from '../mock/types'

const props = defineProps<{ projectPath: string; relativePath: string; entry: FileHistoryEntry }>()
const emit = defineEmits<{ close: []; restored: [] }>()
const content = ref<FileRevisionContent | null>(null)
const loading = ref(false)
const busy = ref(false)
const error = ref('')
const message = ref('')
let feedbackTimer: number | null = null
const revisionFile = computed<MockProjectFile | null>(() => {
  if (!content.value || content.value.isBinary) return null
  const diff = parseUnifiedDiff(content.value.diff)
  return {
    path: props.relativePath,
    name: fileName(props.relativePath),
    status: 'modified',
    size: formatBytes(content.value.size),
    modifiedAt: props.entry.date,
    language: props.relativePath.split('.').pop()?.toUpperCase() || 'File',
    content: content.value.content,
    additions: diff.filter((line) => line.kind === 'added').length,
    deletions: diff.filter((line) => line.kind === 'deleted').length,
    hasSelectedChanges: false,
    hasUnselectedChanges: false,
    diff
  }
})

watch(() => [props.projectPath, props.relativePath, props.entry.commit] as const, () => void loadContent(), { immediate: true })
watch(() => [error.value, message.value] as const, ([nextError, nextMessage]) => {
  if (feedbackTimer !== null) window.clearTimeout(feedbackTimer)
  if (!nextError && !nextMessage) return
  feedbackTimer = window.setTimeout(() => { error.value = ''; message.value = ''; feedbackTimer = null }, 3000)
})
onBeforeUnmount(() => { if (feedbackTimer !== null) window.clearTimeout(feedbackTimer) })

async function loadContent() {
  loading.value = true
  content.value = null
  error.value = ''
  message.value = ''
  try {
    content.value = await readFileRevision(props.projectPath, props.relativePath, props.entry.commit)
  } catch (reason) {
    error.value = `读取历史内容失败：${formatError(reason)}`
  } finally {
    loading.value = false
  }
}

async function saveRevision() {
  const targetPath = await save({ title: `另存 ${props.entry.shortCommit} 版本`, defaultPath: fileName(props.relativePath) })
  if (!targetPath) return
  busy.value = true
  error.value = ''
  try {
    await saveFileRevisionAs(props.projectPath, props.relativePath, props.entry.commit, targetPath)
    message.value = `已另存为：${targetPath}`
  } catch (reason) {
    error.value = `另存失败：${formatError(reason)}`
  } finally {
    busy.value = false
  }
}

async function restoreRevision() {
  const accepted = await confirm(
    `恢复 ${props.relativePath} 到 ${props.entry.shortCommit}？\n\n当前文件将被这个历史版本替换，并作为新的未提交变化保留。`,
    { title: '确认恢复文件记录', kind: 'warning' }
  )
  if (!accepted) return
  busy.value = true
  error.value = ''
  try {
    await restoreFileRevision(props.projectPath, props.relativePath, props.entry.commit)
    emit('restored')
  } catch (reason) {
    error.value = `恢复失败：${formatError(reason)}`
  } finally {
    busy.value = false
  }
}

function fileName(path: string) { return path.split('/').filter(Boolean).pop() || path }
function formatBytes(size: number) { return size < 1024 ? `${size} B` : `${(size / 1024).toFixed(1)} KB` }
function formatDate(value: string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleString() }
function formatError(reason: unknown) { return reason instanceof Error ? reason.message : typeof reason === 'object' && reason && 'message' in reason ? String((reason as { message: unknown }).message) : String(reason) }
</script>
