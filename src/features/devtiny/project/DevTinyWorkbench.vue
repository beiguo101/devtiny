<template>
  <div class="workbench-shell">
    <aside class="workbench-sidebar">
      <div class="brand-row">
        <strong>{{ t('appName') }}</strong>
        <select :value="locale" :aria-label="t('language')" @change="changeLocale">
          <option value="zh-CN">中文</option>
          <option value="en-US">English</option>
        </select>
      </div>

      <div class="project-picker-row">
        <button class="primary-button full-width" type="button" @click="chooseProject">
          {{ t('chooseProject') }}
        </button>
        <button
          class="secondary-button project-refresh-button"
          type="button"
          :disabled="!projectPath"
          @click="refreshAll"
        >
          <IconRefresh />
          {{ t('refresh') }}
        </button>
      </div>

      <button
        class="secondary-button sidebar-runtime-button"
        type="button"
        :class="{ active: mainView === 'runtime' }"
        :disabled="!projectPath"
        @click="mainView = 'runtime'"
      >
        <IconPlayerPlay />
        {{ t('runtime') }}
      </button>

      <div class="sidebar-file-actions">
        <button class="secondary-button compact-button" type="button" :disabled="!projectPath" @click="createFile">
          {{ t('newFile') }}
        </button>
        <button
          class="secondary-button compact-button"
          type="button"
          :disabled="!canDeleteSelectedFile"
          @click="deleteSelectedFile"
        >
          {{ t('deleteFile') }}
        </button>
      </div>

      <div class="segmented">
        <button :class="{ active: leftView === 'files' }" type="button" @click="leftView = 'files'">
          {{ t('fileView') }}
        </button>
        <button :class="{ active: leftView === 'changes' }" type="button" @click="leftView = 'changes'">
          {{ t('changesView') }}
        </button>
      </div>

      <FileTreeView
        v-if="leftView === 'files'"
        :nodes="fileTree"
        :selected-path="selectedPath"
        @select="selectNode"
      />
      <ChangesView
        v-else
        :changes="changes"
        :selected-paths="selectedChangePaths"
        @select="selectChange"
        @selection-change="selectedChangeFiles = $event"
      />
    </aside>

    <main class="workbench-main">
      <section v-if="projectPath && overview && !overview.isGitRepository && showNonGitPrompt" class="non-git-banner">
        <div>
          <strong>{{ t('nonGitTitle') }}</strong>
          <p>{{ t('nonGitBody') }}</p>
        </div>
        <div class="button-row">
          <button class="primary-button" type="button" @click="prepareAction('git.init')">{{ t('initGit') }}</button>
          <button class="secondary-button" type="button" @click="showNonGitPrompt = false">{{ t('skipGit') }}</button>
        </div>
      </section>

      <div v-if="error" class="error-box">{{ error }}</div>
      <RuntimePanel
        v-if="mainView === 'runtime' && projectPath"
        class="main-runtime-panel"
        :project-path="projectPath"
        :is-git-repository="overview?.isGitRepository ?? false"
        :has-compose-file="overview?.hasComposeFile ?? false"
        :compose-file-path="overview?.composeFilePath"
        :running="overview?.running ?? false"
        :task="runtimeTask"
        :mirror-message="mirrorMessage"
        @run="prepareAction"
        @first-run="startFirstRun"
        @configure-mirrors="configureMirrors"
        @cancel-task="cancelFirstRun"
      />

      <template v-else>
        <ContentPanel
          :mode="contentMode"
          :directory-summary="directorySummary"
          :file-content="fileContent"
          :diff="diff"
          :status="selectedStatus"
          :project-path="projectPath"
          :relative-path="selectedPath"
          :is-git-repository="overview?.isGitRepository ?? false"
          @saved="handleFileSaved"
        />

        <section v-if="projectPath && overview?.isGitRepository" class="file-actions-panel">
          <div class="action-row">
            <input v-model="commitMessage" type="text" :placeholder="t('commitMessage')" />
            <button
              class="secondary-button"
              type="button"
              :disabled="!selectedChangeFiles.length"
              @click="commitFiles(selectedChangeFiles)"
            >
              {{ t('commitSelected') }}
            </button>
            <button class="secondary-button" type="button" :disabled="!changes.length" @click="commitAll">
              {{ t('commitAll') }}
            </button>
            <button
              class="secondary-button"
              type="button"
              :disabled="!selectedChangeFiles.length"
              @click="ignoreFiles(selectedChangeFiles)"
            >
              {{ t('ignoreSelected') }}
            </button>
            <button
              class="danger-button"
              type="button"
              :disabled="!selectedChangeFiles.length"
              @click="restoreFiles(selectedChangeFiles)"
            >
              {{ t('restoreSelected') }}
            </button>
          </div>
          <p v-if="actionError" class="action-error">{{ actionError }}</p>

          <div class="staged-area">
            <div class="selected-files-head">
              <h2>{{ t('stagedFiles') }}</h2>
              <span class="muted small">{{ stagedFiles.length }}</span>
            </div>
            <div v-if="stagedFiles.length" class="staged-file-list">
              <span v-for="file in stagedFiles" :key="file.relativePath" class="staged-file-chip">
                <span>{{ file.relativePath }}</span>
                <button type="button" :aria-label="t('unstageFile')" @click="unstageFile(file)">x</button>
              </span>
            </div>
            <p v-else class="muted small">{{ t('noStagedFiles') }}</p>
          </div>
        </section>

        <FileHistoryPanel
          :mode="contentMode"
          :status="selectedStatus"
          :project-path="projectPath"
          :relative-path="selectedPath"
          :is-git-repository="overview?.isGitRepository ?? false"
          @restored="refreshAll"
        />
      </template>
    </main>

    <CommandConfirmDialog
      :preview="preview"
      :busy="busy"
      :error="dialogError"
      @cancel="preview = null"
      @confirm="executePreview"
    />

    <div v-if="outputOpen && lastOutput" class="dialog-backdrop">
      <div class="dialog output-dialog">
        <div class="section-header">
          <h2>{{ t('output') }}</h2>
          <button class="secondary-button compact-button" type="button" @click="outputOpen = false">
            {{ t('close') }}
          </button>
        </div>
        <pre>{{ lastOutput }}</pre>
      </div>
    </div>

    <div v-if="newFileDialogOpen" class="dialog-backdrop">
      <form class="dialog new-file-dialog" @submit.prevent="submitCreateFile">
        <h2>{{ t('newFile') }}</h2>
        <p class="muted small">{{ t('newFilePrompt') }}</p>
        <input v-model="newFilePath" type="text" autofocus placeholder="app/config.py" />
        <p v-if="newFileError" class="error-box">{{ newFileError }}</p>
        <div class="dialog-actions">
          <button class="secondary-button" type="button" @click="newFileDialogOpen = false">
            {{ t('cancel') }}
          </button>
          <button class="primary-button" type="submit">
            {{ t('newFile') }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { confirm, open } from '@tauri-apps/plugin-dialog'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { IconPlayerPlay, IconRefresh } from '@tabler/icons-vue'
import { locale, setLocale, t, type Locale } from '../../../app/i18n'
import {
  cancelRuntimeTask,
  configureProjectMirrors,
  executeWorkbenchAction,
  getRuntimeTask,
  previewWorkbenchAction,
  startRuntimeFirstRun
} from '../../../core/commands/api'
import type {
  CommandPreview,
  GitFileSelection,
  GitFileStatus,
  RuntimeTask,
  WorkbenchAction
} from '../../../core/commands/types'
import {
  createProjectFile,
  deleteProjectFile,
  getDirectorySummary,
  listProjectFiles,
  readProjectFile
} from '../../../core/filesystem/api'
import type { DirectorySummary, FileContent, FileTreeNode } from '../../../core/filesystem/types'
import { getProjectOverview } from '../../../core/project/api'
import type { ProjectOverview } from '../../../core/project/types'
import { getFileDiff, listGitChanges } from '../changes/api'
import type { GitChangeFile } from '../changes/types'
import ChangesView from '../changes/ChangesView.vue'
import ContentPanel from '../files/ContentPanel.vue'
import FileTreeView from '../files/FileTreeView.vue'
import FileHistoryPanel from '../history/FileHistoryPanel.vue'
import RuntimePanel from '../runtime/RuntimePanel.vue'
import CommandConfirmDialog from './CommandConfirmDialog.vue'

const projectPath = ref(localStorage.getItem('devtiny.projectPath') || '')
const overview = ref<ProjectOverview | null>(null)
const fileTree = ref<FileTreeNode[]>([])
const changes = ref<GitChangeFile[]>([])
const selectedChangeFiles = ref<GitFileSelection[]>([])
const leftView = ref<'files' | 'changes'>('files')
const selectedPath = ref('')
const selectedNodeKind = ref<'directory' | 'file' | null>(null)
const selectedStatus = ref<GitFileStatus | null>(null)
const contentMode = ref<'empty' | 'directory' | 'content' | 'diff'>('empty')
const directorySummary = ref<DirectorySummary | null>(null)
const fileContent = ref<FileContent | null>(null)
const diff = ref('')
const error = ref('')
const actionError = ref('')
const dialogError = ref('')
const busy = ref(false)
const preview = ref<CommandPreview | null>(null)
const commitMessage = ref('')
const lastOutput = ref('')
const outputOpen = ref(false)
const runtimeTask = ref<RuntimeTask | null>(null)
const runtimeTaskTimer = ref<number | null>(null)
const mirrorMessage = ref('')
const newFileDialogOpen = ref(false)
const newFilePath = ref('')
const newFileError = ref('')
const showNonGitPrompt = ref(true)
const mainView = ref<'content' | 'runtime'>('content')

const gitEnabled = computed(() => Boolean(overview.value?.isGitRepository))
const stagedFiles = computed<GitFileSelection[]>(() =>
  selectedChangeFiles.value
)
const selectedChangePaths = computed(() => selectedChangeFiles.value.map((file) => file.relativePath))
const canDeleteSelectedFile = computed(() => Boolean(projectPath.value && selectedPath.value && contentMode.value === 'content'))

onMounted(() => {
  if (projectPath.value) {
    void refreshAll()
  }
})

onBeforeUnmount(() => {
  stopRuntimeTaskPolling()
})

async function chooseProject() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: t('chooseProject')
  })
  if (typeof selected === 'string') {
    projectPath.value = selected
    localStorage.setItem('devtiny.projectPath', selected)
    showNonGitPrompt.value = true
    mainView.value = 'runtime'
    await refreshAll()
  }
}

