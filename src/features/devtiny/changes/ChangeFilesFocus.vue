<template>
  <section class="focus-shell" :class="{ 'left-hidden': !leftVisible, 'right-hidden': !rightVisible }">
    <header class="focus-topbar">
      <div class="focus-brand"><span>D</span><strong>DevTiny</strong></div>
      <div class="focus-project-wrap">
        <button class="focus-project" type="button" @click="leaveIgnoredView(); $emit('switchProject')">
          <IconFolder />
          <span><small>当前文件夹</small><strong>{{ projectPath }}</strong></span>
          <IconChevronDown />
        </button>
        <button class="focus-project-refresh" type="button" aria-label="刷新当前文件夹" title="刷新当前文件夹" @click="leaveIgnoredView(); $emit('refresh')"><IconRefresh /></button>
        <button class="focus-project-ignored" type="button" :class="{ active: showIgnored }" :aria-label="`查看已忽略文件，${ignoredFiles.length} 个`" :title="`已忽略文件 · ${ignoredFiles.length}`" @click="toggleIgnoredView">
          <IconEyeOff /><span v-if="ignoredFiles.length">{{ ignoredFiles.length }}</span>
        </button>
      </div>
      <div class="focus-top-actions">
        <button type="button" :class="{ active: leftVisible }" :aria-label="leftVisible ? '隐藏文件树' : '显示文件树'" :title="leftVisible ? '隐藏文件树' : '显示文件树'" @click="leaveIgnoredView(); leftVisible = !leftVisible"><IconLayoutSidebarLeftCollapse /></button>
        <button type="button" :class="{ active: rightVisible }" :aria-label="rightVisible ? '隐藏保存点' : '显示保存点'" :title="rightVisible ? '隐藏保存点' : '显示保存点'" @click="leaveIgnoredView(); rightVisible = !rightVisible"><IconLayoutSidebarRightCollapse /></button>
      </div>
    </header>

    <aside v-if="leftVisible" class="focus-tree-panel">
      <header><div><span>文件</span><strong>{{ projectName }}</strong></div><small>{{ files.length }} 个</small></header>
      <div class="focus-tree-scroll">
        <MockFileTreeNode v-for="entry in tree" :key="entry.id" :entry="entry" :selected-path="detailPath" @select="selectFromTree" />
      </div>
    </aside>

    <main class="focus-change-stage" :class="{ 'detail-open': detailFile && detailMode === 'content' }" @click.self="closeDetail">
      <header v-if="!detailFile || detailMode === 'change'" class="focus-stage-heading">
        <div>
          <span class="focus-kicker"><i></i>{{ selectedSavePoint ? '过去的变化' : '现在需要处理的内容' }}</span>
          <h1>{{ selectedSavePoint ? selectedSavePoint.subject : showIgnored ? '已忽略文件' : '变化文件' }}</h1>
          <p>{{ selectedSavePoint ? `${formatDate(selectedSavePoint.date)} 保存 · ${displayFiles.length} 个文件` : showIgnored ? `${ignoredFiles.length} 个文件不会出现在变化列表中。` : `${displayFiles.length} 个文件发生了变化，点开文件查看它改了什么。` }}</p>
        </div>
      </header>

      <section v-if="detailFile && detailMode === 'content'" class="focus-inline-detail" @click="closeOnDetailBlank">
        <header>
          <div class="focus-detail-title">
            <span class="focus-file-icon"><component :is="fileIcon(detailFile)" /></span>
            <span><small>当前文件内容</small><strong>{{ detailFile.name }}</strong><em>{{ detailFile.path }}</em></span>
          </div>
          <div class="focus-detail-head-actions"><button type="button" aria-label="关闭文件" @click="closeDetail"><IconX /></button></div>
        </header>

        <div v-if="detailBusy" class="focus-detail-loading"><span class="focus-spinner"></span><strong>正在读取文件…</strong></div>
        <CodeViewer v-else-if="!detailFile.binary" :text="detailFile.content" :relative-path="detailFile.path" mode="content" :editable="false" @click.self="closeDetail" />
        <div v-else class="focus-empty"><IconPhoto /><strong>这个文件不能直接预览</strong><span>当前版本是二进制文件。</span></div>
        <p v-if="detailMessage" class="focus-detail-feedback">{{ detailMessage }}</p>
      </section>

      <div v-else-if="loading" class="focus-empty"><span class="focus-spinner"></span><strong>正在整理变化文件…</strong></div>
      <div v-else-if="showIgnored && !ignoredFiles.length" class="focus-empty"><IconEyeOff /><strong>还没有忽略文件</strong><span>忽略的新文件会集中显示在这里。</span></div>
      <div v-else-if="!showIgnored && !displayFiles.length" class="focus-empty"><IconCircleCheck /><strong>{{ selectedSavePoint ? '这个保存点没有文件变化' : '所有文件都已处理' }}</strong><span>{{ selectedSavePoint ? '请选择右侧其他保存点。' : '这里很安静，你可以放心继续工作。' }}</span></div>
      <div v-else-if="showIgnored" class="focus-ignored-list">
        <article v-for="file in ignoredFiles" :key="`ignored-${file.path}`">
          <span class="focus-ignored-icon"><IconFileOff /></span>
          <span><strong>{{ file.name }}</strong><small>{{ file.path }}</small></span>
          <span class="focus-ignore-rule">{{ file.ignoreRule || '仅此文件' }}</span>
          <button type="button" title="停止忽略" @click="$emit('restoreIgnored', file.path)"><IconEye />停止忽略</button>
        </article>
      </div>
      <div v-else class="focus-file-grid">
        <article
          v-for="file in displayFiles"
          :key="`${selectedSavePoint?.commit || 'current'}-${file.path}`"
          class="focus-file-card"
          :class="[`is-${file.status}`, { staged: file.hasSelectedChanges }]"
        >
          <button class="focus-card-main" type="button" @click="openFileDetail(file)">
            <span class="focus-file-icon"><component :is="fileIcon(file)" /></span>
            <span class="focus-file-copy"><strong><span>{{ fileBaseName(file.name) }}</span><em v-if="fileExtension(file.name)" class="focus-file-extension">{{ fileExtension(file.name) }}</em></strong><small>{{ file.path }}</small></span>
            <span class="focus-status-pill">{{ statusText(file.status) }}</span>
            <span class="focus-change-size" v-if="file.additions || file.deletions"><b>增加 {{ file.additions }}</b><em>减少 {{ file.deletions }}</em></span>
            <span v-else class="focus-change-size is-quiet">点开查看变化</span>
          </button>

          <div class="focus-card-action-trigger" :class="{ 'is-open': hoverActionPath === file.path }" @mouseleave="scheduleCloseActions(file.path)">
            <span v-if="hoverActionPath === file.path" class="focus-action-safe-zone" aria-hidden="true"></span>
            <button class="focus-card-more" type="button" :aria-label="`${file.name} 的文件操作`" @mouseenter="openActions(file.path)" @focus="openActions(file.path)" @click.stop="openActions(file.path)"><IconDots /></button>
            <div class="focus-fan-actions" :class="{ 'is-history': selectedSavePoint }">
              <template v-if="selectedSavePoint">
                <button class="is-save-as" type="button" title="另存为" @click.stop="saveSavedFile(file)"><IconDownload /><span>另存为</span></button>
                <button class="is-restore" type="button" title="恢复此记录" @click.stop="restoreSavedFile(file)"><IconHistoryToggle /><span>恢复</span></button>
              </template>
              <template v-else>
                <button class="is-stage" type="button" :title="file.hasSelectedChanges ? '移出本次保存' : '加入本次保存'" @click.stop="stageFile(file)"><IconBookmark /><span>{{ file.hasSelectedChanges ? '移出' : '加入保存' }}</span></button>
                <button class="is-undo" type="button" :disabled="!canUndo(file)" :title="file.status === 'added' ? '移到回收站' : '撤销修改'" @click.stop="undoFile(file)"><component :is="file.status === 'added' ? IconTrash : IconRotate2" /><span>{{ file.status === 'added' ? '删除' : '撤销' }}</span></button>
                <button class="is-ignore" type="button" :disabled="file.status !== 'added'" title="忽略：以后不再提示" @click.stop="ignoreFile(file)"><IconEyeOff /><span>忽略</span></button>
              </template>
            </div>
          </div>
          <span v-if="file.hasSelectedChanges && !selectedSavePoint" class="focus-staged-mark"><IconBookmarkFilled />已加入本次保存</span>
        </article>
      </div>
    </main>

    <aside v-if="rightVisible" class="focus-time-panel">
      <header><div><span>查看时间</span><strong>当前与最近保存</strong></div><IconClock /></header>
      <div class="focus-time-list">
        <button type="button" :class="{ active: !selectedSavePoint }" @click="showCurrent">
          <span class="focus-time-dot is-now"><IconSparkles /></span>
          <span><strong>当前变化</strong><small>{{ changedFiles.length }} 个文件等待处理</small></span>
          <IconChevronRight />
        </button>
        <div class="focus-time-caption">最近保存点</div>
        <button v-for="entry in savePoints" :key="entry.commit" type="button" :class="{ active: selectedSavePoint?.commit === entry.commit }" @click="selectSavePoint(entry)">
          <span class="focus-time-dot"><IconCircleCheck /></span>
          <span><strong>{{ entry.subject }}</strong><small>{{ formatDate(entry.date) }}</small></span>
          <IconChevronRight />
        </button>
        <button v-if="savePointsHasMore" class="focus-time-more" type="button" :disabled="savePointsLoading" @click="loadMoreSavePoints"><IconPlus />{{ savePointsLoading ? '正在加载…' : '再看 3 个' }}</button>
      </div>
      <section v-if="!selectedSavePoint" class="focus-action-dock">
        <div class="focus-save-composite" :class="{ 'is-batch-open': batchRevealOpen }" @mouseleave="batchRevealOpen = false">
          <button class="focus-save-entry" type="button" :disabled="!selectedCount" @click="openSaveDialog"><IconBookmarkFilled /><span><strong>{{ selectedCount }} 个文件已暂存</strong><small>{{ selectedCount ? '新增保存点' : '暂无可保存文件' }}</small></span></button>
          <button class="focus-batch-handle" type="button" aria-label="展开批量暂存操作" title="批量暂存操作" @mouseenter="batchRevealOpen = true" @focus="batchRevealOpen = true" @click="batchRevealOpen = true"><IconCode /></button>
          <div class="focus-batch-options" @focusin="batchRevealOpen = true">
            <button type="button" :disabled="!unstagedCount" title="全部暂存" @click="runBatch(true)"><IconBookmark /><span>全部暂存</span></button>
            <button type="button" :disabled="!selectedCount" title="全部取消暂存" @click="runBatch(false)"><IconBookmarkOff /><span>全部取消暂存</span></button>
          </div>
        </div>
      </section>
    </aside>

    <div v-if="detailFile && detailMode === 'change'" class="focus-change-overlay" @click.self="closeDetail">
      <article class="focus-change-dialog">
        <header>
          <div class="focus-detail-title"><span class="focus-file-icon"><component :is="fileIcon(detailFile)" /></span><span><small>{{ selectedSavePoint ? `保存点 · ${selectedSavePoint.shortCommit}` : '当前变化' }}</small><strong>{{ detailFile.name }}</strong><em>{{ detailFile.path }}</em></span></div>
          <div class="focus-detail-head-actions"><span class="focus-detail-status">{{ statusText(detailFile.status) }}</span><button type="button" aria-label="关闭文件变化" @click="closeDetail"><IconX /></button></div>
        </header>
        <div v-if="detailBusy" class="focus-detail-loading"><span class="focus-spinner"></span><strong>正在读取文件…</strong></div>
        <StructuredDiffViewer v-else :file="detailFile" />
        <footer class="focus-detail-actions">
          <template v-if="selectedSavePoint">
            <button type="button" :disabled="detailBusy" @click="saveSavedFile(detailFile)"><IconDownload />另存为</button>
            <button class="is-primary" type="button" :disabled="detailBusy" @click="restoreSavedFile(detailFile)"><IconHistoryToggle />恢复此记录</button>
          </template>
          <template v-else>
            <button type="button" @click="stageFile(detailFile)"><IconBookmark />{{ detailFile.hasSelectedChanges ? '移出本次保存' : '加入本次保存' }}</button>
            <button type="button" :disabled="!canUndo(detailFile)" @click="undoFile(detailFile)"><component :is="detailFile.status === 'added' ? IconTrash : IconRotate2" />{{ detailFile.status === 'added' ? '移到回收站' : '撤销' }}</button>
            <button type="button" :disabled="detailFile.status !== 'added'" @click="openIgnoreDialog(detailFile)"><IconEyeOff />忽略</button>
          </template>
        </footer>
        <p v-if="detailMessage" class="focus-detail-feedback">{{ detailMessage }}</p>
      </article>
    </div>

    <div v-if="saveDialogOpen" class="focus-save-overlay" @click.self="closeSaveDialog">
      <form class="focus-save-dialog" @submit.prevent="submitSavePoint">
        <header><div><small>建立一个新的保存点</small><h2>保存这 {{ selectedCount }} 个文件</h2><p>写一句简单说明，方便以后找回这次变化。</p></div><button type="button" aria-label="关闭保存点窗口" @click="closeSaveDialog"><IconX /></button></header>
        <div class="focus-save-files">
          <span>本次包含</span>
          <ul>
            <li v-for="file in stagedFiles" :key="file.path">
              <component :is="fileIcon(file)" />
              <span><strong>{{ file.name }}</strong><small>{{ file.path }}</small></span>
              <button class="focus-save-file-remove" type="button" :aria-label="`将 ${file.name} 移出本次保存`" :title="`移出本次保存 · ${file.name}`" @click="removeFromSave(file)"><IconX /></button>
            </li>
          </ul>
        </div>
        <label class="focus-save-message"><span>这次做了什么</span><textarea v-model="saveMessage" rows="3" maxlength="120" autofocus placeholder="例如：完善文件变化查看方式"></textarea><small>{{ saveMessage.trim().length }}/120</small></label>
        <footer><button type="button" @click="closeSaveDialog">取消</button><button class="is-primary" type="submit" :disabled="!saveMessage.trim() || saving"><span v-if="saving" class="focus-button-spinner"></span>{{ saving ? '正在保存…' : '确定保存' }}</button></footer>
      </form>
    </div>

    <div v-if="ignoreDialogOpen && ignoreTarget" class="focus-save-overlay" @click.self="closeIgnoreDialog">
      <form class="focus-ignore-dialog" @submit.prevent="submitIgnore">
        <header><span class="focus-ignore-dialog-icon"><IconEyeOff /></span><div><small>从变化列表中隐藏</small><h2>忽略 {{ ignoreTarget.name }}</h2><p>选择规则范围，之后可以在“已忽略”中查看。</p></div><button type="button" aria-label="关闭忽略窗口" @click="closeIgnoreDialog"><IconX /></button></header>
        <div class="focus-ignore-options">
          <label :class="{ active: ignoreMode === 'file' }"><input v-model="ignoreMode" type="radio" value="file" /><span><strong>只忽略这个文件</strong><small>{{ ignoreTarget.path }}</small></span><IconCircleCheck /></label>
          <label :class="{ active: ignoreMode === 'extension' }"><input v-model="ignoreMode" type="radio" value="extension" /><span><strong>忽略所有同后缀文件</strong><small>匹配 {{ ignoreExtensionLabel }}</small></span><IconCircleCheck /></label>
        </div>
        <footer><button type="button" @click="closeIgnoreDialog">取消</button><button class="is-primary" type="submit">确认忽略</button></footer>
      </form>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, markRaw, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { confirm, save } from '@tauri-apps/plugin-dialog'
