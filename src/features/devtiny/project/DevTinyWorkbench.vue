<template>
  <div class="dt-app-shell">
    <template v-if="!state.projectOpen">
      <main class="dt-project-welcome">
        <div class="dt-welcome-brand"><span>DT</span><strong>DevTiny</strong></div>
        <div class="dt-welcome-copy">
          <span class="dt-eyebrow">本地项目事实视图</span>
          <h1>看见当前，理解变化，<br />建立新的稳定点。</h1>
          <p>从文件出发，清楚了解上一次保存之后发生了什么。</p>
        </div>
        <button class="dt-open-project" type="button" :class="{ 'is-unavailable': !isTauriRuntime }" @click="chooseRealProject">
          <span><IconFolderOpen /></span>
          <span><strong>选择本地项目</strong><small>{{ isTauriRuntime ? '读取真实文件与 Git 状态' : '需要在 Tauri 桌面窗口中运行' }}</small></span>
          <IconArrowRight />
        </button>
        <button v-if="isDev" class="dt-button dt-button-secondary dt-mock-entry" type="button" @click="openProject">打开模拟项目进行界面回归</button>
        <p class="dt-mock-note"><IconInfoCircle /> {{ runtimeHint }}</p>
        <p v-if="state.error" class="dt-welcome-error">{{ state.error }}</p>
      </main>
    </template>

    <ChangeFilesFocus
      v-else
      :project-name="state.project.name"
      :project-path="state.project.path"
      :files="state.files"
      :tree="fileTree"
      :changed-files="changedFiles"
      :ignored-files="state.ignoredFiles"
      :selected-count="selectedFiles.length"
      :is-git-repository="state.isGitRepository"
      :has-first-commit="state.hasFirstCommit"
      :real-mode="state.realMode"
      :loading="state.loading"
      :saving="state.saving"
      :commit-message="state.commitMessage"
      @select-file="selectFile"
      @set-selected="setSelected"
      @request-undo="requestUndo"
      @ignore-with-rule="ignoreFile"
      @restore-ignored="restoreIgnored"
      @stage-all="selectAll"
      @unstage-all="clearSelection"
      @switch-project="chooseRealProject"
      @refresh="refreshRealProject"
      @restored="handleHistoryRestored"
      @update-commit-message="state.commitMessage = $event"
      @create-save-point="createSavePoint"
    />

    <div v-if="state.projectOpen && state.error" class="dt-global-message focus-global-message is-error"><IconAlertTriangle /><span><strong>操作未完成</strong>{{ state.error }}</span><button type="button" @click="state.error = ''">关闭</button></div>
    <div v-if="state.projectOpen && state.notice" class="dt-global-message focus-global-message is-success"><IconCircleCheck /><span><strong>已完成</strong>{{ state.notice }}</span><button type="button" @click="state.notice = ''">关闭</button></div>

    <label v-if="state.projectOpen && isDev" class="dt-scenario-switch">
      <span>DEV · Mock 场景</span>
      <select :value="state.scenario" @change="loadScenario(($event.target as HTMLSelectElement).value as ScenarioId)">
        <option v-for="scenario in scenarioOptions" :key="scenario.id" :value="scenario.id">{{ scenario.label }}</option>
      </select>
    </label>

    <div v-if="state.projectOpen && state.undoDialogOpen" class="dt-result-overlay" role="dialog" aria-modal="true" aria-labelledby="undo-title-focus">
      <article class="dt-confirm-dialog">
        <span class="dt-warning-icon"><IconAlertTriangle /></span>
        <div><span class="dt-eyebrow">{{ undoIsAdded ? '可从系统回收站找回' : '这一步不能恢复' }}</span><h2 id="undo-title-focus">{{ undoIsAdded ? `删除 ${undoFileName}？` : `撤销 ${undoFileName} 的修改？` }}</h2></div>
        <p>{{ undoIsAdded ? '这是一个尚未纳入项目的新文件。确认后会将文件移到系统回收站，而不是永久删除。' : '文件会回到最近一次保存时的样子，本次尚未保存的修改会消失。' }}</p>
        <div class="dt-impact-row"><span>将受影响</span><strong>{{ state.undoTargetPath }}</strong></div>
        <div class="dt-dialog-actions"><button class="dt-button dt-button-secondary" type="button" @click="cancelUndo">{{ undoIsAdded ? '保留文件' : '保留修改' }}</button><button class="dt-button dt-button-danger" type="button" @click="confirmUndo">{{ undoIsAdded ? '移到回收站' : '确认撤销' }}</button></div>
      </article>
    </div>

    <template v-if="false">
      <header class="dt-project-header">
        <div class="dt-project-identity">
          <span class="dt-app-mark">DT</span>
          <div><strong>{{ state.project.name }}</strong><span>{{ state.project.path }}</span></div>
        </div>
        <div class="dt-project-facts">
          <div><IconGitBranch /><span>当前分支</span><strong>{{ state.isGitRepository ? state.project.branch : '非 Git 目录' }}</strong></div>
          <div><IconGitCommit /><span>上次保存</span><strong>{{ state.hasFirstCommit ? state.project.lastCommit : '尚无提交' }}</strong></div>
          <div><IconFileDiff /><span>当前变化</span><strong>{{ changedFiles.length }} 个文件</strong></div>
          <div><IconChecklist /><span>准备保存</span><strong>{{ selectedFiles.length }} 个文件</strong></div>
        </div>
        <div class="dt-header-actions"><button v-if="state.realMode" class="dt-project-switch" type="button" @click="refreshRealProject"><IconRefresh /> 刷新</button><button class="dt-project-switch" type="button" @click="chooseRealProject"><IconFolderOpen /> 切换项目</button></div>
      </header>

      <div class="dt-workspace">
        <nav class="dt-navigation" aria-label="一级导航">
          <div class="dt-nav-brand"><strong>DevTiny</strong><span>V0.1 prototype</span></div>
          <button type="button" :class="{ active: state.activeView === 'overview' }" @click="state.activeView = 'overview'"><IconLayoutDashboard /><span><strong>Overview</strong><small>项目当前状态</small></span></button>
          <button type="button" :class="{ active: state.activeView === 'changes' }" @click="state.activeView = 'changes'"><IconGitCompare /><span><strong>Changes</strong><small>理解与组织变化</small></span><em v-if="changedFiles.length">{{ changedFiles.length }}</em></button>
          <button type="button" :class="{ active: state.activeView === 'files' }" @click="state.activeView = 'files'"><IconFiles /><span><strong>Files</strong><small>当前文件事实</small></span></button>
          <div class="dt-nav-status"><span :class="{ clean: !changedFiles.length }"></span><div><strong>{{ changedFiles.length ? '工作区有变化' : '工作区干净' }}</strong><small>{{ state.isGitRepository ? state.project.branch : '未开启版本管理' }}</small></div></div>
        </nav>

        <div class="dt-main-area">
          <div v-if="state.loading" class="dt-loading-state"><span class="dt-loading-mark"></span><strong>正在读取项目…</strong><p>正在整理文件和 Git 变化事实。</p></div>
          <template v-else>
            <div v-if="state.error" class="dt-global-message is-error"><IconAlertTriangle /><span><strong>操作未完成</strong>{{ state.error }}</span><button type="button" @click="state.error = ''">关闭</button></div>
            <div v-if="state.notice" class="dt-global-message is-success"><IconCircleCheck /><span><strong>状态已更新</strong>{{ state.notice }}</span><button type="button" @click="state.notice = ''">关闭</button></div>

            <OverviewView
              v-if="state.activeView === 'overview'"
              :project="state.project"
              :files="state.files"
              :changed-files="changedFiles"
              :selected-count="selectedFiles.length"
              :unselected-count="unselectedFiles.length"
              :status-counts="statusCounts"
              :is-git-repository="state.isGitRepository"
              :has-first-commit="state.hasFirstCommit"
              :initializing-git="state.initializingGit"
              @open-changes="state.activeView = 'changes'"
              @show-diff="showDiff"
              @init-git="initializeGit"
            />
            <ChangesExperience
              v-else-if="state.activeView === 'changes'"
              :changed-files="changedFiles"
              :selected-files="selectedFiles"
              :unselected-files="unselectedFiles"
              :selected-file="selectedFile"
              :selected-part="state.selectedPart"
              :additions="additions"
              :deletions="deletions"
              :commit-message="state.commitMessage"
              :can-save="canSave"
              :saving="state.saving"
              :save-result="state.saveResult"
              :is-git-repository="state.isGitRepository"
              :has-first-commit="state.hasFirstCommit"
              :project-path="state.project.path"
              :real-mode="state.realMode"
              @select-file="selectFile"
              @set-selected="setSelected"
              @select-all="selectAll"
              @clear-selection="clearSelection"
              @request-undo="requestUndo"
              @update-commit-message="state.commitMessage = $event"
              @create-save-point="createSavePoint"
              @go-overview="state.saveResult = null; state.activeView = 'overview'"
              @dismiss-result="state.saveResult = null"
              @restored="handleHistoryRestored"
            />
            <FilesExperience
              v-else
              :tree="fileTree"
              :files="state.files"
              :selected-file="selectedFile"
              :project-path="state.project.path"
              :is-git-repository="state.isGitRepository"
              :real-mode="state.realMode"
              @select-file="selectFile"
              @show-diff="showDiff"
              @restored="handleHistoryRestored"
            />
          </template>
        </div>
      </div>

      <label v-if="isDev" class="dt-scenario-switch">
        <span>DEV · Mock 场景</span>
        <select :value="state.scenario" @change="loadScenario(($event.target as HTMLSelectElement).value as ScenarioId)">
          <option v-for="scenario in scenarioOptions" :key="scenario.id" :value="scenario.id">{{ scenario.label }}</option>
        </select>
      </label>

      <div v-if="state.undoDialogOpen" class="dt-result-overlay" role="dialog" aria-modal="true" aria-labelledby="undo-title">
        <article class="dt-confirm-dialog">
          <span class="dt-warning-icon"><IconAlertTriangle /></span>
          <div><span class="dt-eyebrow">不可恢复的操作</span><h2 id="undo-title">撤销 {{ undoFileName }} 的未保存修改？</h2></div>
          <p>文件将恢复到最近一次提交状态。该修改在 Git 中尚未保存，撤销后无法通过 DevTiny 恢复。</p>
          <div class="dt-impact-row"><span>将受影响</span><strong>{{ state.undoTargetPath }}</strong></div>
          <div class="dt-dialog-actions"><button class="dt-button dt-button-secondary" type="button" @click="cancelUndo">保留修改</button><button class="dt-button dt-button-danger" type="button" @click="confirmUndo">确认撤销修改</button></div>
        </article>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import {
  IconAlertTriangle, IconArrowRight, IconChecklist, IconCircleCheck, IconFileDiff,
  IconFiles, IconFolderOpen, IconGitBranch, IconGitCommit, IconGitCompare,
  IconInfoCircle, IconLayoutDashboard, IconRefresh
} from '@tabler/icons-vue'
import ChangesExperience from '../changes/ChangesExperience.vue'
import ChangeFilesFocus from '../changes/ChangeFilesFocus.vue'
import FilesExperience from '../files/FilesExperience.vue'
import { scenarioOptions, useMockProject } from '../mock/projectMock'
import type { ScenarioId } from '../mock/types'
import OverviewView from '../overview/OverviewView.vue'

