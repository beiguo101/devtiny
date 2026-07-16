<template>
  <section class="dt-view dt-changes-view" :class="{ 'is-save-point-mode': selectedSavePoint }">
    <aside class="dt-changes-list">
      <template v-if="selectedSavePoint">
        <div class="dt-save-point-files-head">
          <button type="button" @click="showCurrentChanges"><IconArrowDown /> 当前变化</button>
          <span class="dt-eyebrow">保存点文件</span>
          <strong>{{ selectedSavePoint.subject }}</strong>
          <small><code>{{ selectedSavePoint.shortCommit }}</code> · {{ formatSavePointDate(selectedSavePoint.date) }}</small>
        </div>
        <div v-if="savePointFilesLoading" class="dt-side-empty"><IconLoader2 class="dt-spin" /><strong>正在读取文件…</strong></div>
        <div v-else-if="savePointError" class="dt-side-empty"><IconFileDiff /><strong>无法读取保存点</strong><span>{{ savePointError }}</span></div>
        <div v-else class="dt-save-point-file-list">
          <button
            v-for="file in savePointFiles"
            :key="file.relativePath"
            type="button"
            :class="{ active: selectedSavePointFile?.relativePath === file.relativePath }"
            @click="selectSavePointFile(file)"
          >
            <span class="dt-status-mark" :class="`is-${normalizeSavePointStatus(file.status)}`">{{ statusLetter(normalizeSavePointStatus(file.status)) }}</span>
            <span class="dt-change-name"><strong>{{ fileName(file.relativePath) }}</strong><small>{{ file.oldRelativePath ? `${file.oldRelativePath} → ${file.relativePath}` : file.relativePath }}</small></span>
            <IconChevronRight />
          </button>
        </div>
      </template>
      <template v-else>
      <div class="dt-changes-summary">
        <div><span class="dt-eyebrow">工作区变化</span><strong>{{ changedFiles.length }}</strong><span>个变化文件</span></div>
        <div class="dt-mini-stats"><span>+{{ additions }}</span><span>−{{ deletions }}</span></div>
      </div>
      <div class="dt-list-actions">
        <button type="button" :disabled="!unselectedFiles.length" @click="$emit('selectAll')">全选</button>
        <button type="button" :disabled="!selectedFiles.length" @click="$emit('clearSelection')">清除选择</button>
      </div>

      <div v-if="!isGitRepository" class="dt-side-empty"><IconGitBranch /><strong>没有 Git 变化</strong><span>当前目录不是 Git 仓库。</span></div>
      <div v-else-if="!hasFirstCommit" class="dt-side-empty"><IconGitCommit /><strong>等待首次保存</strong><span>仓库还没有可比较的稳定点。</span></div>
      <div v-else-if="!changedFiles.length" class="dt-side-empty"><IconCircleCheck /><strong>工作区干净</strong><span>没有尚未保存的变化。</span></div>
      <div v-else class="dt-change-groups">
        <section>
          <div class="dt-group-heading"><div><span class="dt-selection-dot is-selected"></span><strong>已选择保存</strong></div><span>{{ selectedFiles.length }}</span></div>
          <p class="dt-group-help">对应 Git staged，将包含在下一个保存点中。</p>
          <div class="dt-file-change-list">
            <article
              v-for="file in selectedFiles"
              :key="`${file.path}-selected`"
              class="dt-change-item"
              :class="{ active: selectedFile?.path === file.path && selectedPart === 'selected' }"
            >
              <button class="dt-change-main" type="button" @click="$emit('selectFile', file.path, 'selected')">
                <span class="dt-status-mark" :class="`is-${file.status}`">{{ statusLetter(file.status) }}</span>
                <span class="dt-change-name"><strong>{{ file.name }}</strong><small>{{ file.path }}</small></span>
                <span class="dt-line-stats"><b>+{{ file.additions }}</b><em>-{{ file.deletions }}</em></span>
              </button>
              <button class="dt-inline-action" type="button" @click="$emit('setSelected', file.path, false)">移出</button>
            </article>
          </div>
        </section>

        <section>
          <div class="dt-group-heading"><div><span class="dt-selection-dot"></span><strong>尚未选择</strong></div><span>{{ unselectedFiles.length }}</span></div>
          <p class="dt-group-help">这些变化不会包含在本次保存中。</p>
          <div class="dt-file-change-list">
            <article
              v-for="file in unselectedFiles"
              :key="`${file.path}-unselected`"
              class="dt-change-item"
              :class="{ active: selectedFile?.path === file.path && selectedPart === 'unselected' }"
            >
              <button class="dt-change-main" type="button" @click="$emit('selectFile', file.path, 'unselected')">
                <span class="dt-status-mark" :class="`is-${file.status}`">{{ statusLetter(file.status) }}</span>
                <span class="dt-change-name"><strong>{{ file.name }}</strong><small>{{ file.oldPath ? `${file.oldPath} → ${file.path}` : file.path }}</small></span>
                <span class="dt-line-stats"><b>+{{ file.additions }}</b><em>-{{ file.deletions }}</em></span>
              </button>
              <div class="dt-item-actions">
                <button class="dt-inline-action" type="button" @click="$emit('setSelected', file.path, true)">加入</button>
                <button v-if="file.status === 'modified'" class="dt-inline-action is-danger" type="button" @click="$emit('requestUndo', file.path)">撤销</button>
              </div>
            </article>
          </div>
        </section>
      </div>
      </template>
    </aside>

    <main class="dt-diff-pane" :class="{ 'is-revision-mode': selectedRevision }">
      <template v-if="selectedSavePoint">
        <template v-if="savePointPreview">
          <header class="dt-diff-heading">
            <div>
              <span class="dt-eyebrow">保存点中的文件变化</span>
              <h1>{{ savePointPreview.name }}</h1>
              <p>{{ savePointPreview.path }} · {{ selectedSavePoint.subject }} · {{ selectedSavePoint.shortCommit }}</p>
            </div>
            <button class="dt-button dt-button-secondary" type="button" @click="showCurrentChanges"><IconArrowDown /> 查看当前变化</button>
          </header>
          <StructuredDiffViewer :file="savePointPreview" />
        </template>
        <div v-else class="dt-empty-large"><IconFileDiff /><strong>选择保存点中的文件</strong><span>文件列表位于左侧；选择后在这里阅读该保存点的具体差异。</span></div>
      </template>
      <template v-else-if="selectedFile && selectedFile.status !== 'clean'">
        <FileRevisionContent
          v-if="selectedRevision"
          :project-path="projectPath"
          :relative-path="selectedFile.path"
          :entry="selectedRevision"
          @close="selectedRevision = null"
          @restored="handleRevisionRestored"
        />
        <template v-else>
        <header class="dt-diff-heading">
          <div>
            <span class="dt-eyebrow">{{ selectedPart === 'selected' ? '已选择保存的变化' : '尚未选择的变化' }}</span>
            <h1>{{ selectedFile.name }}</h1>
            <p>{{ selectedFile.path }} · {{ statusLabel(selectedFile.status) }}</p>
          </div>
          <button
            class="dt-button"
            :class="selectedPart === 'selected' ? 'dt-button-secondary' : 'dt-button-primary'"
            type="button"
            @click="$emit('setSelected', selectedFile.path, selectedPart !== 'selected')"
          >
            {{ selectedPart === 'selected' ? '移出本次保存' : '加入本次保存' }}
          </button>
        </header>
        <StructuredDiffViewer :file="selectedFile" />
        </template>
        <FileRevisionHistory
          :project-path="projectPath"
          :relative-path="selectedFile.path"
          :is-git-repository="isGitRepository"
          :real-mode="realMode"
          :selected-commit="selectedRevision?.commit"
          @select-revision="selectedRevision = $event"
        />
      </template>
      <div v-else class="dt-empty-large"><IconFileDiff /><strong>选择一个变化文件</strong><span>在同一屏幕阅读具体变化，并决定是否包含在本次保存中。</span></div>
    </main>

    <footer v-if="!selectedSavePoint" class="dt-save-bar">
      <div class="dt-save-selection">
        <span class="dt-selection-count">{{ selectedFiles.length }}</span>
        <div><strong>个文件准备保存</strong><span>{{ selectedSummary }}</span></div>
      </div>
      <label class="dt-commit-field">
        <span>提交说明</span>
        <input :value="commitMessage" type="text" placeholder="例如：feat: improve project change view" @input="$emit('updateCommitMessage', ($event.target as HTMLInputElement).value)" />
      </label>
      <div class="dt-save-action">
        <button class="dt-button dt-button-primary" type="button" :disabled="!canSave" @click="$emit('createSavePoint')">
          <IconLoader2 v-if="saving" class="dt-spin" />
          <IconGitCommit v-else />
          {{ saving ? '正在创建…' : '创建保存点' }}
        </button>
        <small>{{ saveDisabledReason }}</small>
      </div>
    </footer>

    <SavePointDrawer
      :entries="savePoints"
      :selected="selectedSavePoint"
      :has-more="savePointsHasMore"
      :loading="savePointsLoading"
      :error="savePointsError"
      @select="selectSavePoint"
      @load-more="loadMoreSavePoints"
      @show-current="showCurrentChanges"
    />

    <div v-if="saveResult" class="dt-result-overlay" role="dialog" aria-modal="true" aria-labelledby="save-result-title">
      <article class="dt-result-dialog">
        <span class="dt-result-icon"><IconCheck /></span>
        <span class="dt-eyebrow">操作完成</span>
        <h2 id="save-result-title">已建立新的保存点</h2>
        <div class="dt-result-message"><span>提交说明</span><strong>{{ saveResult.message }}</strong></div>
        <dl>
          <div><dt>包含</dt><dd>{{ saveResult.fileCount }} 个文件</dd></div>
          <div><dt>代码变化</dt><dd><b>+{{ saveResult.additions }}</b> / <em>-{{ saveResult.deletions }}</em></dd></div>
          <div><dt>仍有</dt><dd>{{ saveResult.remainingCount }} 个未保存变化</dd></div>
        </dl>
        <div class="dt-dialog-actions">
          <button class="dt-button dt-button-secondary" type="button" @click="$emit('goOverview')">返回项目概览</button>
          <button class="dt-button dt-button-primary" type="button" @click="$emit('dismissResult')">查看剩余变化</button>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { IconArrowDown, IconCheck, IconChevronRight, IconCircleCheck, IconFileDiff, IconGitBranch, IconGitCommit, IconLoader2 } from '@tabler/icons-vue'
