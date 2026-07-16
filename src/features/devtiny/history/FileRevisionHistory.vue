<template>
  <section v-if="canShow" class="dt-revision-history">
    <div class="dt-section-title">
      <div><h2>文件修改记录</h2><p>时间倒序，恢复操作会替换当前文件</p></div>
      <span>{{ loading ? '读取中…' : `${history.length} 条` }}</span>
    </div>
    <div v-if="error" class="dt-history-message is-error">{{ error }}</div>
    <div v-if="message" class="dt-history-message">{{ message }}</div>
    <div v-if="loading" class="dt-history-loading"><span></span>正在读取提交记录…</div>
    <div v-else-if="visibleHistory.length" class="dt-history-list">
      <button v-for="(entry, index) in visibleHistory" :key="entry.commit" class="dt-history-row" :class="{ selected: selectedCommit === entry.commit }" type="button" @click="$emit('selectRevision', selectedCommit === entry.commit ? null : entry)">
        <span class="dt-history-index">{{ index + 1 }}</span>
        <div class="dt-history-meta">
          <strong>{{ entry.subject }}</strong>
          <span><code>{{ entry.shortCommit }}</code> · {{ formatDate(entry.date) }}</span>
        </div>
        <IconChevronRight class="dt-history-arrow" />
      </button>
      <div v-if="history.length > 3" class="dt-history-more">
        <button v-if="visibleCount < history.length" type="button" @click="visibleCount += 5">
          显示更多（剩余 {{ history.length - visibleCount }} 条）<IconChevronDown />
        </button>
        <button v-else type="button" @click="visibleCount = 3">收起到最近 3 条<IconChevronUp /></button>
      </div>
    </div>
    <div v-else class="dt-history-empty">这个文件还没有可恢复的提交记录。</div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { IconChevronDown, IconChevronRight, IconChevronUp } from '@tabler/icons-vue'
import { listFileHistory } from '../changes/api'
import type { FileHistoryEntry } from '../changes/types'

const props = defineProps<{
  projectPath: string
  relativePath: string
  isGitRepository: boolean
  realMode: boolean
  selectedCommit?: string
}>()

defineEmits<{ selectRevision: [entry: FileHistoryEntry | null] }>()
const history = ref<FileHistoryEntry[]>([])
const visibleCount = ref(3)
const loading = ref(false)
const error = ref('')
const message = ref('')

const canShow = computed(() => props.realMode && props.isGitRepository && Boolean(props.projectPath && props.relativePath))
const visibleHistory = computed(() => history.value.slice(0, visibleCount.value))

watch(() => [props.projectPath, props.relativePath, props.isGitRepository, props.realMode] as const, () => void refresh(), { immediate: true })

async function refresh() {
  history.value = []
  visibleCount.value = 3
  error.value = ''
  message.value = ''
  if (!canShow.value) return
  loading.value = true
  try {
    history.value = await listFileHistory(props.projectPath, props.relativePath)
  } catch (reason) {
    error.value = formatError(reason)
  } finally {
    loading.value = false
  }
}

function formatDate(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}
function formatError(reason: unknown) {
  if (reason instanceof Error) return reason.message
  if (typeof reason === 'object' && reason && 'message' in reason) return String((reason as { message: unknown }).message)
  return String(reason)
}
</script>