import {
  IconBookmark, IconBookmarkFilled, IconChevronDown, IconChevronRight, IconCircleCheck,
  IconBookmarkOff, IconClock, IconDots, IconEye, IconEyeOff, IconFile, IconFileCode, IconFileDescription, IconFileOff, IconFolder,
  IconLayoutSidebarLeftCollapse, IconLayoutSidebarRightCollapse, IconPhoto, IconPlus, IconRefresh, IconRotate2,
  IconSettings, IconSparkles, IconDownload, IconHistoryToggle, IconCode, IconTrash, IconX
} from '@tabler/icons-vue'
import type { GitFileStatus } from '../../../core/commands/types'
import MockFileTreeNode from '../files/MockFileTreeNode.vue'
import CodeViewer from '../files/CodeViewer.vue'
import type { ChangeStatus, FileTreeEntry, MockProjectFile } from '../mock/types'
import { getSavePointFileDiff, listSavePointFiles, listSavePoints, restoreFileRevision, saveFileRevisionAs } from './api'
import { parseUnifiedDiff } from './diffParser'
import StructuredDiffViewer from './StructuredDiffViewer.vue'
import type { FileHistoryEntry, SavePointFile } from './types'

const props = defineProps<{
  projectName: string
  projectPath: string
  files: MockProjectFile[]
  tree: FileTreeEntry[]
  changedFiles: MockProjectFile[]
  ignoredFiles: MockProjectFile[]
  selectedCount: number
  isGitRepository: boolean
  hasFirstCommit: boolean
  realMode: boolean
  loading: boolean
  saving: boolean
  commitMessage: string
}>()