import type { GitFileStatus } from '../../../core/commands/types'
import type { ChangeStatus, MockProjectFile, SaveResult } from '../mock/types'
import StructuredDiffViewer from './StructuredDiffViewer.vue'
import SavePointDrawer from './SavePointDrawer.vue'
import FileRevisionHistory from '../history/FileRevisionHistory.vue'
import FileRevisionContent from '../history/FileRevisionContent.vue'
import { getSavePointFileDiff, listSavePointFiles, listSavePoints } from './api'
import { parseUnifiedDiff } from './diffParser'
import type { FileHistoryEntry, SavePointFile } from './types'

const props = defineProps<{
  changedFiles: MockProjectFile[]
  selectedFiles: MockProjectFile[]
  unselectedFiles: MockProjectFile[]
  selectedFile: MockProjectFile | null
  selectedPart: 'selected' | 'unselected'
  additions: number
  deletions: number
  commitMessage: string
  canSave: boolean
  saving: boolean
  saveResult: SaveResult | null
  isGitRepository: boolean
  hasFirstCommit: boolean
  projectPath: string
  realMode: boolean
}>()
const selectedRevision = ref<FileHistoryEntry | null>(null)
const savePoints = ref<FileHistoryEntry[]>([])
const savePointsHasMore = ref(false)
const savePointsLoading = ref(false)
const savePointsError = ref('')
const selectedSavePoint = ref<FileHistoryEntry | null>(null)
const savePointFiles = ref<SavePointFile[]>([])
const savePointFilesLoading = ref(false)
const selectedSavePointFile = ref<SavePointFile | null>(null)
const savePointPreview = ref<MockProjectFile | null>(null)
const savePointError = ref('')

