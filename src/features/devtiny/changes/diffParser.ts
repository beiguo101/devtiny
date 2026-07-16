import type { DiffLine } from '../mock/types'

export function parseUnifiedDiff(raw: string): DiffLine[] {
  const result: DiffLine[] = []
  let oldLine = 0
  let newLine = 0
  for (const line of raw.split('\n')) {
    if (line.startsWith('@@')) {
      const match = /@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line)
      if (match) { oldLine = Number(match[1]); newLine = Number(match[2]) }
      result.push({ kind: 'hunk', content: line })
      continue
    }
    if (line.startsWith('diff ') || line.startsWith('index ') || line.startsWith('---') || line.startsWith('+++') || line.startsWith('\\ No newline')) continue
    if (line.startsWith('+')) { result.push({ kind: 'added', newLine: newLine++, content: line.slice(1) }); continue }
    if (line.startsWith('-')) { result.push({ kind: 'deleted', oldLine: oldLine++, content: line.slice(1) }); continue }
    if (line.startsWith(' ')) result.push({ kind: 'context', oldLine: oldLine++, newLine: newLine++, content: line.slice(1) })
  }
  return result
}
