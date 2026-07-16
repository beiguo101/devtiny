<template>
  <section class="dt-view dt-files-view">
    <aside class="dt-files-tree">
      <div class="dt-pane-heading"><div><span class="dt-eyebrow">项目文件</span><h2>文件树</h2></div><span>{{ files.length }}</span></div>
      <div class="dt-tree-scroll">
        <MockFileTreeNode v-for="entry in tree" :key="entry.id" :entry="entry" :selected-path="selectedFile?.path || ''" @select="$emit('selectFile', $event)" />
      </div>
    </aside>
    <main class="dt-file-detail">
      <template v-if="selectedFile">
        <header class="dt-file-heading">
          <div><span class="dt-eyebrow">文件详情</span><h1>{{ selectedFile.name }}</h1><p>{{ selectedFile.path }}</p></div>
          <button v-if="selectedFile.status !== 'clean'" class="dt-button dt-button-primary" type="button" @click="$emit('showDiff', selectedFile.path)">
            查看变化 <IconArrowRight />
          </button>
        </header>
        <dl class="dt-file-facts">
          <div><dt>大小</dt><dd>{{ selectedFile.size }}</dd></div>
          <div><dt>最后修改</dt><dd>{{ selectedFile.modifiedAt }}</dd></div>
          <div><dt>类型</dt><dd>{{ selectedFile.language }}</dd></div>
          <div><dt>Git 状态</dt><dd><span class="dt-status-text" :class="`is-${selectedFile.status}`">{{ statusLabel(selectedFile.status) }}</span></dd></div>
          <div><dt>保存选择</dt><dd>{{ selectionLabel }}</dd></div>
        </dl>
        <FileRevisionContent
          v-if="selectedRevision"
          :project-path="projectPath"
          :relative-path="selectedFile.path"
          :entry="selectedRevision"
          @close="selectedRevision = null"
          @restored="handleRevisionRestored"
        />
        <section v-else class="dt-content-preview">
          <div class="dt-section-title"><div><h2>当前文件内容</h2><p>只读预览</p></div></div>
          <div v-if="selectedFile.status === 'deleted'" class="dt-empty-compact"><IconFileX /><strong>文件已从工作区删除</strong><span>可前往变化视图查看删除内容。</span></div>
          <div v-else-if="selectedFile.binary" class="dt-empty-compact"><IconPhoto /><strong>二进制文件</strong><span>{{ selectedFile.name }} · {{ selectedFile.size }}，不提供文本预览。</span></div>
          <CodeViewer v-else :text="selectedFile.content" :relative-path="selectedFile.path" mode="content" :editable="false" />
        </section>
        <FileRevisionHistory
          :project-path="projectPath"
          :relative-path="selectedFile.path"
          :is-git-repository="isGitRepository"
          :real-mode="realMode"
          :selected-commit="selectedRevision?.commit"
          @select-revision="selectedRevision = $event"
        />
      </template>
      <div v-else class="dt-empty-large"><IconFile /><strong>选择一个文件</strong><span>查看文件信息、当前内容和 Git 状态。</span></div>
    </main>
  </section>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from 'vue'
import { IconArrowRight, IconFile, IconFileX, IconPhoto } from '@tabler/icons-vue'
import type { ChangeStatus, FileTreeEntry, MockProjectFile } from '../mock/types'
import MockFileTreeNode from './MockFileTreeNode.vue'
import FileRevisionHistory from '../history/FileRevisionHistory.vue'
import FileRevisionContent from '../history/FileRevisionContent.vue'
import type { FileHistoryEntry } from '../changes/types'

const CodeViewer = defineAsyncComponent(() => import('./CodeViewer.vue'))

const props = defineProps<{ tree: FileTreeEntry[]; files: MockProjectFile[]; selectedFile: MockProjectFile | null; projectPath: string; isGitRepository: boolean; realMode: boolean }>()
const emit = defineEmits<{ selectFile: [path: string]; showDiff: [path: string]; restored: [] }>()
const selectedRevision = ref<FileHistoryEntry | null>(null)

watch(() => props.selectedFile?.path, () => { selectedRevision.value = null })

const selectionLabel = computed(() => {
  if (!props.selectedFile || props.selectedFile.status === 'clean') return '无变化'
  if (props.selectedFile.hasSelectedChanges && props.selectedFile.hasUnselectedChanges) return '部分已选择，部分尚未选择'
  return props.selectedFile.hasSelectedChanges ? '已选择保存（Git staged）' : '尚未选择'
})

function statusLabel(status: ChangeStatus) {
  return ({ clean: '无变化', modified: '修改', added: '新增', deleted: '删除', renamed: '重命名' })[status]
}

function handleRevisionRestored() {
  selectedRevision.value = null
  emit('restored')
}
</script>