const emit = defineEmits<{
  selectFile: [path: string, part: 'selected' | 'unselected']
  setSelected: [path: string, selected: boolean]
  requestUndo: [path: string]
  ignoreFile: [path: string]
  ignoreWithRule: [path: string, mode: 'file' | 'extension']
  restoreIgnored: [path: string]
  stageAll: []
  unstageAll: []
  switchProject: []
  refresh: []
  restored: []
  updateCommitMessage: [message: string]
  createSavePoint: []
}>()

const leftVisible = ref(false)
const rightVisible = ref(true)
const hoverActionPath = ref('')
let actionCloseTimer: number | null = null
const detailPath = ref('')
const detailMode = ref<'content' | 'change'>('change')
const detailBusy = ref(false)
const detailMessage = ref('')
let detailMessageTimer: number | null = null
const savePoints = ref<FileHistoryEntry[]>([])
const savePointsHasMore = ref(false)
const savePointsLoading = ref(false)
const savePointsReloadPending = ref(false)
const selectedSavePoint = ref<FileHistoryEntry | null>(null)
const savedFiles = ref<MockProjectFile[]>([])
const saveDialogOpen = ref(false)
const saveMessage = ref('')
const batchRevealOpen = ref(false)
const showIgnored = ref(false)
const ignoreDialogOpen = ref(false)
const ignoreTarget = ref<MockProjectFile | null>(null)
const ignoreMode = ref<'file' | 'extension'>('file')