const emit = defineEmits<{
  selectFile: [path: string, part: 'selected' | 'unselected']
  setSelected: [path: string, selected: boolean]
  selectAll: []
  clearSelection: []
  requestUndo: [path: string]
  updateCommitMessage: [message: string]
  createSavePoint: []
  goOverview: []
  dismissResult: []
  restored: []
}>()

watch(() => props.selectedFile?.path, () => { selectedRevision.value = null })
watch(() => [props.projectPath, props.isGitRepository, props.hasFirstCommit] as const, () => {
  showCurrentChanges()
  savePoints.value = []
  void loadMoreSavePoints()
})
watch(() => props.saveResult, (result) => {
  if (!result) return
  savePoints.value = []
  showCurrentChanges()
  void loadMoreSavePoints()
})

onMounted(() => { void loadMoreSavePoints() })

function handleRevisionRestored() {
  selectedRevision.value = null
  emit('restored')
}

async function loadMoreSavePoints() {
  if (!props.isGitRepository || !props.hasFirstCommit || savePointsLoading.value) return
  savePointsLoading.value = true
  savePointsError.value = ''
  try {
    const page = props.realMode
      ? await listSavePoints(props.projectPath, savePoints.value.length, 3)
      : mockSavePointPage(savePoints.value.length)
    savePoints.value.push(...page.entries)
    savePointsHasMore.value = page.hasMore
  } catch (error) {
    savePointsError.value = formatError(error)
  } finally {
    savePointsLoading.value = false
  }
}

