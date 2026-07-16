import { computed, reactive } from 'vue'
import { executeWorkbenchAction, previewWorkbenchAction } from '../../../core/commands/api'
import type { GitFileStatus, WorkbenchAction } from '../../../core/commands/types'
import { listProjectFiles, readProjectFile } from '../../../core/filesystem/api'
import type { FileTreeNode as ApiFileTreeNode } from '../../../core/filesystem/types'
import { getProjectOverview } from '../../../core/project/api'
import { addIgnoredRule, getFileDiff, listGitChanges, listIgnoredRules, removeIgnoredRule } from '../changes/api'
import type { GitChangeFile } from '../changes/types'
import { parseUnifiedDiff } from '../changes/diffParser'
import type {
  DiffLine,
  FileTreeEntry,
  MockProjectFile,
  SaveResult,
  ScenarioId,
  ScenarioOption,
  WorkbenchView
} from './types'

const context = (oldLine: number, newLine: number, content: string): DiffLine => ({
  kind: 'context', oldLine, newLine, content
})
const added = (newLine: number, content: string): DiffLine => ({ kind: 'added', newLine, content })
const deleted = (oldLine: number, content: string): DiffLine => ({ kind: 'deleted', oldLine, content })
const hunk = (content: string): DiffLine => ({ kind: 'hunk', content })

function baseFiles(): MockProjectFile[] {
  return [
    {
      path: 'src/App.vue', name: 'App.vue', status: 'modified', size: '8.4 KB', modifiedAt: '今天 14:32',
      language: 'Vue', additions: 42, deletions: 18, hasSelectedChanges: true, hasUnselectedChanges: false,
      content: '<template>\n  <DevTinyWorkbench />\n</template>\n',
      diff: [
        hunk('@@ -12,7 +12,10 @@'),
        context(12, 12, '  <main class="workbench">'),
        deleted(13, '    <RuntimePanel v-if="activeView === \'runtime\'" />'),
        added(13, '    <ProjectHeader :project="project" />'),
        added(14, '    <ChangesView v-if="activeView === \'changes\'" />'),
        context(14, 15, '  </main>')
      ]
    },
    {
      path: 'src/components/DiffViewer.vue', name: 'DiffViewer.vue', status: 'added', size: '5.2 KB', modifiedAt: '今天 14:26',
      language: 'Vue', additions: 84, deletions: 0, hasSelectedChanges: true, hasUnselectedChanges: false,
      content: '<template>\n  <div class="diff-viewer">...</div>\n</template>\n',
      diff: [hunk('@@ -0,0 +1,6 @@'), added(1, '<template>'), added(2, '  <div class="diff-viewer">'), added(3, '    <DiffLine v-for="line in lines" :line="line" />'), added(4, '  </div>'), added(5, '</template>')]
    },
    {
      path: 'src/stores/project.ts', name: 'project.ts', status: 'modified', size: '4.1 KB', modifiedAt: '今天 14:18',
      language: 'TypeScript', additions: 28, deletions: 9, hasSelectedChanges: false, hasUnselectedChanges: true,
      content: 'export const project = reactive({\n  changes: []\n})\n',
      diff: [hunk('@@ -4,5 +4,8 @@'), context(4, 4, 'export const project = reactive({'), deleted(5, '  files: []'), added(5, '  files: [] as ProjectFile[],'), added(6, '  selectedPaths: new Set<string>()'), context(6, 7, '})')]
    },
    {
      path: 'src-tauri/src/git.rs', name: 'git.rs', status: 'modified', size: '12.8 KB', modifiedAt: '今天 13:54',
      language: 'Rust', additions: 31, deletions: 7, hasSelectedChanges: false, hasUnselectedChanges: true,
      content: 'pub fn git_status() -> Result<()> {\n    Ok(())\n}\n',
      diff: [hunk('@@ -28,6 +28,9 @@'), context(28, 28, 'pub fn git_status(path: &Path) -> Result<Status> {'), deleted(29, '    run_git(path, &["status"])'), added(29, '    let output = run_git(path, &["status", "--porcelain"])?;'), added(30, '    parse_status(&output)'), context(30, 31, '}')]
    },
    {
      path: 'README.md', name: 'README.md', status: 'modified', size: '3.7 KB', modifiedAt: '今天 13:42',
      language: 'Markdown', additions: 12, deletions: 4, hasSelectedChanges: true, hasUnselectedChanges: false,
      content: '# DevTiny\n\nA tiny local project change viewer.\n',
      diff: [hunk('@@ -1,4 +1,6 @@'), context(1, 1, '# DevTiny'), deleted(3, 'A tiny Git client.'), added(3, 'A local project fact view.'), added(4, ''), added(5, 'See what changed before saving a stable point.')]
    },
    {
      path: 'old-notes.md', name: 'old-notes.md', status: 'deleted', size: '1.2 KB', modifiedAt: '昨天 18:06',
      language: 'Markdown', additions: 0, deletions: 24, hasSelectedChanges: false, hasUnselectedChanges: true,
      content: '',
      diff: [hunk('@@ -1,4 +0,0 @@'), deleted(1, '# Old notes'), deleted(2, ''), deleted(3, 'Temporary implementation ideas.'), deleted(4, 'Remove before release.')]
    },
    {
      path: 'logo.png', name: 'logo.png', status: 'added', size: '148 KB', modifiedAt: '今天 12:30',
      language: 'Binary', additions: 0, deletions: 0, hasSelectedChanges: false, hasUnselectedChanges: true,
      content: '', binary: true, previewUnavailable: '这是二进制文件，无法显示文本 Diff。', diff: []
    },
    {
      path: 'package.json', name: 'package.json', status: 'clean', size: '1.4 KB', modifiedAt: '7月 10日',
      language: 'JSON', additions: 0, deletions: 0, hasSelectedChanges: false, hasUnselectedChanges: false,
      content: '{\n  "name": "devtiny",\n  "version": "0.1.0"\n}\n', diff: []
    },
    {
      path: 'vite.config.ts', name: 'vite.config.ts', status: 'clean', size: '0.5 KB', modifiedAt: '7月 10日',
      language: 'TypeScript', additions: 0, deletions: 0, hasSelectedChanges: false, hasUnselectedChanges: false,
      content: "import { defineConfig } from 'vite'\n\nexport default defineConfig({})\n", diff: []
    }
  ]
}