const displayFiles = computed(() => selectedSavePoint.value ? savedFiles.value : props.changedFiles)
const stagedFiles = computed(() => props.changedFiles.filter((file) => file.hasSelectedChanges))
const unstagedCount = computed(() => props.changedFiles.filter((file) => file.hasUnselectedChanges).length)
const ignoreExtensionLabel = computed(() => {
  const name = ignoreTarget.value?.name || ''
  const index = name.lastIndexOf('.')
  return index > 0 ? `*${name.slice(index)}` : '无后缀文件'
})
const detailFile = computed(() => {
  const source = detailMode.value === 'content' ? props.files : displayFiles.value
  return source.find((file) => file.path === detailPath.value) || null
})

watch(() => props.projectPath, () => {
  leftVisible.value = false
  rightVisible.value = true
  showCurrent()
  void reloadSavePoints()
})
watch(() => props.selectedCount, (count) => {
  if (saveDialogOpen.value && count === 0) closeSaveDialog()
})
watch(() => props.saving, (saving, wasSaving) => {
  if (wasSaving && !saving) void reloadSavePoints()
})

onMounted(() => { void loadMoreSavePoints() })
onBeforeUnmount(() => {
  if (detailMessageTimer !== null) window.clearTimeout(detailMessageTimer)
  if (actionCloseTimer !== null) window.clearTimeout(actionCloseTimer)
})