async function selectSavePoint(entry: FileHistoryEntry) {
  if (selectedSavePoint.value?.commit === entry.commit) {
    showCurrentChanges()
    return
  }
  selectedSavePoint.value = entry
  selectedRevision.value = null
  savePointFiles.value = []
  selectedSavePointFile.value = null
  savePointPreview.value = null
  savePointError.value = ''
  savePointFilesLoading.value = true
  try {
    savePointFiles.value = props.realMode ? await listSavePointFiles(props.projectPath, entry.commit) : mockSavePointFiles(entry.commit)
    if (savePointFiles.value[0]) await selectSavePointFile(savePointFiles.value[0])
  } catch (error) {
    savePointError.value = formatError(error)
  } finally {
    savePointFilesLoading.value = false
  }
}

async function selectSavePointFile(file: SavePointFile) {
  if (!selectedSavePoint.value) return
  selectedSavePointFile.value = file
  savePointError.value = ''
  const status = normalizeSavePointStatus(file.status)
  const preview: MockProjectFile = {
    path: file.relativePath,
    oldPath: file.oldRelativePath,
    name: fileName(file.relativePath),
    status,
    size: '—', modifiedAt: selectedSavePoint.value.date, language: fileLanguage(file.relativePath), content: '',
    additions: 0, deletions: 0, hasSelectedChanges: false, hasUnselectedChanges: false, diff: []
  }
  savePointPreview.value = preview
  try {
    const result = props.realMode
      ? await getSavePointFileDiff(props.projectPath, file.relativePath, selectedSavePoint.value.commit)
      : { relativePath: file.relativePath, diff: mockSavePointDiff(file) }
    preview.diff = parseUnifiedDiff(result.diff)
    preview.additions = preview.diff.filter((line) => line.kind === 'added').length
    preview.deletions = preview.diff.filter((line) => line.kind === 'deleted').length
  } catch (error) {
    preview.previewUnavailable = formatError(error)
  }
}

