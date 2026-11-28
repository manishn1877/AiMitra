import { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface CompletionRequest {
  prompt: string
  max_tokens?: number
  temperature?: number
  top_p?: number
  stop?: string[]
  stream?: boolean
}

// The Rust backend serializes these structs as camelCase.
export interface CompletionResponse {
  text: string
  done: boolean
  tokensGenerated?: number
  timing?: {
    promptMs: number
    generationMs: number
    tokensPerSecond: number
  }
}

export interface UseCompletionsOptions {
  onToken?: (token: string) => void
  onComplete?: (response: CompletionResponse) => void
  onError?: (error: Error) => void
}

export function useCompletions(options: UseCompletionsOptions = {}) {
  const [isGenerating, setIsGenerating] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  // Always call the latest callbacks without re-subscribing to events on every render.
  const optionsRef = useRef(options)
  optionsRef.current = options
  const busyRef = useRef(false)

  const generate = useCallback(async (request: CompletionRequest) => {
    if (busyRef.current) return

    busyRef.current = true
    setIsGenerating(true)
    setError(null)

    try {
      const response = await invoke<CompletionResponse>('generate_completion', {
        request: {
          ...request,
          stream: true,
        },
      })
      optionsRef.current.onComplete?.(response)
    } catch (err) {
      const e = err instanceof Error ? err : new Error(String(err))
      setError(e)
      optionsRef.current.onError?.(e)
    } finally {
      busyRef.current = false
      setIsGenerating(false)
    }
  }, [])

  const stop = useCallback(() => {
    invoke('stop_generation').catch(console.error)
    setIsGenerating(false)
  }, [])

  // Streaming tokens are emitted by the backend as "llama:token" events.
  useEffect(() => {
    let cancelled = false
    let unlisten: (() => void) | null = null

    listen<string>('llama:token', (event) => {
      optionsRef.current.onToken?.(event.payload)
    })
      .then((fn) => {
        if (cancelled) fn()
        else unlisten = fn
      })
      .catch(console.error)

    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [])

  return {
    generate,
    stop,
    isGenerating,
    error,
  }
}