export const scenarioOptions: ScenarioOption[] = [
  { id: 'normal', label: '普通 Git 项目' }, { id: 'clean', label: '没有变化' },
  { id: 'non-git', label: '非 Git 目录' }, { id: 'empty-git', label: '空 Git 仓库' },
  { id: 'partial', label: '同文件含两类变化' }, { id: 'added', label: '新增文件' },
  { id: 'deleted', label: '删除文件' }, { id: 'renamed', label: '重命名文件' },
  { id: 'binary', label: '二进制文件' }, { id: 'large-diff', label: '大 Diff' },
  { id: 'commit-success', label: '保存成功' }, { id: 'commit-failure', label: '保存失败' },
  { id: 'undo-confirm', label: '撤销确认' }, { id: 'undo-success', label: '撤销成功' },
  { id: 'operation-failure', label: '操作失败' }, { id: 'loading', label: '加载中' },
  { id: 'read-failure', label: '项目读取失败' }
]

function buildLargeDiff() {
  const lines: DiffLine[] = [hunk('@@ -40,80 +40,112 @@')]
  for (let index = 40; index < 120; index += 1) {
    if (index % 9 === 0) lines.push(deleted(index, `    old_value_${index} = legacy_config()`))
    lines.push(added(index + 5, `    value_${index} = project_state.resolve(${index})`))
    lines.push(context(index + 1, index + 6, '    apply_change(value)'))
  }
  return lines
}

function buildTree(files: MockProjectFile[]): FileTreeEntry[] {
  const root: FileTreeEntry[] = []
  for (const file of files) {
    const parts = file.path.split('/')
    let level = root
    let currentPath = ''
    parts.forEach((part, index) => {
      currentPath = currentPath ? `${currentPath}/${part}` : part
      const isFile = index === parts.length - 1
      let entry = level.find((item) => item.name === part)
      if (!entry) {
        entry = { id: currentPath, name: part, path: currentPath, kind: isFile ? 'file' : 'directory', file: isFile ? file : undefined, children: [] }
        level.push(entry)
      }
      level = entry.children
    })
  }
  const sort = (items: FileTreeEntry[]) => items.sort((a, b) => a.kind === b.kind ? a.name.localeCompare(b.name) : a.kind === 'directory' ? -1 : 1).forEach((item) => sort(item.children))
  sort(root)
  return root
}

