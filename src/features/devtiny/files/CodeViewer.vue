<template>
  <div class="code-viewer">
    <div v-if="mode === 'diff'" class="diff-viewer" v-html="renderedDiff"></div>
    <div v-else ref="editorHost" class="code-editor"></div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { basicSetup, EditorView } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { python } from '@codemirror/lang-python'
import { javascript } from '@codemirror/lang-javascript'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { json } from '@codemirror/lang-json'
import { markdown } from '@codemirror/lang-markdown'
import { java } from '@codemirror/lang-java'
import { sql } from '@codemirror/lang-sql'
import { xml } from '@codemirror/lang-xml'
import { yaml } from '@codemirror/lang-yaml'

const props = defineProps<{
  text: string
  relativePath: string
  mode: 'content' | 'diff'
  editable?: boolean
}>()

const emit = defineEmits<{
  change: [value: string]
}>()

const editorHost = ref<HTMLElement | null>(null)
let editor: EditorView | null = null
let syncingFromProp = false

const language = computed(() => detectLanguage(props.relativePath))
const renderedDiff = computed(() => renderDiff(props.text, language.value))

onMounted(() => {
  mountEditor()
})

onBeforeUnmount(() => {
  editor?.destroy()
  editor = null
})

watch(
  () => [props.text, props.relativePath, props.editable] as const,
  () => {
    if (!editor || props.mode === 'diff') {
      mountEditor()
      return
    }
    const current = editor.state.doc.toString()
    if (current !== props.text) {
      syncingFromProp = true
      editor.dispatch({
        changes: { from: 0, to: editor.state.doc.length, insert: props.text }
      })
      syncingFromProp = false
    }
  }
)

watch(
  () => props.mode,
  () => {
    mountEditor()
  }
)

function mountEditor() {
  if (props.mode === 'diff' || !editorHost.value) {
    editor?.destroy()
    editor = null
    return
  }

  editor?.destroy()
  editor = new EditorView({
    parent: editorHost.value,
    state: EditorState.create({
      doc: props.text,
      extensions: [
        basicSetup,
        devtinyEditorTheme,
        languageExtension(language.value),
        EditorView.editable.of(props.editable !== false),
        EditorView.lineWrapping,
        EditorView.updateListener.of((update) => {
          if (syncingFromProp || !update.docChanged) return
          emit('change', update.state.doc.toString())
        })
      ]
    })
  })
}

const devtinyEditorTheme = EditorView.theme({
  '&': {
    color: '#17251f',
    backgroundColor: '#fbfdfb',
    borderRadius: '6px',
    overflow: 'hidden',
    fontSize: '13px',
    border: '1px solid #dde6df'
  },
  '.cm-content': {
    fontFamily: '"SFMono-Regular", Consolas, "Liberation Mono", monospace',
    lineHeight: '1.55',
    minHeight: '360px'
  },
  '.cm-scroller': {
    maxHeight: '62vh',
    overflow: 'auto'
  },
  '.cm-gutters': {
    backgroundColor: '#f3f7f4',
    color: '#7a8b81',
    borderRightColor: '#dde6df'
  },
  '.cm-activeLine': {
    backgroundColor: '#edf6f0'
  },
  '.cm-activeLineGutter': {
    backgroundColor: '#edf6f0'
  },
  '.cm-selectionBackground': {
    backgroundColor: '#c9ead7 !important'
  },
  '.cm-cursor': {
    borderLeftColor: '#17834f'
  }
})

function detectLanguage(path: string) {
  const lower = path.toLowerCase()
  const fileName = lower.split('/').pop() || lower
  const ext = fileName.includes('.') ? fileName.split('.').pop() || '' : ''

  if (['md', 'markdown', 'mdx'].includes(ext)) return 'markdown'
  if (['py', 'pyw'].includes(ext)) return 'python'
  if (['html', 'htm', 'vue', 'svelte'].includes(ext)) return 'html'
  if (['xml'].includes(ext)) return 'xml'
  if (['java', 'kt', 'kts'].includes(ext)) return 'java'
  if (['js', 'jsx', 'ts', 'tsx', 'mjs', 'cjs'].includes(ext)) return 'javascript'
  if (['css', 'scss', 'sass', 'less'].includes(ext)) return 'css'
  if (['json', 'jsonc'].includes(ext)) return 'json'
  if (['yml', 'yaml'].includes(ext)) return 'yaml'
  if (['sql'].includes(ext)) return 'sql'
  return 'plain'
}