function openActions(path: string) {
  if (actionCloseTimer !== null) window.clearTimeout(actionCloseTimer)
  actionCloseTimer = null
  hoverActionPath.value = path
}
function scheduleCloseActions(path: string) {
  if (actionCloseTimer !== null) window.clearTimeout(actionCloseTimer)
  actionCloseTimer = window.setTimeout(() => {
    if (hoverActionPath.value === path) hoverActionPath.value = ''
    actionCloseTimer = null
  }, 240)
}
function closeActions() {
  if (actionCloseTimer !== null) window.clearTimeout(actionCloseTimer)
  actionCloseTimer = null
  hoverActionPath.value = ''
}
function closeDetail() { detailPath.value = ''; detailMessage.value = '' }
function closeOnDetailBlank(event: MouseEvent) {
  const target = event.target
  if (!(target instanceof Element)) return
  if (target.closest('button, textarea, .cm-content, .dt-diff-line, .dt-diff-file-head, .focus-detail-actions, .focus-detail-title')) return
  closeDetail()
}
function leaveIgnoredView() {
  showIgnored.value = false
  batchRevealOpen.value = false
}

function toggleIgnoredView() {
  if (showIgnored.value) {
    showIgnored.value = false
    return
  }
  selectedSavePoint.value = null
  savedFiles.value = []
  closeDetail()
  closeActions()
  showIgnored.value = true
}

