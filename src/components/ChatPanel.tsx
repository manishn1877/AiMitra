import { useState, useRef, useEffect, useCallback } from 'react'
import { Send, X, Copy, Bot, User, Zap } from 'lucide-react'
import { cn } from '../lib/utils'
import { useCompletions } from '../hooks/useCompletions'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
  timing?: {
    prompt_ms: number
    generation_ms: number
    tokens_per_second: number
  }
}

export function ChatPanel() {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isGenerating, setIsGenerating] = useState(false)
  const [currentResponse, setCurrentResponse] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [])

  useEffect(() => {
    scrollToBottom()
  }, [messages, currentResponse, scrollToBottom])

  const { generate, stop, error } = useCompletions({
    onToken: (token) => {
      setCurrentResponse(prev => prev + token)
    },
    onComplete: (response) => {
      if (response.done) {
        const answer = response.text || currentResponse
        if (answer.trim()) {
          setMessages(prev => [...prev, {
            id: crypto.randomUUID(),
            role: 'assistant',
            content: answer,
            timestamp: new Date(),
            timing: response.timing,
          }])
        }
      }
      // Always unlock the composer when the backend finishes, even if it
      // returns an empty or incomplete response.
      setCurrentResponse('')
      setIsGenerating(false)
    },
    onError: (err) => {
      console.error('Generation error:', err)
      setIsGenerating(false)
    },
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim() || isGenerating) return

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date(),
    }

    setMessages(prev => [...prev, userMessage])
    setIsGenerating(true)
    const prompt = input.trim()
    setInput('')

    // Build context from recent messages
    const context = messages
      .slice(-6) // Last 6 messages for context
      .map(m => `${m.role === 'user' ? 'User' : 'Assistant'}: ${m.content}`)
      .join('\n\n')

    const fullPrompt = `${context}\n\nUser: ${prompt}\n\nAssistant:`

    await generate({
      prompt: fullPrompt,
      max_tokens: 2048,
      temperature: 0.2,
      top_p: 0.95,
      stop: ['User:', 'Human:'],
      stream: true,
    })
  }

  const handleStop = () => {
    stop()
    if (currentResponse) {
      setMessages(prev => [...prev, {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: currentResponse + ' [stopped]',
        timestamp: new Date(),
      }])
      setCurrentResponse('')
    }
    setIsGenerating(false)
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6 scrollbar-thin">
        {messages.length === 0 && !isGenerating && (
          <div className="flex flex-col items-center justify-center h-full text-center text-slate-500 dark:text-slate-400">
            <Bot className="w-16 h-16 mb-4 text-brand-500/50" />
            <h3 className="text-lg font-medium mb-2">Welcome to AiMitra</h3>
            <p className="text-sm max-w-xs">
              Your local AI code assistant. Ask me to write, explain, or debug code — all running privately on your machine.
            </p>
            <div className="mt-6 flex flex-wrap gap-2 justify-center">
              {[
                'Write a Python function to validate GSTIN',
                'Explain React useEffect cleanup',
                'Create a REST API in Go with Gin',
                'Debug this TypeScript error...',
              ].map((suggestion, i) => (
                <button
                  key={i}
                  onClick={() => { setInput(suggestion); textareaRef.current?.focus(); }}
                  className="btn-secondary text-sm px-3 py-1.5"
                >
                  {suggestion}
                </button>
              ))}
            </div>
          </div>
        )}

        {messages.map((message) => (
          <div key={message.id} className={cn('flex gap-3', message.role === 'user' && 'flex-row-reverse')}>
            <div
              className={cn(
                'w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0',
                message.role === 'user'
                  ? 'bg-brand-100 dark:bg-brand-900/30 text-brand-600 dark:text-brand-400'
                  : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400'
              )}
            >
              {message.role === 'user' ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
            </div>
            <div className={cn('flex-1 min-w-0', message.role === 'user' && 'text-right')}>
              <div className={cn(
                'relative rounded-2xl p-4 max-w-[85%]',
                message.role === 'user'
                  ? 'bg-brand-50 dark:bg-brand-900/20 ml-auto text-slate-900 dark:text-slate-100 rounded-tr-sm'
                  : 'bg-slate-100 dark:bg-slate-800 mr-auto text-slate-900 dark:text-slate-100 rounded-tl-sm'
              )}>
                <pre className="whitespace-pre-wrap text-sm font-mono leading-relaxed">{message.content}</pre>
                <div className="flex items-center justify-end gap-2 mt-2">
                  <span className="text-xs text-slate-500 dark:text-slate-400">
                    {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                  </span>
                  {message.role === 'assistant' && (
                    <>
                      <button
                        onClick={() => copyToClipboard(message.content)}
                        className="btn-ghost p-1.5 hover:bg-slate-200 dark:hover:bg-slate-700"
                        title="Copy"
                      >
                        <Copy className="w-4 h-4" />
                      </button>
                      {message.timing && (
                        <span className="text-xs text-slate-500 dark:text-slate-400 flex items-center gap-1">
                          <Zap className="w-3 h-3" />
                          {message.timing.tokens_per_second?.toFixed(1)} tok/s
                        </span>
                      )}
                    </>
                  )}
                </div>
              </div>
            </div>
          </div>
        ))}

        {/* Streaming response */}
        {isGenerating && currentResponse && (
          <div className="flex gap-3 animate-in">
            <div className="w-8 h-8 rounded-full bg-slate-100 dark:bg-slate-800 flex items-center justify-center flex-shrink-0">
              <Bot className="w-4 h-4 text-slate-600 dark:text-slate-400" />
            </div>
            <div className="flex-1">
              <div className="bg-slate-100 dark:bg-slate-800 rounded-2xl p-4 max-w-[85%] rounded-tl-sm">
                <pre className="whitespace-pre-wrap text-sm font-mono leading-relaxed">{currentResponse}<span className="animate-pulse text-brand-500">█</span></pre>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Error Banner */}
      {error && (
        <div className="border-t border-red-200 dark:border-red-800 p-3 bg-red-50 dark:bg-red-900/20">
          <div className="flex items-center gap-2 text-red-700 dark:text-red-300 text-sm">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clipRule="evenodd"/></svg>
            {error.message}
          </div>
        </div>
      )}

      {/* Input Area */}
      <form onSubmit={handleSubmit} className="border-t border-slate-200 dark:border-slate-700 p-4">
        <div className="flex gap-2">
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Ask me to write, explain, or debug code..."
            className="flex-1 input resize-none min-h-[56px] max-h-48 font-mono text-sm"
            rows={1}
            disabled={isGenerating}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                handleSubmit(e)
              }
            }}
          />
          <div className="flex flex-col gap-2 justify-end">
            {isGenerating ? (
              <button type="button" onClick={handleStop} className="btn-danger">
                <X className="w-4 h-4" />
                <span className="hidden sm:inline">Stop</span>
              </button>
            ) : (
              <button type="submit" disabled={!input.trim()} className="btn-primary">
                <Send className="w-4 h-4" />
                <span className="hidden sm:inline">Send</span>
              </button>
            )}
          </div>
        </div>
        <p className="text-xs text-slate-500 dark:text-slate-400 mt-2 text-center">
          Press <kbd className="px-1.5 py-0.5 bg-slate-100 dark:bg-slate-800 rounded text-slate-700 dark:text-slate-300">Enter</kbd> to send, <kbd className="px-1.5 py-0.5 bg-slate-100 dark:bg-slate-800 rounded text-slate-700 dark:text-slate-300">Shift+Enter</kbd> for new line
        </p>
      </form>
    </div>
  )
}