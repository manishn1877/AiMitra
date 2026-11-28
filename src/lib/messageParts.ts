export type MessagePart =
  | { type: 'text'; text: string }
  | { type: 'code'; language: string; code: string; closed: boolean }

/**
 * Splits a model reply into plain text and fenced code blocks (```lang ... ```).
 * An unterminated fence (reply still incomplete, or cut off) is returned as a
 * code block with `closed: false` so the code is still usable.
 */
export function parseMessageParts(content: string): MessagePart[] {
  const parts: MessagePart[] = []
  const fence = /```([^\n`]*)\n([\s\S]*?)(```|$)/g
  let last = 0
  let match: RegExpExecArray | null

  while ((match = fence.exec(content)) !== null) {
    if (match.index > last) {
      parts.push({ type: 'text', text: content.slice(last, match.index) })
    }
    parts.push({
      type: 'code',
      language: match[1].trim().split(/\s+/)[0].toLowerCase(),
      code: match[2].replace(/\n$/, ''),
      closed: match[3] === '```',
    })
    last = match.index + match[0].length
  }

  if (last < content.length) {
    parts.push({ type: 'text', text: content.slice(last) })
  }
  return parts
}

/** Languages the desktop app can execute. Keep in sync with normalize_language in main.rs. */
export function runnableLanguage(language: string): 'python' | 'javascript' | null {
  switch (language.trim().toLowerCase()) {
    case 'python':
    case 'py':
    case 'python3':
      return 'python'
    case 'javascript':
    case 'js':
    case 'node':
    case 'mjs':
      return 'javascript'
    default:
      return null
  }
}