function showCurrent() {
  leaveIgnoredView()
  selectedSavePoint.value = null
  savedFiles.value = []
  closeDetail()
  closeActions()
}

function selectFromTree(path: string) {
  const file = props.files.find((item) => item.path === path)
  if (!file) return
  leaveIgnoredView()
  selectedSavePoint.value = null
  savedFiles.value = []
  closeActions()
  detailMode.value = 'content'
  detailPath.value = file.path
  emit('selectFile', file.path, file.hasUnselectedChanges ? 'unselected' : 'selected')
}

function openFileDetail(file: MockProjectFile) {
  closeActions()
  detailMode.value = 'change'
  detailPath.value = file.path
  if (selectedSavePoint.value) void loadSavedFileDiff(file)
  else emit('selectFile', file.path, file.hasUnselectedChanges ? 'unselected' : 'selected')
}

function stageFile(file: MockProjectFile) { closeActions(); emit('setSelected', file.path, !file.hasSelectedChanges) }
function undoFile(file: MockProjectFile) { closeActions(); emit('requestUndo', file.path) }
function ignoreFile(file: MockProjectFile) { openIgnoreDialog(file) }
function canUndo(file: MockProjectFile) { return (file.status === 'modified' || file.status === 'added') && file.hasUnselectedChanges }
function runBatch(stage: boolean) {
  leaveIgnoredView()
  if (stage) emit('stageAll')
  else emit('unstageAll')
}
function openIgnoreDialog(file: MockProjectFile) { closeActions(); ignoreTarget.value = file; ignoreMode.value = 'file'; ignoreDialogOpen.value = true }
function closeIgnoreDialog() { ignoreDialogOpen.value = false; ignoreTarget.value = null }
function submitIgnore() {
  if (!ignoreTarget.value) return
  emit('ignoreWithRule', ignoreTarget.value.path, ignoreMode.value)
  closeIgnoreDialog()
}