export function useMockProject() {
  const state = reactive({
    projectOpen: false,
    realMode: false,
    activeView: 'changes' as WorkbenchView,
    scenario: 'normal' as ScenarioId,
    project: { name: 'devtiny', path: '/Users/lin/Projects/devtiny', branch: 'main', lastCommit: 'a61d723', lastCommitSubject: 'Initial local project view', lastCommitAt: '今天 11:48' },
    isGitRepository: true,
    hasFirstCommit: true,
    files: baseFiles(),
    ignoredFiles: [] as MockProjectFile[],
    selectedPath: 'src/App.vue',
    selectedPart: 'selected' as 'selected' | 'unselected',
    commitMessage: '',
    saving: false,
    saveResult: null as SaveResult | null,
    error: '',
    notice: '',
    loading: false,
    undoTargetPath: '',
    undoDialogOpen: false,
    initializingGit: false
  })

  const changedFiles = computed(() => state.files.filter((file) => file.status !== 'clean' && (file.hasSelectedChanges || file.hasUnselectedChanges)))
  const selectedFiles = computed(() => changedFiles.value.filter((file) => file.hasSelectedChanges))
  const unselectedFiles = computed(() => changedFiles.value.filter((file) => file.hasUnselectedChanges))
  const selectedFile = computed(() => state.files.find((file) => file.path === state.selectedPath) || null)
  const additions = computed(() => changedFiles.value.reduce((sum, file) => sum + file.additions, 0))
  const deletions = computed(() => changedFiles.value.reduce((sum, file) => sum + file.deletions, 0))
  const statusCounts = computed(() => ({
    added: changedFiles.value.filter((file) => file.status === 'added').length,
    modified: changedFiles.value.filter((file) => file.status === 'modified').length,
    deleted: changedFiles.value.filter((file) => file.status === 'deleted').length,
    renamed: changedFiles.value.filter((file) => file.status === 'renamed').length
  }))
  const fileTree = computed(() => buildTree(state.files))
  const canSave = computed(() => selectedFiles.value.length > 0 && Boolean(state.commitMessage.trim()) && !state.saving)

  function openProject() {
    state.projectOpen = true
    state.activeView = 'overview'
  }

  async function loadRealProject(projectPath: string) {
    state.projectOpen = true
    state.realMode = true
    state.loading = true
    state.error = ''
    state.notice = ''
    state.saveResult = null
    state.activeView = 'overview'
    try {
      const [overview, tree, ignoredRules] = await Promise.all([
        getProjectOverview(projectPath),
        listProjectFiles(projectPath),
        listIgnoredRules(projectPath)
      ])
      const changes = overview.isGitRepository ? await listGitChanges(projectPath) : []
      const flattened = flattenApiTree(tree)
      const changeByPath = new Map(changes.map((change) => [change.relativePath, change]))
      const knownPaths = new Set(flattened.map((file) => file.path))
      for (const change of changes) {
        if (!knownPaths.has(change.relativePath)) flattened.push(realFile(change.relativePath, change))
      }
      state.files = flattened.map((file) => {
        const change = changeByPath.get(file.path)
        return change ? mergeRealChange(file, change) : file
      })
      state.ignoredFiles = ignoredRules.map((entry) => ignoredRuleFile(entry.rule, entry.displayName, entry.mode))
      state.project = {
        name: projectPath.split('/').filter(Boolean).pop() || projectPath,
        path: overview.projectPath,
        branch: overview.branch || 'HEAD',
        lastCommit: overview.lastCommit || '',
        lastCommitSubject: overview.lastCommitSubject || (overview.isGitRepository ? '尚无首次提交' : '未开启版本管理'),
        lastCommitAt: overview.lastCommitAt ? new Date(overview.lastCommitAt).toLocaleString() : '—'
      }
      state.isGitRepository = overview.isGitRepository
      state.hasFirstCommit = Boolean(overview.lastCommit)
      const initial = state.files.find((file) => file.status !== 'clean') || state.files[0]
      state.selectedPath = initial?.path || ''
      state.selectedPart = initial?.hasUnselectedChanges ? 'unselected' : 'selected'
      if (initial) await loadRealFile(initial.path, state.selectedPart)
    } catch (error) {
      state.error = `无法读取项目：${formatError(error)}`
    } finally {
      state.loading = false
    }
  }

  async function refreshRealProject() {
    if (state.realMode && state.project.path) {
      const currentView = state.activeView
      await loadRealProject(state.project.path)
      state.activeView = currentView
    }
  }

  async function initializeGit() {
    if (!state.realMode || state.isGitRepository) return
    state.initializingGit = true
    state.error = ''
    try {
      await executeRealAction('git.init', [])
      await refreshRealProject()
      state.activeView = 'overview'
      state.notice = '已在当前目录初始化 Git。你现在可以开始建立第一个保存点。'
    } catch (error) {
      state.error = `初始化 Git 失败：${formatError(error)}`
    } finally {
      state.initializingGit = false
    }
  }

  function selectFile(path: string, part: 'selected' | 'unselected' = 'unselected') {
    state.selectedPath = path
    state.selectedPart = part
    state.saveResult = null
    if (state.realMode) void loadRealFile(path, part)
  }

  async function loadRealFile(path: string, part: 'selected' | 'unselected') {
    const file = state.files.find((item) => item.path === path)
    if (!file) return
    state.error = ''
    try {
      if (file.status !== 'deleted') {
        const content = await readProjectFile(state.project.path, path)
        file.content = content.content
        file.binary = content.isBinary
        file.size = formatBytes(content.size)
        file.modifiedAt = content.modifiedAt ? new Date(content.modifiedAt).toLocaleString() : '—'
      }
      if (file.status !== 'clean' && !file.binary) {
        const status: GitFileStatus = part === 'selected' ? 'staged' : file.status === 'added' ? 'untracked' : file.status
        const result = await getFileDiff(state.project.path, path, status)
        file.diff = parseUnifiedDiff(result.diff)
        file.additions = file.diff.filter((line) => line.kind === 'added').length
        file.deletions = file.diff.filter((line) => line.kind === 'deleted').length
        if (!file.diff.length && file.status === 'added' && file.content) {
          file.diff = file.content.split('\n').map((content, index) => added(index + 1, content))
          file.additions = file.diff.length
        }
      }
    } catch (error) {
      file.previewUnavailable = formatError(error)
    }
  }

  function showDiff(path: string) {
    selectFile(path, state.files.find((file) => file.path === path)?.hasUnselectedChanges ? 'unselected' : 'selected')
    state.activeView = 'changes'
  }

  async function setSelected(path: string, selected: boolean) {
    const file = state.files.find((item) => item.path === path)
    if (!file || file.status === 'clean') return
    if (state.realMode) {
      try {
        await executeRealAction(selected ? 'git.stageFiles' : 'git.unstageFiles', [file])
        await refreshRealProject()
        state.selectedPath = path
        state.selectedPart = selected ? 'selected' : 'unselected'
        state.notice = selected ? `${file.name} 已加入本次保存。` : `${file.name} 已移出本次保存。`
      } catch (error) {
        state.error = `${selected ? '暂存' : '取消暂存'} ${file.path} 失败：${formatError(error)}`
      }
      return
    }
    file.hasSelectedChanges = selected
    file.hasUnselectedChanges = !selected
    state.selectedPart = selected ? 'selected' : 'unselected'
    state.notice = selected ? `${file.name} 已加入本次保存。` : `${file.name} 已移出本次保存。`
  }

  async function selectAll() {
    if (state.realMode) {
      try {
        await executeRealAction('git.stageFiles', unselectedFiles.value)
        await refreshRealProject()
        state.notice = '所有变化已加入本次保存。'
      } catch (error) {
        state.error = `全部暂存失败：${formatError(error)}`
      }
      return
    }
    changedFiles.value.forEach((file) => { file.hasSelectedChanges = true; file.hasUnselectedChanges = false })
    state.notice = `已选择全部 ${changedFiles.value.length} 个变化文件。`
  }

  async function clearSelection() {
    if (state.realMode) {
      try {
        await executeRealAction('git.unstageFiles', selectedFiles.value)
        await refreshRealProject()
        state.notice = '已清除本次保存选择。'
      } catch (error) {
        state.error = `全部取消暂存失败：${formatError(error)}`
      }
      return
    }
    changedFiles.value.forEach((file) => { file.hasSelectedChanges = false; file.hasUnselectedChanges = true })
    state.notice = '已清除本次保存选择。'
  }

  async function ignoreFile(path: string, mode: 'file' | 'extension' = 'file') {
    const file = state.files.find((item) => item.path === path)
    if (!file || file.status !== 'added') {
      state.error = '只有尚未纳入项目的新文件可以忽略。'
      return
    }
    if (state.realMode) {
      try {
        if (file.hasSelectedChanges) await executeRealAction('git.unstageFiles', [file])
        await addIgnoredRule(state.project.path, file.path, mode)
        await refreshRealProject()
        state.notice = mode === 'extension' ? `已忽略所有 ${file.name.slice(file.name.lastIndexOf('.'))} 文件。` : `${file.name} 已忽略，之后不会再出现在变化文件中。`
      } catch (error) {
        state.error = `忽略 ${file.path} 失败：${formatError(error)}`
      }
      return
    }
    const extension = file.name.includes('.') ? file.name.slice(file.name.lastIndexOf('.')) : ''
    const matches = mode === 'extension'
      ? state.files.filter((item) => item.status === 'added' && (extension ? item.name.endsWith(extension) : !item.name.includes('.')))
      : [file]
    state.ignoredFiles.push(...matches.map((item) => ({ ...item, ignoreRule: mode === 'extension' ? `所有 *${extension || '（无后缀）'}` : '仅此文件' })))
    const ignoredPaths = new Set(matches.map((item) => item.path))
    state.files = state.files.filter((item) => !ignoredPaths.has(item.path))
    state.notice = mode === 'extension' ? `已忽略 ${matches.length} 个同后缀文件。` : `${file.name} 已忽略，之后不会再出现在变化文件中。`
  }

  async function restoreIgnored(path: string) {
    const file = state.ignoredFiles.find((item) => item.path === path)
    if (!file) return
    if (state.realMode) {
      try {
        await removeIgnoredRule(state.project.path, file.path)
        await refreshRealProject()
        state.notice = `${file.name} 已停止忽略。`
      } catch (error) {
        state.error = `停止忽略 ${file.name} 失败：${formatError(error)}`
      }
      return
    }
    state.ignoredFiles = state.ignoredFiles.filter((item) => item.path !== path)
    state.files.push({ ...file, ignoreRule: undefined })
    state.notice = `${file.name} 已停止忽略，重新回到变化列表。`
  }

  async function createSavePoint() {
    if (!canSave.value) return
    state.saving = true
    state.error = ''
    state.notice = ''
    if (state.realMode) {
      const included = selectedFiles.value.map((file) => file.path)
      const totals = selectedFiles.value.reduce((value, file) => ({ additions: value.additions + file.additions, deletions: value.deletions + file.deletions }), { additions: 0, deletions: 0 })
      try {
        await executeRealAction('git.commitFiles', selectedFiles.value, state.commitMessage.trim())
        await refreshRealProject()
        state.saveResult = { message: state.project.lastCommitSubject, fileCount: included.length, additions: totals.additions, deletions: totals.deletions, files: included, remainingCount: changedFiles.value.length }
        state.commitMessage = ''
      } catch (error) {
        state.error = `创建保存点失败：${formatError(error)}。文件选择和提交说明均已保留。`
      } finally {
        state.saving = false
      }
      return
    }
    await new Promise((resolve) => window.setTimeout(resolve, 650))
    if (state.scenario === 'commit-failure') {
      state.error = '创建保存点失败：模拟 Git 锁文件冲突。文件选择和提交说明均已保留。'
      state.saving = false
      return
    }
    const included = selectedFiles.value.map((file) => ({ path: file.path, additions: file.additions, deletions: file.deletions }))
    const result: SaveResult = {
      message: state.commitMessage.trim(), fileCount: included.length,
      additions: included.reduce((sum, file) => sum + file.additions, 0),
      deletions: included.reduce((sum, file) => sum + file.deletions, 0), files: included.map((file) => file.path), remainingCount: 0
    }
    state.files = state.files.filter((file) => {
      if (!file.hasSelectedChanges) return true
      if (file.hasUnselectedChanges) { file.hasSelectedChanges = false; return true }
      if (file.status === 'deleted') return false
      file.status = 'clean'; file.hasSelectedChanges = false; file.additions = 0; file.deletions = 0; file.diff = []
      return true
    })
    result.remainingCount = state.files.filter((file) => file.status !== 'clean' && file.hasUnselectedChanges).length
    state.project.lastCommit = 'e4c91b2'
    state.project.lastCommitSubject = result.message
    state.project.lastCommitAt = '刚刚'
    state.commitMessage = ''
    state.saveResult = result
    state.saving = false
  }

  function requestUndo(path: string) {
    const file = state.files.find((item) => item.path === path)
    if (!file || !file.hasUnselectedChanges || (file.status !== 'modified' && file.status !== 'added')) return
    state.undoTargetPath = path
    state.undoDialogOpen = true
  }

  function cancelUndo() { state.undoDialogOpen = false; state.undoTargetPath = '' }

  async function confirmUndo() {
    const file = state.files.find((item) => item.path === state.undoTargetPath)
    if (!file) return cancelUndo()
    state.undoDialogOpen = false
    if (state.realMode) {
      try {
        await executeRealAction('git.restoreFiles', [file])
        await refreshRealProject()
        state.notice = file.status === 'added'
          ? `${file.path} 已移到系统回收站。`
          : `已撤销 ${file.path} 的未保存修改，文件已恢复到最近一次提交状态。`
      } catch (error) {
        state.error = `${file.status === 'added' ? '移到回收站' : '撤销'} ${file.path} 失败：${formatError(error)}。原文件仍然保留。`
      }
      state.undoTargetPath = ''
      return
    }
    if (file.status === 'added') {
      state.files = state.files.filter((item) => item.path !== file.path)
      state.notice = `${file.path} 已移到回收站。`
      state.undoTargetPath = ''
      return
    }
    if (state.scenario === 'operation-failure') {
      state.error = `撤销 ${file.path} 失败：模拟文件被其他程序占用。原修改仍然保留。`
      return
    }
    file.hasUnselectedChanges = false
    if (!file.hasSelectedChanges) { file.status = 'clean'; file.additions = 0; file.deletions = 0; file.diff = [] }
    state.notice = `已撤销 ${file.path} 的未保存修改，文件已恢复到最近一次提交状态。`
    state.undoTargetPath = ''
  }

  function loadScenario(id: ScenarioId) {
    state.realMode = false
    state.scenario = id
    state.files = baseFiles()
    state.ignoredFiles = []
    state.isGitRepository = true
    state.hasFirstCommit = true
    state.loading = false
    state.error = ''
    state.notice = ''
    state.saveResult = null
    state.undoDialogOpen = false
    state.commitMessage = ''
    state.activeView = 'changes'
    state.selectedPath = 'src/App.vue'
    if (id === 'clean') state.files.forEach((file) => { file.status = 'clean'; file.hasSelectedChanges = false; file.hasUnselectedChanges = false; file.additions = 0; file.deletions = 0; file.diff = [] })
    if (id === 'non-git') { state.isGitRepository = false; state.hasFirstCommit = false }
    if (id === 'empty-git') { state.hasFirstCommit = false; state.files = state.files.filter((file) => file.status === 'clean') }
    if (id === 'partial') { const file = state.files[0]; file.hasSelectedChanges = true; file.hasUnselectedChanges = true }
    if (id === 'renamed') { const file = state.files[2]; file.status = 'renamed'; file.oldPath = 'src/stores/workspace.ts'; file.additions = 2; file.deletions = 2; state.selectedPath = file.path }
    if (id === 'binary') state.selectedPath = 'logo.png'
    if (id === 'large-diff') { const file = state.files[3]; file.diff = buildLargeDiff(); file.additions = 112; file.deletions = 28; state.selectedPath = file.path }
    if (id === 'commit-failure') { state.commitMessage = 'feat: improve project change view' }
    if (id === 'commit-success') { state.commitMessage = 'feat: improve project change view'; void createSavePoint() }
    if (id === 'undo-confirm' || id === 'operation-failure') { state.selectedPath = 'src/stores/project.ts'; requestUndo(state.selectedPath) }
    if (id === 'undo-success') { state.selectedPath = 'src/stores/project.ts'; state.undoTargetPath = state.selectedPath; void confirmUndo() }
    if (id === 'loading') state.loading = true
    if (id === 'read-failure') state.error = '无法读取项目：模拟目录权限不足。请检查目录访问权限后重试。'
    if (id === 'added') state.selectedPath = 'src/components/DiffViewer.vue'
    if (id === 'deleted') state.selectedPath = 'old-notes.md'
  }

  async function executeRealAction(action: WorkbenchAction, files: MockProjectFile[], message?: string) {
    const selections = files.map((file) => ({
      relativePath: file.path,
      status: toGitStatus(file),
      indexStatus: file.indexStatus ?? (file.hasSelectedChanges ? 'M' : ' '),
      worktreeStatus: file.worktreeStatus ?? (file.hasUnselectedChanges ? 'M' : ' ')
    }))
    const preview = await previewWorkbenchAction({ projectPath: state.project.path, action, payload: action === 'git.init' ? undefined : { files: selections, message } })
    const result = await executeWorkbenchAction(preview.previewToken)
    if (!result.success) throw new Error(result.stderr || result.stdout || 'Git 操作未成功。')
  }

  return { state, changedFiles, selectedFiles, unselectedFiles, selectedFile, additions, deletions, statusCounts, fileTree, canSave, openProject, loadRealProject, refreshRealProject, initializeGit, selectFile, showDiff, setSelected, selectAll, clearSelection, ignoreFile, restoreIgnored, createSavePoint, requestUndo, cancelUndo, confirmUndo, loadScenario }
}

