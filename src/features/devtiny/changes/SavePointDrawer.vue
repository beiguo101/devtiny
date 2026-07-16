<template>
  <section class="dt-save-point-drawer" aria-label="最近保存点">
    <header>
      <div>
        <IconHistory />
        <span><strong>最近保存点</strong><small>按时间倒序，选择后在上方查看文件和具体差异</small></span>
      </div>
      <button v-if="selected" type="button" @click="$emit('showCurrent')"><IconArrowUp /> 返回当前变化</button>
    </header>
    <div v-if="loading && !entries.length" class="dt-save-point-loading"><span></span>正在读取保存点…</div>
    <div v-else-if="error" class="dt-save-point-empty is-error">{{ error }}</div>
    <div v-else-if="!entries.length" class="dt-save-point-empty">还没有可展示的保存点。</div>
    <div v-else class="dt-save-point-body">
      <div class="dt-save-point-grid">
        <button
          v-for="entry in entries"
          :key="entry.commit"
          type="button"
          :class="{ active: selected?.commit === entry.commit }"
          @click="$emit('select', entry)"
        >
          <span class="dt-save-point-node"><IconGitCommit /></span>
          <span class="dt-save-point-copy"><strong>{{ entry.subject }}</strong><small><code>{{ entry.shortCommit }}</code> · {{ formatDate(entry.date) }}</small></span>
          <IconChevronUp class="dt-save-point-chevron" />
        </button>
      </div>
      <button v-if="hasMore" class="dt-save-point-more" type="button" :disabled="loading" @click="$emit('loadMore')">
        <IconLoader2 v-if="loading" class="dt-spin" />
        <IconPlus v-else />
        {{ loading ? '正在加载…' : '再加载 3 个保存点' }}
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { IconArrowUp, IconChevronUp, IconGitCommit, IconHistory, IconLoader2, IconPlus } from '@tabler/icons-vue'
import type { FileHistoryEntry } from './types'

defineProps<{
  entries: FileHistoryEntry[]
  selected: FileHistoryEntry | null
  hasMore: boolean
  loading: boolean
  error: string
}>()

defineEmits<{
  select: [entry: FileHistoryEntry]
  loadMore: []
  showCurrent: []
}>()

function formatDate(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}
</script>