const {
  state, changedFiles, selectedFiles, unselectedFiles, selectedFile, additions, deletions,
  statusCounts, fileTree, canSave, openProject, loadRealProject, refreshRealProject, initializeGit, selectFile, showDiff, setSelected,
  selectAll, clearSelection, ignoreFile, restoreIgnored, createSavePoint, requestUndo, cancelUndo, confirmUndo, loadScenario
} = useMockProject()

const isDev = Boolean((import.meta as ImportMeta & { env?: { DEV?: boolean } }).env?.DEV)
const isTauriRuntime = Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__)
const undoFileName = computed(() => state.undoTargetPath.split('/').pop() || state.undoTargetPath)
const undoIsAdded = computed(() => state.files.find((file) => file.path === state.undoTargetPath)?.status === 'added')
const runtimeHint = computed(() => isTauriRuntime
  ? 'Git 写操作会显示明确影响范围；撤销修改不可恢复。'
  : '当前是浏览器预览，只能体验 mock。请运行 pnpm tauri:dev 使用真实本地项目。')
let operationFeedbackTimer: number | null = null

watch(() => [state.notice, state.error] as const, ([notice, error]) => {
  if (operationFeedbackTimer !== null) window.clearTimeout(operationFeedbackTimer)
  if (!notice && !error) return
  operationFeedbackTimer = window.setTimeout(() => {
    state.notice = ''
    state.error = ''
    operationFeedbackTimer = null
  }, 3000)
})

onBeforeUnmount(() => {
  if (operationFeedbackTimer !== null) window.clearTimeout(operationFeedbackTimer)
})

async function chooseRealProject() {
  if (!isTauriRuntime) {
    state.error = '当前是浏览器预览，无法访问本地文件系统。请停止 pnpm dev，改用 pnpm tauri:dev 启动桌面应用；也可以点击下方“打开模拟项目”继续体验界面。'
    return
  }
  try {
    const selected = await open({ directory: true, multiple: false, title: '选择本地项目' })
    if (typeof selected === 'string') await loadRealProject(selected)
  } catch (error) {
    state.error = `无法打开目录选择器：${formatRuntimeError(error)}`
  }
}

async function handleHistoryRestored() {
  await refreshRealProject()
  state.notice = '历史版本已恢复，并作为新的未提交变化保留。'
}

function formatRuntimeError(error: unknown) {
  if (error instanceof Error) return error.message
  if (typeof error === 'object' && error && 'message' in error) return String((error as { message: unknown }).message)
  return String(error)
}
</script>
