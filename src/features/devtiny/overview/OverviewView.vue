<template>
  <section class="dt-view dt-overview-view">
    <header class="dt-view-heading">
      <div>
        <span class="dt-eyebrow">项目概览</span>
        <h1>当前项目处于什么状态？</h1>
        <p>从最近一个稳定点开始，项目中还有哪些变化需要理解和处理。</p>
      </div>
      <button class="dt-button dt-button-primary" type="button" @click="$emit('openChanges')">
        查看全部变化 <IconArrowRight />
      </button>
    </header>

    <div v-if="!isGitRepository" class="dt-state-panel is-warning">
      <IconGitBranch />
      <div><strong>这不是 Git 仓库</strong><p>仍可浏览当前文件；初始化后即可记录变化和建立保存点。</p></div>
      <button class="dt-button dt-button-primary" type="button" :disabled="initializingGit" @click="$emit('initGit')">
        <IconLoader2 v-if="initializingGit" class="dt-spin" /><IconGitCommit v-else />
        {{ initializingGit ? '正在初始化…' : '初始化 Git' }}
      </button>
    </div>
    <div v-else-if="!hasFirstCommit" class="dt-state-panel">
      <IconGitCommit />
      <div><strong>还没有第一个保存点</strong><p>这是一个空 Git 仓库。创建首次提交后，DevTiny 才能比较之后的变化。</p></div>
    </div>

    <div class="dt-overview-grid">
      <article class="dt-overview-primary">
        <span class="dt-card-label">当前工作区</span>
        <strong class="dt-large-number">{{ changedFiles.length }}</strong>
        <span>{{ changedFiles.length ? '个文件发生变化' : '没有未保存变化' }}</span>
        <div class="dt-change-breakdown">
          <span><i class="is-added"></i>新增 {{ statusCounts.added }}</span>
          <span><i class="is-modified"></i>修改 {{ statusCounts.modified }}</span>
          <span><i class="is-deleted"></i>删除 {{ statusCounts.deleted }}</span>
          <span v-if="statusCounts.renamed"><i class="is-renamed"></i>重命名 {{ statusCounts.renamed }}</span>
        </div>
      </article>
      <article class="dt-fact-card">
        <span class="dt-card-label">准备保存</span>
        <strong>{{ selectedCount }} 个文件</strong>
        <p>仍有 {{ unselectedCount }} 个文件未包含</p>
      </article>
      <article class="dt-fact-card">
        <span class="dt-card-label">项目文件</span>
        <strong>{{ files.filter((file) => file.status !== 'deleted').length }} 个</strong>
        <p>{{ changedFiles.length ? '工作区尚未干净' : '工作区干净' }}</p>
      </article>
      <article class="dt-fact-card">
        <span class="dt-card-label">最近保存</span>
        <strong>{{ project.lastCommit || '暂无' }}</strong>
        <p>{{ project.lastCommitSubject }}</p>
        <small>{{ project.lastCommitAt }}</small>
      </article>
    </div>

    <div class="dt-recent-section">
      <div class="dt-section-title"><div><h2>最近变化</h2><p>按最近修改时间排列</p></div><span>{{ changedFiles.length }} 个</span></div>
      <div v-if="changedFiles.length" class="dt-recent-list">
        <button v-for="file in changedFiles.slice(0, 5)" :key="file.path" type="button" @click="$emit('showDiff', file.path)">
          <span class="dt-status-mark" :class="`is-${file.status}`">{{ statusLetter(file.status) }}</span>
          <span><strong>{{ file.name }}</strong><small>{{ file.path }}</small></span>
          <span class="dt-line-stats"><b>+{{ file.additions }}</b><em>-{{ file.deletions }}</em></span>
          <IconChevronRight />
        </button>
      </div>
      <div v-else class="dt-empty-compact"><IconCircleCheck /><strong>项目已处于稳定状态</strong><span>没有需要处理的文件变化。</span></div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { IconArrowRight, IconChevronRight, IconCircleCheck, IconGitBranch, IconGitCommit } from '@tabler/icons-vue'
import type { ChangeStatus, MockProjectFile } from '../mock/types'

defineProps<{
  project: { lastCommit: string; lastCommitSubject: string; lastCommitAt: string }
  files: MockProjectFile[]
  changedFiles: MockProjectFile[]
  selectedCount: number
  unselectedCount: number
  statusCounts: { added: number; modified: number; deleted: number; renamed: number }
  isGitRepository: boolean
  hasFirstCommit: boolean
  initializingGit: boolean
}>()

defineEmits<{ openChanges: []; showDiff: [path: string]; initGit: [] }>()

function statusLetter(status: ChangeStatus) {
  return ({ modified: 'M', added: 'A', deleted: 'D', renamed: 'R', clean: '—' })[status]
}
</script>