function flattenApiTree(nodes: ApiFileTreeNode[]): MockProjectFile[] {
  return nodes.flatMap((node) => node.kind === 'directory' ? flattenApiTree(node.children) : [realFile(node.relativePath)])
}

function realFile(path: string, change?: GitChangeFile): MockProjectFile {
  return {
    path, name: path.split('/').pop() || path, status: change ? changeStatus(change) : 'clean',
    size: '—', modifiedAt: '—', language: fileLanguage(path), content: '', additions: change?.additions ?? 0, deletions: change?.deletions ?? 0,
    hasSelectedChanges: change ? isIndexChanged(change.indexStatus) : false,
    hasUnselectedChanges: change ? isWorktreeChanged(change.worktreeStatus) : false,
    indexStatus: change?.indexStatus,
    worktreeStatus: change?.worktreeStatus,
    diff: []
  }
}

function mergeRealChange(file: MockProjectFile, change: GitChangeFile) {
  const next = realFile(file.path, change)
  return { ...file, ...next, oldPath: change.oldRelativePath }
}

function changeStatus(change: GitChangeFile): MockProjectFile['status'] {
  const code = change.worktreeStatus !== ' ' && change.worktreeStatus !== '?' ? change.worktreeStatus : change.indexStatus
  if (code === 'A' || code === '?') return 'added'
  if (code === 'D') return 'deleted'
  if (code === 'R') return 'renamed'
  return 'modified'
}
function isIndexChanged(value: string) { return value !== ' ' && value !== '?' }
function isWorktreeChanged(value: string) { return value !== ' ' }
function toGitStatus(file: MockProjectFile): GitFileStatus {
  if (file.status === 'added' && file.indexStatus === '?' && file.worktreeStatus === '?') return 'untracked'
  return file.status === 'clean' ? 'clean' : file.status
}

function ignoredRuleFile(rule: string, displayName: string, mode: 'file' | 'extension'): MockProjectFile {
  return {
    path: rule, name: displayName, status: 'clean', size: '—', modifiedAt: '—', language: 'Ignore rule', content: '',
    additions: 0, deletions: 0, hasSelectedChanges: false, hasUnselectedChanges: false,
    ignoreRule: mode === 'extension' ? `所有 ${rule}` : '仅此文件', diff: []
  }
}
function fileLanguage(path: string) { return path.split('.').pop()?.toUpperCase() || 'File' }
function formatBytes(size: number) { return size < 1024 ? `${size} B` : size < 1024 * 1024 ? `${(size / 1024).toFixed(1)} KB` : `${(size / 1024 / 1024).toFixed(1)} MB` }
function formatError(error: unknown) { return error instanceof Error ? error.message : typeof error === 'object' && error && 'message' in error ? String((error as { message: unknown }).message) : String(error) }
