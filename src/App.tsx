import { useState, useEffect } from 'react'
import { LayoutDashboard, Settings, Square, ChevronLeft, ChevronRight, Download } from 'lucide-react'
import { cn } from './lib/utils'
import { MODEL_REGISTRY } from './lib/models'
import { invoke } from '@tauri-apps/api/core'
import { ChatPanel } from './components/ChatPanel'
import { ModelSelector } from './components/ModelSelector'
import { SettingsPanel } from './components/SettingsPanel'
import { StatusBar } from './components/StatusBar'
import { ChatHistory } from './components/ChatHistory'

type View = 'chat' | 'models' | 'settings'

export function App() {
  const [view, setView] = useState<View>('chat')
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [currentModel, setCurrentModel] = useState('')
  const [downloadedModels, setDownloadedModels] = useState<string[]>([])
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null)
  const [downloadProgress, setDownloadProgress] = useState(0)
  const [theme, setTheme] = useState<'light' | 'dark' | 'system'>('system')

  // Load initial state
  useEffect(() => {
    loadInitialState()
    setupEventListeners()
  }, [])

  const loadInitialState = async () => {
    try {
      const [models, settings] = await Promise.all([
        invoke<string[]>('list_downloaded_models'),
        invoke<{ model: string; theme: string }>('get_settings'),
      ])
      setDownloadedModels(models)
      // A saved preference does not mean llama-server is running.
      setCurrentModel('')
      setTheme(settings.theme as 'light' | 'dark' | 'system')
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      applyTheme(settings.theme as 'light' | 'dark' | 'system', systemTheme)
    } catch (err) {
      console.error('Failed to load initial state:', err)
    }
  }

  const setupEventListeners = () => {
    // Listen for download progress events
    const setup = async () => {
      const { listen } = await import('@tauri-apps/api/event')
      const unlistenStarted = await listen('model:download_started', (event) => {
        const { modelId } = event.payload as { modelId: string }
        setDownloadingModel(modelId)
        setDownloadProgress(0)
      })
      const unlistenProgress = await listen('model:download_progress', (event) => {
        const { modelId, progress } = event.payload as { modelId: string; progress: number }
        setDownloadingModel(modelId)
        setDownloadProgress(progress)
        if (progress >= 100) {
          setTimeout(() => {
            setDownloadingModel(null)
            setDownloadProgress(0)
            refreshDownloadedModels()
          }, 1000)
        }
      })
      const unlistenError = await listen('model:download_error', (event) => {
        const { modelId, error } = event.payload as { modelId: string; error: string }
        setDownloadingModel(null)
        setDownloadProgress(0)
        window.alert(`Could not download ${modelId}: ${error}`)
      })
      const unlistenComplete = await listen('model:download_complete', (event) => {
        const { modelId, success } = event.payload as { modelId: string; success: boolean }
        if (success) {
          // Downloaded is not the same as loaded; activation is a separate step.
          refreshDownloadedModels()
        }
      })
      return () => {
        unlistenStarted()
        unlistenProgress()
        unlistenError()
        unlistenComplete()
      }
    }
    setup()
  }

  const applyTheme = (themeSetting: 'light' | 'dark' | 'system', systemTheme: 'light' | 'dark') => {
    const effectiveTheme = themeSetting === 'system' ? systemTheme : themeSetting
    document.documentElement.classList.toggle('dark', effectiveTheme === 'dark')
  }

  // Keep the selected theme synchronized with Settings and the operating system.
  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const syncSystemTheme = () => applyTheme(theme, media.matches ? 'dark' : 'light')
    const onThemePreferenceChange = (event: Event) => {
      const preference = (event as CustomEvent<'light' | 'dark' | 'system'>).detail
      if (preference) setTheme(preference)
    }

    syncSystemTheme()
    media.addEventListener('change', syncSystemTheme)
    window.addEventListener('aimitra:theme-change', onThemePreferenceChange)
    return () => {
      media.removeEventListener('change', syncSystemTheme)
      window.removeEventListener('aimitra:theme-change', onThemePreferenceChange)
    }
  }, [theme])

  const refreshDownloadedModels = async () => {
    try {
      const models = await invoke<string[]>('list_downloaded_models')
      setDownloadedModels(models)
    } catch (err) {
      console.error('Failed to refresh models:', err)
    }
  }

  const handleModelChange = async (modelId: string) => {
    // load_model unloads the previous model before starting the new one.
    setCurrentModel('')
    try {
      await invoke('load_model', { modelId })
      setCurrentModel(modelId)

      // save_settings requires the complete Settings object.
      const settings = await invoke<Record<string, unknown>>('get_settings')
      await invoke('save_settings', { settings: { ...settings, model: modelId } })
    } catch (err) {
      console.error('Failed to load model:', err)
      window.alert(`Could not activate this model: ${String(err)}`)
    }
  }

  const handleModelDelete = async (modelId: string) => {
    if (!confirm(`Delete ${MODEL_REGISTRY.find(m => m.id === modelId)?.name || modelId}?`)) return
    try {
      await invoke('delete_model', { modelId })
      setDownloadedModels(prev => prev.filter(m => m !== modelId))
      if (currentModel === modelId) {
        setCurrentModel('')
        await invoke('unload_model')
      }
    } catch (err) {
      console.error('Failed to delete model:', err)
    }
  }

  const navigation = [
    { id: 'chat' as View, label: 'Chat', icon: LayoutDashboard },
    { id: 'models' as View, label: 'Models', icon: Download },
    { id: 'settings' as View, label: 'Settings', icon: Settings },
  ]

  return (
    <div className="flex h-screen bg-surface-light dark:bg-surface-dark overflow-hidden">
      {/* Sidebar */}
      <aside
        className={cn(
          'flex flex-col border-r border-slate-200 dark:border-slate-700 bg-white/50 dark:bg-slate-900/50 backdrop-blur-sm transition-all duration-300',
          sidebarOpen ? 'w-64' : 'w-16'
        )}
      >
        {/* Header */}
        <div className="flex items-center justify-between h-16 px-4 border-b border-slate-200 dark:border-slate-700">
          {sidebarOpen && (
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-brand-500 to-brand-600 flex items-center justify-center">
                <Square className="w-5 h-5 text-white" />
              </div>
              <span className="font-semibold text-slate-900 dark:text-slate-100">AiMitra</span>
            </div>
          )}
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className="btn-ghost p-2"
            aria-label={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
          >
            {sidebarOpen ? <ChevronLeft className="w-5 h-5" /> : <ChevronRight className="w-5 h-5" />}
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 px-2 py-4 space-y-1 overflow-y-auto" role="navigation" aria-label="Main navigation">
          {navigation.map((item) => (
            <button
              key={item.id}
              onClick={() => setView(item.id)}
              className={cn(
                'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-150',
                view === item.id
                  ? 'bg-brand-50 dark:bg-brand-900/30 text-brand-700 dark:text-brand-300'
                  : 'text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-900 dark:hover:text-slate-100'
              )}
              title={sidebarOpen ? undefined : item.label}
            >
              <item.icon className="w-5 h-5 flex-shrink-0" aria-hidden="true" />
              {sidebarOpen && <span className="font-medium">{item.label}</span>}
            </button>
          ))}
          {sidebarOpen && <ChatHistory onOpen={() => setView('chat')} />}
        </nav>

        {/* Current Model Quick Info */}
        {sidebarOpen && currentModel && (
          <div className="p-4 border-t border-slate-200 dark:border-slate-700">
            <div className="flex items-center gap-2 text-sm mb-2">
              <span className="badge-success">Active Model</span>
            </div>
            <div className="flex items-center justify-between p-2 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <span className="font-medium text-slate-900 dark:text-slate-100 truncate">
                {MODEL_REGISTRY.find(m => m.id === currentModel)?.name || currentModel}
              </span>
              <button
                onClick={() => setView('models')}
                className="btn-ghost p-1.5 text-xs"
              >
                Change
              </button>
            </div>
          </div>
        )}

        {sidebarOpen && (
          <div className="p-4 border-t border-slate-200 dark:border-slate-700">
            <p className="text-xs text-slate-500 dark:text-slate-400 text-center">
              v0.1.0 • Local AI Assistant
            </p>
          </div>
        )}
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col overflow-hidden">
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Stay mounted so the draft and any running reply survive switching pages. */}
          <div className={cn('flex-1 min-h-0 flex-col overflow-hidden', view === 'chat' ? 'flex' : 'hidden')}>
            <ChatPanel />
          </div>
          {view === 'models' && (
            <div className="flex-1 overflow-y-auto p-6">
              <div className="max-w-3xl mx-auto">
                <div className="mb-6">
                  <h2 className="text-2xl font-semibold text-slate-900 dark:text-slate-100">Models</h2>
                  <p className="text-slate-600 dark:text-slate-400 mt-1">
                    Download and manage local AI models. All models run entirely on your machine.
                  </p>
                </div>
                <ModelSelector
                  currentModel={currentModel}
                  onModelChange={handleModelChange}
                  onModelDelete={handleModelDelete}
                  downloadedModels={downloadedModels}
                  downloadingModel={downloadingModel}
                  downloadProgress={downloadProgress}
                />
              </div>
            </div>
          )}
          {view === 'settings' && (
            <div className="flex-1 overflow-y-auto p-6">
              <div className="max-w-2xl mx-auto">
                <div className="mb-6">
                  <h2 className="text-2xl font-semibold text-slate-900 dark:text-slate-100">Settings</h2>
                  <p className="text-slate-600 dark:text-slate-400 mt-1">
                    Configure model parameters, system performance, and interface preferences.
                  </p>
                </div>
                <SettingsPanel />
              </div>
            </div>
          )}
        </div>

        {/* Status Bar */}
        <StatusBar />
      </main>
    </div>
  )
}