function openSaveDialog() { leaveIgnoredView(); saveMessage.value = props.commitMessage; saveDialogOpen.value = true }
function closeSaveDialog() { if (!props.saving) saveDialogOpen.value = false }
function removeFromSave(file: MockProjectFile) { emit('setSelected', file.path, false) }
function submitSavePoint() {
  const message = saveMessage.value.trim()
  if (!message || props.saving) return
  emit('updateCommitMessage', message)
  emit('createSavePoint')
  saveDialogOpen.value = false
}

function showDetailMessage(message: string) {
  detailMessage.value = message
  if (detailMessageTimer !== null) window.clearTimeout(detailMessageTimer)
  detailMessageTimer = window.setTimeout(() => { detailMessage.value = ''; detailMessageTimer = null }, 3000)
}

async function saveSavedFile(file: MockProjectFile) {
  if (!selectedSavePoint.value) return
  closeActions()
  if (!props.realMode) { showDetailMessage(`已模拟另存 ${file.name}`); return }
  const targetPath = await save({ title: `另存 ${selectedSavePoint.value.shortCommit} 版本`, defaultPath: file.name })
  if (!targetPath) return
  detailBusy.value = true
  try {
    await saveFileRevisionAs(props.projectPath, file.path, selectedSavePoint.value.commit, targetPath)
    showDetailMessage(`已另存为：${targetPath}`)
  } catch (error) {
    showDetailMessage(`另存失败：${formatError(error)}`)
  } finally { detailBusy.value = false }
}

async function restoreSavedFile(file: MockProjectFile) {
  if (!selectedSavePoint.value) return
  closeActions()
  if (!props.realMode) { showDetailMessage(`已模拟恢复 ${file.name}`); return }
  const accepted = await confirm(`恢复 ${file.path} 到 ${selectedSavePoint.value.shortCommit}？\n\n当前文件将被这个版本替换，并作为新的未保存变化保留。`, { title: '确认恢复文件', kind: 'warning' })
  if (!accepted) return
  detailBusy.value = true
  try {
    await restoreFileRevision(props.projectPath, file.path, selectedSavePoint.value.commit)
    emit('restored')
    closeDetail()
  } catch (error) {
    showDetailMessage(`恢复失败：${formatError(error)}`)
  } finally { detailBusy.value = false }
}

async function loadMoreSavePoints() {
  if (!props.isGitRepository || !props.hasFirstCommit || savePointsLoading.value) return
  savePointsLoading.value = true
  try {
    const page = props.realMode ? await listSavePoints(props.projectPath, savePoints.value.length, 3) : mockSavePointPage(savePoints.value.length)
    savePoints.value.push(...page.entries)
    savePointsHasMore.value = page.hasMore
  } finally {
    savePointsLoading.value = false
    if (savePointsReloadPending.value) {
      savePointsReloadPending.value = false
      void reloadSavePoints()
    }
  }
}

async function reloadSavePoints() {
  if (savePointsLoading.value) {
    savePointsReloadPending.value = true
    return
  }
  savePoints.value = []
  savePointsHasMore.value = false
  await loadMoreSavePoints()
}

async function selectSavePoint(entry: FileHistoryEntry) {
  leaveIgnoredView()
  selectedSavePoint.value = entry
  savedFiles.value = []
  closeDetail()
  const files = props.realMode ? await listSavePointFiles(props.projectPath, entry.commit) : mockSavedFiles()
  savedFiles.value = files.map(savePointFileToCard)
}

async function loadSavedFileDiff(file: MockProjectFile) {
  if (!selectedSavePoint.value || file.diff.length) return
  try {
    const result = props.realMode
      ? await getSavePointFileDiff(props.projectPath, file.path, selectedSavePoint.value.commit)
      : { relativePath: file.path, diff: mockDiff(file.path) }
    file.diff = parseUnifiedDiff(result.diff)
    file.additions = file.diff.filter((line) => line.kind === 'added').length
    file.deletions = file.diff.filter((line) => line.kind === 'deleted').length
  } catch (error) {
    file.previewUnavailable = error instanceof Error ? error.message : String(error)
  }
}

