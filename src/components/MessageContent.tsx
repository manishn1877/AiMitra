import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Check, Copy, ExternalLink, Loader2, Play } from 'lucide-react'
import { parseMessageParts, runnableLanguage } from '../lib/messageParts'

interface RunResult {
  stdout: string
  stderr: string
  exitCode: number | null
  timedOut: boolean
}

// Ask once per app session, not on every click.
let runWarningAccepted = false

function CodeBlock({ language, code }: { language: string; code: string }) {
  const [copied, setCopied] = useState(false)
  const [running, setRunning] = useState(false)
  const [result, setResult] = useState<RunResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const runnable = runnableLanguage(language) !== null

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch (err) {
      setError(`Could not copy: ${String(err)}`)
    }
  }

  const run = async () => {
    if (!runWarningAccepted) {
      const ok = window.confirm(
        'This runs the code on your computer with your own permissions. Only run code you understand. Continue?',
      )
      if (!ok) return
      runWarningAccepted = true
    }
    setRunning(true)
    setResult(null)
    setError(null)
    try {
      setResult(await invoke<RunResult>('run_code', { language, code }))
    } catch (err) {
      setError(String(err))
    } finally {
      setRunning(false)
    }
  }

  const openInEditor = async () => {
    setError(null)
    setNotice(null)
    try {
      setNotice(await invoke<string>('open_in_editor', { language, code }))
    } catch (err) {
      setError(String(err))
    }
  }

  const buttonClass =
    'inline-flex items-center gap-1 rounded px-2 py-1 text-xs text-slate-200 hover:bg-slate-700 disabled:opacity-50'

  return (
    <div className="my-3 overflow-hidden rounded-lg border border-slate-700 bg-slate-900 text-left">
      <div className="flex items-center justify-between bg-slate-800 px-3 py-1.5">
        <span className="font-mono text-xs text-slate-300">{language || 'code'}</span>
        <div className="flex items-center gap-1">
          <button type="button" onClick={copy} className={buttonClass} title="Copy code">
            {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            {copied ? 'Copied' : 'Copy'}
          </button>
          <button type="button" onClick={openInEditor} className={buttonClass} title="Save and open in VS Code">
            <ExternalLink className="h-3.5 w-3.5" />
            VS Code
          </button>
          {runnable && (
            <button
              type="button"
              onClick={run}
              disabled={running}
              className="inline-flex items-center gap-1 rounded bg-green-600 px-2 py-1 text-xs font-medium text-white hover:bg-green-700 disabled:opacity-50"
              title="Run this code on your computer"
            >
              {running ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Play className="h-3.5 w-3.5" />}
              Run
            </button>
          )}
        </div>
      </div>
      <pre className="overflow-x-auto p-3 text-sm leading-relaxed text-slate-100">
        <code>{code}</code>
      </pre>

      {(result || error || notice) && (
        <div className="border-t border-slate-700 bg-black/40 p-3 font-mono text-xs">
          {notice && <p className="text-slate-300">{notice}</p>}
          {error && <p className="whitespace-pre-wrap text-red-400">{error}</p>}
          {result && (
            <>
              {result.stdout && <pre className="whitespace-pre-wrap text-slate-100">{result.stdout}</pre>}
              {result.stderr && <pre className="whitespace-pre-wrap text-red-400">{result.stderr}</pre>}
              {!result.stdout && !result.stderr && (
                <p className="text-slate-400">(no output)</p>
              )}
              <p className="mt-2 text-slate-400">
                {result.timedOut
                  ? 'Stopped: timed out'
                  : `Finished with exit code ${result.exitCode ?? 'unknown'}`}
              </p>
            </>
          )}
        </div>
      )}
    </div>
  )
}

export function MessageContent({ content }: { content: string }) {
  const parts = parseMessageParts(content)
  return (
    <div className="text-sm leading-relaxed">
      {parts.map((part, i) =>
        part.type === 'code' ? (
          <CodeBlock key={i} language={part.language} code={part.code} />
        ) : (
          <div key={i} className="whitespace-pre-wrap break-words">
            {part.text}
          </div>
        ),
      )}
    </div>
  )
}