async function refreshAll() {
  if (!projectPath.value) return
  error.value = ''
  try {
    overview.value = await getProjectOverview(projectPath.value)
    fileTree.value = await listProjectFiles(projectPath.value)
    changes.value = overview.value.isGitRepository ? await listGitChanges(projectPath.value) : []
    selectedChangeFiles.value = selectedChangeFiles.value.filter((file) =>
      changes.value.some((change) => change.relativePath === file.relativePath)
    )
  } catch (err) {
    error.value = formatError(err)
  }
}

async function selectNode(node: FileTreeNode) {
  mainView.value = 'content'
  selectedPath.value = node.relativePath
  selectedNodeKind.value = node.kind
  selectedStatus.value = node.status
  directorySummary.value = null
  fileContent.value = null
  diff.value = ''
  error.value = ''

  try {
    if (node.kind === 'directory') {
      directorySummary.value = await getDirectorySummary(projectPath.value, node.relativePath)
      contentMode.value = 'directory'
      return
    }

    fileContent.value = await readProjectFile(projectPath.value, node.relativePath)
    contentMode.value = 'content'
  } catch (err) {
    error.value = formatError(err)
  }
}

async function selectChange(change: GitChangeFile) {
  mainView.value = 'content'
  selectedPath.value = change.relativePath
  selectedNodeKind.value = 'file'
  selectedStatus.value = change.status
  directorySummary.value = null
  fileContent.value = null
  diff.value = ''
  error.value = ''

  try {
    if (change.status === 'added' || change.status === 'untracked') {
      fileContent.value = await readProjectFile(projectPath.value, change.relativePath)
      contentMode.value = 'content'
      return
    }

    const result = await getFileDiff(projectPath.value, change.relativePath, change.status)
    diff.value = result.diff
    contentMode.value = 'diff'
  } catch (err) {
    error.value = formatError(err)
  }
}

