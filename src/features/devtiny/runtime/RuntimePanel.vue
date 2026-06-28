<template>
  <section class="runtime-panel">
    <div class="runtime-hero">
      <div class="runtime-title-block">
        <div class="runtime-title-row">
          <h1>{{ projectName }}</h1>
          <span class="status-pill" :class="running ? 'running' : 'stopped'">
            {{ running ? t('running') : t('stopped') }}
          </span>
        </div>
        <p class="runtime-compose-path muted">{{ projectPath || t('noProject') }}</p>
      </div>
      <div class="runtime-kind">
        <strong>Docker Compose {{ t('runtime') }}</strong>
        <span class="muted small">{{ hasComposeFile ? composeFilePath : t('composeMissing') }}</span>
      </div>
    </div>

    <div class="runtime-status-summary" aria-label="Project status">
      <div class="runtime-status-item">
        <IconGitBranch class="runtime-status-icon" />
        <span class="muted small">{{ t('gitRepository') }}</span>
        <strong :class="isGitRepository ? 'text-ok' : 'text-warn'">
          {{ isGitRepository ? t('gitRepoYes') : t('gitRepoNo') }}
        </strong>
      </div>
      <div class="runtime-status-item">
        <IconSettings class="runtime-status-icon" />
        <span class="muted small">{{ t('compose') }}</span>
        <strong :class="hasComposeFile ? 'text-ok' : 'muted'">
          {{ hasComposeFile ? t('composeFound') : t('composeMissing') }}
        </strong>
      </div>
      <div class="runtime-status-item">
        <IconPlayerPlay class="runtime-status-icon" />
        <span class="muted small">{{ t('projectStatus') }}</span>
        <strong :class="running ? 'text-ok' : 'muted'">
          {{ running ? t('running') : t('stopped') }}
        </strong>
      </div>
    </div>

    <div class="runtime-section runtime-setup-section">
      <div class="runtime-section-head">
        <h2>{{ t('projectPrepare') }}</h2>
        <p>{{ t('projectPrepareBody') }}</p>
      </div>
      <div class="runtime-status-row">
        <div class="runtime-action-tile setup-tile">
          <IconDownload class="runtime-tile-icon" />
          <div>
            <strong>{{ t('firstRunProject') }}</strong>
            <p>{{ t('firstRunBody') }}</p>
          </div>
          <button class="tile-arrow-button" type="button" :disabled="!hasComposeFile || task?.status === 'running'" @click="$emit('firstRun')">
            <IconChevronRight />
          </button>
        </div>
        <div class="runtime-action-tile setup-tile">
          <IconWorld class="runtime-tile-icon" />
          <div>
            <strong>{{ t('configureMirrors') }}</strong>
            <p>{{ t('mirrorConfigBody') }}</p>
          </div>
          <button class="tile-arrow-button" type="button" :disabled="!hasComposeFile" @click="$emit('configureMirrors')">
            <IconChevronRight />
          </button>
        </div>
      </div>
    </div>

    <div class="runtime-section runtime-daily-section">
      <div class="runtime-section-head">
        <h2>{{ t('dailyOperations') }}</h2>
        <p>{{ t('dailyOperationsBody') }}</p>
      </div>
      <div class="runtime-action-row">
        <button class="runtime-daily-button primary-button" type="button" :disabled="!hasComposeFile" @click="$emit('run', 'runtime.start')">
          <IconPlayerPlayFilled />
          <strong>{{ t('startProject') }}</strong>
        </button>
        <button class="runtime-daily-button danger-button" type="button" :disabled="!hasComposeFile" @click="$emit('run', 'runtime.stop')">
          <IconPlayerStopFilled />
          <strong>{{ t('stopProject') }}</strong>
        </button>
        <button class="runtime-daily-button secondary-button" type="button" :disabled="!hasComposeFile" @click="$emit('run', 'runtime.restart')">
          <IconRefresh />
          <strong>{{ t('restartProject') }}</strong>
        </button>
        <button class="runtime-daily-button secondary-button" type="button" :disabled="!hasComposeFile" @click="$emit('run', 'runtime.logs')">
          <IconFileText />
          <strong>{{ t('viewLogs') }}</strong>
        </button>
      </div>
    </div>

    <div v-if="mirrorMessage" class="notice">{{ mirrorMessage }}</div>
    <div v-if="task" class="runtime-progress">
      <div class="section-header">
        <h2>{{ t('firstRunProgress') }}</h2>
        <div class="runtime-progress-meta">
          <span class="muted small">{{ t('autoRefreshHint') }}</span>
          <span class="status-pill" :class="task.status === 'succeeded' ? 'ok' : task.status === 'failed' ? 'deleted' : 'running'">
            {{ taskStatusLabel }}
          </span>
          <button
            v-if="task.status === 'running'"
            class="secondary-button compact-button"
            type="button"
            @click="$emit('cancelTask')"
          >
            <IconPlayerStop />
            {{ t('cancelTask') }}
          </button>
        </div>
      </div>
      <code>{{ task.command }}</code>
      <pre>{{ task.output || t('noOutput') }}</pre>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  IconChevronRight,
  IconDownload,
  IconFileText,
  IconGitBranch,
  IconPlayerPlay,
  IconPlayerPlayFilled,
  IconPlayerStop,
  IconPlayerStopFilled,
  IconRefresh,
  IconSettings,
  IconWorld
} from '@tabler/icons-vue'
import { t } from '../../../app/i18n'
import type { RuntimeTask, WorkbenchAction } from '../../../core/commands/types'

const props = defineProps<{
  projectPath: string
  isGitRepository: boolean
  hasComposeFile: boolean
  composeFilePath?: string
  running: boolean
  task: RuntimeTask | null
  mirrorMessage: string
}>()

defineEmits<{
  run: [action: Extract<WorkbenchAction, `runtime.${string}`>]
  firstRun: []
  configureMirrors: []
  cancelTask: []
}>()

const projectName = computed(() => {
  if (!props.projectPath) return t('noProject')
  return props.projectPath.split('/').filter(Boolean).pop() || props.projectPath
})

const taskStatusLabel = computed(() => {
  if (!props.task) return ''
  if (props.task.status === 'succeeded') return t('taskSucceeded')
  if (props.task.status === 'failed') return t('taskFailed')
  if (props.task.status === 'cancelled') return t('taskCancelled')
  return t('taskRunning')
})
</script>
