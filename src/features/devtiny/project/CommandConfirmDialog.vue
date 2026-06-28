<template>
  <div v-if="preview" class="dialog-backdrop">
    <div class="dialog">
      <h2>{{ t('commandPreview') }}</h2>
      <div class="command-preview">
        <div class="muted small">{{ t('currentProject') }}</div>
        <code>{{ preview.projectPath }}</code>
        <pre>{{ preview.commands.map((command) => command.display).join('\n') }}</pre>
      </div>
      <div v-if="preview.affectedFiles.length" class="affected-list">
        <strong>{{ t('affectedFiles') }}</strong>
        <ul>
          <li v-for="file in preview.affectedFiles" :key="file.relativePath">
            {{ statusLabel(file.status) }} · {{ file.relativePath }}
          </li>
        </ul>
      </div>
      <p v-if="preview.deletesUntracked" class="warning-box">{{ t('untrackedWarning') }}</p>
      <p v-if="preview.action === 'git.ignoreFiles'" class="warning-box">{{ t('ignoreWarning') }}</p>
      <p v-if="error" class="error-box">{{ error }}</p>
      <div class="dialog-actions">
        <button class="secondary-button" type="button" :disabled="busy" @click="$emit('cancel')">{{ t('cancel') }}</button>
        <button class="primary-button" type="button" :disabled="busy" @click="$emit('confirm')">
          {{ preview.action === 'git.restoreFiles' ? t('confirmRestore') : t('confirmRun') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { statusLabel, t } from '../../../app/i18n'
import type { CommandPreview } from '../../../core/commands/types'

defineProps<{
  preview: CommandPreview | null
  busy: boolean
  error: string
}>()

defineEmits<{
  cancel: []
  confirm: []
}>()
</script>