async function createFile() {
  if (!projectPath.value) return
  newFilePath.value = defaultNewFilePath()
  newFileError.value = ''
  newFileDialogOpen.value = true
}

function defaultNewFilePath() {
  if (!selectedPath.value) return ''
  if (selectedNodeKind.value === 'directory') {
    return selectedPath.value ? `${selectedPath.value.replace(/\/$/, '')}/` : ''
  }
  return selectedPath.value.includes('/') ? `${selectedPath.value.split('/').slice(0, -1).join('/')}/` : ''
}

async function submitCreateFile() {
  if (!projectPath.value) return
  const relativePath = newFilePath.value.trim()
  if (!relativePath) {
    newFileError.value = t('newFilePrompt')
    return
  }

  error.value = ''
  newFileError.value = ''
  try {
    const result = await createProjectFile(projectPath.value, relativePath, '')
    newFileDialogOpen.value = false
    await refreshAll()
    await selectNode({
      name: result.relativePath.split('/').filter(Boolean).pop() || result.relativePath,
      relativePath: result.relativePath,
      kind: 'file',
      status: 'untracked',
      children: []
    })
  } catch (err) {
    newFileError.value = formatError(err)
  }
}

async function deleteSelectedFile() {
  if (!projectPath.value || !selectedPath.value || contentMode.value !== 'content') return
  const confirmed = await confirm(`${t('deleteFileConfirm')}\n\n${selectedPath.value}`, {
    title: t('deleteFile'),
    kind: 'warning'
  })
  if (!confirmed) return

  error.value = ''
  try {
    await deleteProjectFile(projectPath.value, selectedPath.value)
    selectedPath.value = ''
    selectedNodeKind.value = null
    selectedStatus.value = null
    fileContent.value = null
    diff.value = ''
    contentMode.value = 'empty'
    await refreshAll()
  } catch (err) {
    error.value = formatError(err)
  }
}

