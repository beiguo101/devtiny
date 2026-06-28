<template>
  <section class="content-panel">
    <div v-if="mode === 'empty'" class="empty-state">
      <strong>{{ t('selectFile') }}</strong>
      <p>{{ t('selectFileBody') }}</p>
    </div>

    <div v-else-if="mode === 'directory'" class="section">
      <h2>{{ t('directorySummary') }}</h2>
      <div class="summary-grid">
        <div>
          <span class="muted small">{{ t('directories') }}</span>
          <strong>{{ directorySummary?.directories ?? 0 }}</strong>
        </div>
        <div>
          <span class="muted small">{{ t('files') }}</span>
          <strong>{{ directorySummary?.files ?? 0 }}</strong>
        </div>
      </div>
    </div>

    <div v-else class="section">
      <div class="content-title-block">
        <span class="muted small">{{ mode === 'diff' ? t('fileDiff') : t('fileContent') }}</span>
        <h1>{{ relativePath || t('fileContent') }}</h1>
      </div>
      <div class="section-header">
        <h2>{{ mode === 'diff' ? t('fileDiff') : t('fileContent') }}</h2>
        <div v-if="canEdit" class="button-row">
          <span v-if="dirty" class="muted small">{{ t('unsavedChanges') }}</span>
          <button class="secondary-button compact-button" type="button" :disabled="!dirty" @click="resetDraft">
            {{ t('discardChanges') }}
          </button>
          <button class="primary-button compact-button" type="button" :disabled="!dirty || saving" @click="saveDraft">
            {{ t('saveFile') }}
          </button>
        </div>
      </div>
      <p v-if="status === 'added' || status === 'untracked'" class="notice">{{ t('newFileNotice') }}</p>
      <p v-if="status === 'deleted'" class="notice">{{ t('deletedFileNotice') }}</p>
      <div v-if="message" class="notice">{{ message }}</div>
      <div v-if="error" class="error-box">{{ error }}</div>
      <CodeViewer
        :text="displayText"
        :relative-path="relativePath"
        :mode="mode === 'diff' ? 'diff' : 'content'"
        :editable="canEdit"
        @change="draft = $event"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { t } from '../../../app/i18n'
import type { GitFileStatus } from '../../../core/commands/types'
import { writeProjectFile } from '../../../core/filesystem/api'
import type { DirectorySummary, FileContent } from '../../../core/filesystem/types'
import CodeViewer from './CodeViewer.vue'

const props = defineProps<{
  mode: 'empty' | 'directory' | 'content' | 'diff'
  directorySummary: DirectorySummary | null
  fileContent: FileContent | null
  diff: string
  status: GitFileStatus | null
  projectPath: string
  relativePath: string
  isGitRepository: boolean
}>()

const emit = defineEmits<{
  saved: []
}>()

const draft = ref('')
const saving = ref(false)
const error = ref('')
const message = ref('')

const displayText = computed(() => {
  if (props.mode === 'diff') return props.diff || t('noOutput')
  if (props.fileContent?.isBinary) return t('binaryFile')
  return draft.value
})

const canEdit = computed(
  () => props.mode === 'content' && !props.fileContent?.isBinary && Boolean(props.projectPath && props.relativePath)
)

const dirty = computed(() => canEdit.value && draft.value !== (props.fileContent?.content || ''))

watch(
  () => [props.fileContent?.content, props.relativePath, props.mode] as const,
  () => {
    draft.value = props.fileContent?.content || ''
    error.value = ''
    message.value = ''
  },
  { immediate: true }
)

function resetDraft() {
  draft.value = props.fileContent?.content || ''
  error.value = ''
  message.value = ''
}

async function saveDraft() {
  if (!canEdit.value) return
  saving.value = true
  error.value = ''
  message.value = ''
  try {
    await writeProjectFile(props.projectPath, props.relativePath, draft.value)
    message.value = t('fileSaved')
    emit('saved')
  } catch (err) {
    error.value = formatError(err)
  } finally {
    saving.value = false
  }
}

function formatError(err: unknown) {
  if (typeof err === 'object' && err && 'message' in err) {
    return String((err as { message: unknown }).message)
  }
  return String(err)
}
</script>
