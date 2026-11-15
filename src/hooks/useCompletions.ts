import { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

export interface CompletionRequest {
  prompt: string
  max_tokens?: number
  temperature?: number
  top_p?: number
  stop?: string[]
  stream?: boolean
}

export interface CompletionResponse {
  text: string
  done: boolean
  tokens_generated?: number
  timing?: {
    prompt_ms: number
    generation_ms: number
    tokens_per_second: number
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
  const abortControllerRef = useRef<AbortController | null>(null)

  const generate = useCallback(async (request: CompletionRequest) => {
    if (isGenerating) return

    setIsGenerating(true)
    setError(null)
    abortControllerRef.current = new AbortController()

    try {
      const response = await invoke<CompletionResponse>('generate_completion', {
        request: {
          ...request,
          stream: true,
        },
      })

      // For streaming, the backend sends events via Tauri event system
      // This is handled by the event listener below
      options.onComplete?.(response)
    } catch (err) {
      const error = err instanceof Error ? err : new Error(String(err))
      setError(error)
      options.onError?.(error)
    } finally {
      setIsGenerating(false)
    }
  }, [isGenerating, options])

  const stop = useCallback(() => {
    abortControllerRef.current?.abort()
    invoke('stop_generation').catch(console.error)
    setIsGenerating(false)
  }, [])

  // Listen for streaming tokens from backend
  useEffect(() => {
    // Tauri 2 event listening
    let unlistenToken: (() => void) | null = null
    let unlistenDone: (() => void) | null = null

    const setupListeners = async () => {
      const { listen } = await import('@tauri-apps/api/event')
      unlistenToken = await listen('llama:token', (event) => {
        options.onToken?.(event.payload as string)
      })
      unlistenDone = await listen('llama:done', (event) => {
        options.onComplete?.(event.payload as CompletionResponse)
      })
    }

    setupListeners()

    return () => {
      unlistenToken?.()
      unlistenDone?.()
    }
  }, [options])

  return {
    generate,
    stop,
    isGenerating,
    error,
  }
}