function languageExtension(lang: string) {
  switch (lang) {
    case 'python':
      return python()
    case 'javascript':
      return javascript({ jsx: true, typescript: true })
    case 'html':
      return html()
    case 'xml':
      return xml()
    case 'css':
      return css()
    case 'json':
      return json()
    case 'markdown':
      return markdown()
    case 'java':
      return java()
    case 'sql':
      return sql()
    case 'yaml':
      return yaml()
    default:
      return []
  }
}

function renderDiff(text: string, lang: string) {
  return text
    .split('\n')
    .map((line, index) => {
      const kind = diffKind(line)
      const prefix = escapeHtml(line.slice(0, 1) || ' ')
      const body = line.length > 1 ? line.slice(1) : ''
      return `<div class="diff-line diff-${kind}"><span class="diff-number">${index + 1}</span><span class="diff-prefix">${prefix}</span><code>${highlightDiffLine(body, lang)}</code></div>`
    })
    .join('')
}

function diffKind(line: string) {
  if (line.startsWith('+++') || line.startsWith('---')) return 'meta'
  if (line.startsWith('@@')) return 'hunk'
  if (line.startsWith('+')) return 'added-line'
  if (line.startsWith('-')) return 'deleted-line'
  return 'context'
}

function highlightDiffLine(text: string, lang: string) {
  const keywords = keywordSet(lang)
  const pattern =
    /(\/\*[\s\S]*?\*\/|<!--[\s\S]*?-->|#.*$|\/\/.*$|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\b\d+(?:\.\d+)?\b|\b[A-Za-z_$][\w$]*\b)/gm

  let html = ''
  let cursor = 0
  for (const match of text.matchAll(pattern)) {
    const token = match[0]
    const index = match.index ?? 0
    html += escapeHtml(text.slice(cursor, index))
    html += renderToken(token, lang, keywords)
    cursor = index + token.length
  }
  html += escapeHtml(text.slice(cursor))
  return html
}

function renderToken(token: string, lang: string, keywords: Set<string>) {
  const escaped = escapeHtml(token)
  if (isComment(token, lang)) return `<span class="tok-comment">${escaped}</span>`
  if (/^["'`]/.test(token)) return `<span class="tok-string">${escaped}</span>`
  if (/^\d/.test(token)) return `<span class="tok-number">${escaped}</span>`
  if (keywords.has(token)) return `<span class="tok-keyword">${escaped}</span>`
  return escaped
}

function keywordSet(lang: string) {
  const common = ['true', 'false', 'null', 'undefined', 'None', 'self', 'this', 'super']
  const byLanguage: Record<string, string[]> = {
    python: ['and', 'as', 'assert', 'async', 'await', 'break', 'class', 'continue', 'def', 'elif', 'else', 'except', 'finally', 'for', 'from', 'if', 'import', 'in', 'is', 'lambda', 'not', 'or', 'pass', 'raise', 'return', 'try', 'while', 'with', 'yield'],
    javascript: ['await', 'async', 'break', 'case', 'catch', 'class', 'const', 'continue', 'default', 'else', 'export', 'extends', 'finally', 'for', 'from', 'function', 'if', 'import', 'interface', 'let', 'new', 'of', 'return', 'switch', 'throw', 'try', 'type', 'typeof', 'var', 'while'],
    java: ['abstract', 'boolean', 'break', 'case', 'catch', 'class', 'continue', 'default', 'double', 'else', 'enum', 'extends', 'final', 'finally', 'float', 'for', 'if', 'implements', 'import', 'int', 'interface', 'long', 'new', 'package', 'private', 'protected', 'public', 'return', 'static', 'switch', 'throw', 'try', 'void', 'while'],
    sql: ['select', 'from', 'where', 'join', 'insert', 'update', 'delete', 'create', 'alter', 'drop', 'table', 'group', 'order', 'by', 'limit', 'and', 'or', 'not', 'null']
  }
  return new Set([...(byLanguage[lang] || []), ...common])
}

function isComment(token: string, lang: string) {
  if (token.startsWith('/*') || token.startsWith('//') || token.startsWith('<!--')) return true
  return token.startsWith('#') && ['python', 'shell', 'yaml'].includes(lang)
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}
</script>