function showCurrentChanges() {
  selectedSavePoint.value = null
  selectedSavePointFile.value = null
  savePointPreview.value = null
  savePointFiles.value = []
  savePointError.value = ''
}

function normalizeSavePointStatus(status: GitFileStatus): ChangeStatus {
  if (status === 'added' || status === 'untracked') return 'added'
  if (status === 'deleted') return 'deleted'
  if (status === 'renamed') return 'renamed'
  return 'modified'
}

function fileName(path: string) { return path.split('/').pop() || path }
function fileLanguage(path: string) { return path.split('.').pop()?.toUpperCase() || 'Text' }
function formatSavePointDate(value: string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleString('zh-CN') }
function formatError(error: unknown) { return error instanceof Error ? error.message : String(error) }

function mockSavePointPage(skip: number) {
  const all = [
    ['a61d723', '完善 Changes 保存点交互', '2026-07-13T15:32:00+08:00'],
    ['9cb8e41', '接入文件历史真实能力', '2026-07-13T12:18:00+08:00'],
    ['74bf210', '支持 Git 初始化和忽略规则', '2026-07-12T18:46:00+08:00'],
    ['51a723c', '优化文件差异展示', '2026-07-12T14:20:00+08:00'],
    ['38d9f12', '完成项目 Overview', '2026-07-11T20:08:00+08:00'],
    ['20c1ab8', '建立 DevTiny 工作台', '2026-07-11T10:15:00+08:00'],
    ['18b9e42', '初始化桌面项目', '2026-07-10T16:30:00+08:00']
  ]
  const entries = all.slice(skip, skip + 3).map(([shortCommit, subject, date]) => ({ commit: shortCommit.padEnd(40, '0'), shortCommit, subject, date }))
  return { entries, hasMore: skip + entries.length < all.length }
}

function mockSavePointFiles(_commit: string): SavePointFile[] {
  return [
    { relativePath: 'src/features/devtiny/changes/ChangesExperience.vue', status: 'modified' },
    { relativePath: 'src/styles.css', status: 'modified' },
    { relativePath: 'src-tauri/src/git/git_file_history.rs', status: 'added' }
  ]
}

function mockSavePointDiff(file: SavePointFile) {
  return `diff --git a/${file.relativePath} b/${file.relativePath}\n--- a/${file.relativePath}\n+++ b/${file.relativePath}\n@@ -18,2 +18,4 @@\n-  const visible = history.slice(0, 3)\n+  const visible = savePoints.slice(0, visibleCount)\n+  const loadMore = () => visibleCount += 3\n   return visible\n`
}

const selectedSummary = computed(() => props.selectedFiles.length ? props.selectedFiles.map((file) => file.name).slice(0, 3).join('、') + (props.selectedFiles.length > 3 ? ` 等 ${props.selectedFiles.length} 个` : '') : '尚未选择任何文件')
const saveDisabledReason = computed(() => {
  if (props.saving) return '将在本地创建一次 Git commit（当前为模拟操作）'
  if (!props.selectedFiles.length) return '请先选择至少一个文件'
  if (!props.commitMessage.trim()) return '请填写提交说明'
  return '将在本地创建一次 Git commit（当前为模拟操作）'
})

function statusLabel(status: ChangeStatus) { return ({ clean: '无变化', modified: '修改', added: '新增', deleted: '删除', renamed: '重命名' })[status] }
function statusLetter(status: ChangeStatus) { return ({ clean: '—', modified: 'M', added: 'A', deleted: 'D', renamed: 'R' })[status] }
</script>