async function handleFileSaved() {
  await refreshAll()
  if (selectedPath.value && contentMode.value === 'content') {
    fileContent.value = await readProjectFile(projectPath.value, selectedPath.value)
  }
}

function unstageFile(file: GitFileSelection) {
  selectedChangeFiles.value = selectedChangeFiles.value.filter(
    (selectedFile) => selectedFile.relativePath !== file.relativePath
  )
}

function commitFiles(files: GitFileSelection[]) {
  actionError.value = ''
  if (!commitMessage.value.trim()) {
    actionError.value = t('commitMessage')
    return
  }
  void prepareAction('git.commitFiles', { files, message: commitMessage.value })
}

function commitAll() {
  actionError.value = ''
  if (!commitMessage.value.trim()) {
    actionError.value = t('commitMessage')
    return
  }
  void prepareAction('git.commitAll', { message: commitMessage.value })
}

function restoreFiles(files: GitFileSelection[]) {
  void prepareAction('git.restoreFiles', { files })
}

function ignoreFiles(files: GitFileSelection[]) {
  void prepareAction('git.ignoreFiles', { files })
}

async function prepareAction(
  action: WorkbenchAction,
  payload?: { files?: GitFileSelection[]; message?: string }
) {
  if (!projectPath.value) return
  error.value = ''
  actionError.value = ''
  dialogError.value = ''
  try {
    preview.value = await previewWorkbenchAction({
      projectPath: projectPath.value,
      action,
      payload
    })
  } catch (err) {
    error.value = formatError(err)
  }
}

async function executePreview() {
  if (!preview.value) return
  busy.value = true
  dialogError.value = ''
  try {
    const result = await executeWorkbenchAction(preview.value.previewToken)
    lastOutput.value = [result.stdout, result.stderr].filter(Boolean).join('\n')
    outputOpen.value = Boolean(lastOutput.value)
    preview.value = null
    await refreshAll()
  } catch (err) {
    dialogError.value = formatError(err)
  } finally {
    busy.value = false
  }
}

async function startFirstRun() {
  if (!projectPath.value) return
  const confirmed = await confirm(t('confirmFirstRunBody'), {
    title: t('confirmFirstRunTitle'),
    kind: 'warning'
  })
  if (!confirmed) return

  error.value = ''
  mirrorMessage.value = ''
  mainView.value = 'runtime'
  stopRuntimeTaskPolling()

  try {
    runtimeTask.value = await startRuntimeFirstRun(projectPath.value)
    runtimeTaskTimer.value = window.setInterval(() => {
      void refreshRuntimeTask()
    }, 10000)
    void refreshRuntimeTask()
  } catch (err) {
    error.value = formatError(err)
  }
}

async function refreshRuntimeTask() {
  if (!runtimeTask.value) return

  try {
    runtimeTask.value = await getRuntimeTask(runtimeTask.value.taskId)
    if (runtimeTask.value.status !== 'running') {
      stopRuntimeTaskPolling()
      await refreshAll()
    }
  } catch (err) {
    stopRuntimeTaskPolling()
    error.value = formatError(err)
  }
}

function stopRuntimeTaskPolling() {
  if (runtimeTaskTimer.value === null) return
  window.clearInterval(runtimeTaskTimer.value)
  runtimeTaskTimer.value = null
}

async function configureMirrors() {
  if (!projectPath.value) return
  const confirmed = await confirm(t('confirmMirrorsBody'), {
    title: t('confirmMirrorsTitle'),
    kind: 'warning'
  })
  if (!confirmed) return

  error.value = ''
  mirrorMessage.value = ''
  mainView.value = 'runtime'

  try {
    const result = await configureProjectMirrors(projectPath.value)
    mirrorMessage.value = `${t('mirrorConfigDone')}: ${result.files.join(', ')}`
    await refreshAll()
  } catch (err) {
    error.value = formatError(err)
  }
}

async function cancelFirstRun() {
  if (!runtimeTask.value || runtimeTask.value.status !== 'running') return
  error.value = ''

  try {
    runtimeTask.value = await cancelRuntimeTask(runtimeTask.value.taskId)
    stopRuntimeTaskPolling()
  } catch (err) {
    error.value = formatError(err)
  }
}

function changeLocale(event: Event) {
  const value = (event.target as HTMLSelectElement).value as Locale
  setLocale(value)
}

function formatError(err: unknown) {
  if (typeof err === 'object' && err && 'message' in err) {
    return String((err as { message: unknown }).message)
  }
  return String(err)
}
</script>
