<template>
  <section v-if="canShowHistory" class="section file-history-section">
    <div class="section-header">
      <h2>{{ t('fileHistory') }}</h2>
      <span class="muted small">{{ history.length }}</span>
    </div>
    <p class="notice">{{ t('restoreVersionNotice') }}</p>
    <div v-if="historyMessage" class="notice">{{ historyMessage }}</div>
    <div v-if="historyError" class="error-box">{{ historyError }}</div>
    <div v-if="history.length" class="history-list">
      <div v-for="entry in history" :key="entry.commit" class="history-row">
        <div class="history-meta">
          <strong>{{ entry.subject }}</strong>
          <span class="muted small">{{ entry.shortCommit }} · {{ formatDate(entry.date) }}</span>
        </div>
        <div class="button-row">
          <button class="secondary-button" type="button" @click="saveRevision(entry)">
            {{ t('saveAs') }}
          </button>
          <button class="danger-button" type="button" @click="restoreRevision(entry)">
            {{ t('restoreVersion') }}
          </button>
        </div>
      </div>
    </div>
    <p v-else class="muted small">{{ t('noFileHistory') }}</p>
  </section>
</template>

<script setup lang="ts">
import { confirm, save } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'
import { t } from '../../../app/i18n'
import type { GitFileStatus } from '../../../core/commands/types'
import { listFileHistory, restoreFileRevision, saveFileRevisionAs } from '../changes/api'
import type { FileHistoryEntry } from '../changes/types'

const props = defineProps<{
  mode: 'empty' | 'directory' | 'content' | 'diff'
  status: GitFileStatus | null
  projectPath: string
  relativePath: string
  isGitRepository: boolean
}>()

const emit = defineEmits<{
  restored: []
}>()

const history = ref<FileHistoryEntry[]>([])
const historyError = ref('')
const historyMessage = ref('')

const canShowHistory = computed(
  () =>
    props.isGitRepository &&
    Boolean(props.projectPath) &&
    Boolean(props.relativePath) &&
    props.mode !== 'empty' &&
    props.mode !== 'directory' &&
    props.status !== 'untracked'
)

watch(
  () => [props.projectPath, props.relativePath, props.isGitRepository, props.mode, props.status] as const,
  () => {
    void refreshHistory()
  },
  { immediate: true }
)

async function refreshHistory() {
  history.value = []
  historyError.value = ''
  historyMessage.value = ''
  if (!canShowHistory.value) return

  try {
    history.value = await listFileHistory(props.projectPath, props.relativePath)
  } catch (err) {
    historyError.value = formatError(err)
  }
}

async function saveRevision(entry: FileHistoryEntry) {
  historyError.value = ''
  historyMessage.value = ''
  const targetPath = await save({
    title: t('saveAs'),
    defaultPath: fileName(props.relativePath)
  })
  if (!targetPath) return

  try {
    await saveFileRevisionAs(props.projectPath, props.relativePath, entry.commit, targetPath)
    historyMessage.value = `${t('saveAsSuccess')}: ${targetPath}`
  } catch (err) {
    historyError.value = formatError(err)
  }
}

async function restoreRevision(entry: FileHistoryEntry) {
  historyError.value = ''
  historyMessage.value = ''
  const command = `git checkout ${entry.commit} -- ${props.relativePath}`
  const confirmed = await confirm(`${t('restoreVersionNotice')}\n\n${command}`, {
    title: t('restoreVersion'),
    kind: 'warning'
  })
  if (!confirmed) return

  try {
    await restoreFileRevision(props.projectPath, props.relativePath, entry.commit)
    emit('restored')
  } catch (err) {
    historyError.value = formatError(err)
  }
}

function fileName(relativePath: string) {
  return relativePath.split('/').filter(Boolean).pop() || relativePath
}

function formatDate(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

function formatError(err: unknown) {
  if (typeof err === 'object' && err && 'message' in err) {
    return String((err as { message: unknown }).message)
  }
  return String(err)
}
</script>