function savePointFileToCard(file: SavePointFile): MockProjectFile {
  const status = normalizeStatus(file.status)
  return { path: file.relativePath, oldPath: file.oldRelativePath, name: file.relativePath.split('/').pop() || file.relativePath, status, size: '—', modifiedAt: selectedSavePoint.value?.date || '', language: '', content: '', additions: 0, deletions: 0, hasSelectedChanges: false, hasUnselectedChanges: false, diff: [] }
}

function normalizeStatus(status: GitFileStatus): ChangeStatus {
  if (status === 'added' || status === 'untracked') return 'added'
  if (status === 'deleted') return 'deleted'
  if (status === 'renamed') return 'renamed'
  return 'modified'
}

function statusText(status: ChangeStatus) { return ({ clean: '没有变化', modified: '已修改', added: '新文件', deleted: '已删除', renamed: '已改名' })[status] }
function fileExtension(name: string) { const index = name.lastIndexOf('.'); return index > 0 ? name.slice(index) : '' }
function fileBaseName(name: string) { const extension = fileExtension(name); return extension ? name.slice(0, -extension.length) : name }
function formatDate(value: string) { const date = new Date(value); return Number.isNaN(date.getTime()) ? value : date.toLocaleString('zh-CN', { month: 'long', day: 'numeric', hour: '2-digit', minute: '2-digit' }) }
function formatError(error: unknown) { return error instanceof Error ? error.message : typeof error === 'object' && error && 'message' in error ? String((error as { message: unknown }).message) : String(error) }

function fileIcon(file: MockProjectFile) {
  const name = file.name.toLowerCase()
  if (/\.(png|jpe?g|gif|webp|svg)$/.test(name)) return markRaw(IconPhoto)
  if (/\.(md|txt|docx?|pdf)$/.test(name)) return markRaw(IconFileDescription)
  if (/\.(json|ya?ml|toml|ini|env)$/.test(name) || name.startsWith('.')) return markRaw(IconSettings)
  if (/\.(vue|tsx?|jsx?|rs|py|java|css|html|sql|sh)$/.test(name)) return markRaw(IconFileCode)
  return markRaw(IconFile)
}

function mockSavePointPage(skip: number) {
  const all = [
    ['a61d723', '整理变化文件界面', '2026-07-14T18:32:00+08:00'], ['9cb8e41', '补充文件历史', '2026-07-14T14:18:00+08:00'],
    ['74bf210', '支持忽略新文件', '2026-07-13T18:46:00+08:00'], ['51a723c', '优化文件预览', '2026-07-12T14:20:00+08:00'],
    ['38d9f12', '完成项目概览', '2026-07-11T20:08:00+08:00'], ['20c1ab8', '建立工作台', '2026-07-11T10:15:00+08:00']
  ]
  const entries = all.slice(skip, skip + 3).map(([shortCommit, subject, date]) => ({ commit: shortCommit.padEnd(40, '0'), shortCommit, subject, date }))
  return { entries, hasMore: skip + entries.length < all.length }
}

function mockSavedFiles(): SavePointFile[] {
  return [
    { relativePath: 'src/App.vue', status: 'modified' }, { relativePath: 'src/styles.css', status: 'modified' },
    { relativePath: 'src/features/changes.ts', status: 'added' }, { relativePath: 'docs/old-notes.md', status: 'deleted' }
  ]
}

function mockDiff(path: string) { return `diff --git a/${path} b/${path}\n--- a/${path}\n+++ b/${path}\n@@ -12,3 +12,5 @@\n-  const oldView = true\n+  const focusOnFiles = true\n+  const hiddenActions = ['暂存', '撤销', '忽略']\n   return focusOnFiles\n` }
</script>
