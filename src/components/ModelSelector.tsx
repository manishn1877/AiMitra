import { useState } from 'react'
import { Download, Check, Info, Trash2, ExternalLink, Zap, Loader2 } from 'lucide-react'
import { cn, formatBytes } from '../lib/utils'
import { MODEL_REGISTRY, type ModelInfo } from '../lib/models'
import { invoke } from '@tauri-apps/api/core'

interface ModelSelectorProps {
  currentModel: string
  onModelChange: (modelId: string) => void
  onModelDelete: (modelId: string) => void
  downloadedModels: string[]
  downloadingModel: string | null
  downloadProgress: number
}

export function ModelSelector({
  currentModel,
  onModelChange,
  onModelDelete,
  downloadedModels,
  downloadingModel,
  downloadProgress,
}: ModelSelectorProps) {
  const [showAll, setShowAll] = useState(false)
  const [hoveredModel, setHoveredModel] = useState<string | null>(null)

  const isDownloaded = (id: string) => downloadedModels.includes(id)
  const isCurrent = (id: string) => currentModel === id
  const isDownloading = (id: string) => downloadingModel === id

  const handleDownload = async (model: ModelInfo) => {
    try {
      // The backend looks up the URL and filename from its own registry.
      await invoke('download_model', { modelId: model.id })
    } catch (err) {
      console.error('Download failed:', err)
      window.alert(`Could not start the download of ${model.name}: ${String(err)}`)
    }
  }

  const handleSelect = (model: ModelInfo) => {
    if (isDownloaded(model.id)) {
      onModelChange(model.id)
    } else {
      handleDownload(model)
    }
  }

  const displayModels = showAll ? MODEL_REGISTRY : MODEL_REGISTRY.filter(m => m.recommended)

  return (
    <div className="card p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-slate-900 dark:text-slate-100">Models</h3>
        <button
          onClick={() => setShowAll(!showAll)}
          className="btn-ghost text-sm px-2 py-1"
        >
          {showAll ? 'Show Recommended' : 'Show All Models'}
        </button>
      </div>

      <div className="space-y-3" role="list" aria-label="Available models">
        {displayModels.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            isCurrent={isCurrent(model.id)}
            isDownloaded={isDownloaded(model.id)}
            isDownloading={isDownloading(model.id)}
            downloadProgress={isDownloading(model.id) ? downloadProgress : 0}
            onSelect={() => handleSelect(model)}
            onDelete={() => onModelDelete(model.id)}
            onHover={setHoveredModel}
            hoveredModel={hoveredModel}
          />
        ))}
      </div>

      {downloadingModel && (
        <div className="mt-4 p-3 bg-brand-50 dark:bg-brand-900/20 rounded-lg border border-brand-200 dark:border-brand-800">
          <div className="flex items-center gap-2 mb-2">
            <Loader2 className="w-4 h-4 text-brand-600 dark:text-brand-400 animate-spin" />
            <span className="text-sm font-medium text-brand-700 dark:text-brand-300">
              Downloading {MODEL_REGISTRY.find(m => m.id === downloadingModel)?.name || downloadingModel}
            </span>
          </div>
          <div className="h-2 bg-brand-100 dark:bg-brand-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-brand-600 dark:bg-brand-400 transition-all duration-300"
              style={{ width: `${downloadProgress}%` }}
            />
          </div>
          <p className="text-xs text-brand-600 dark:text-brand-400 mt-1 text-right">
            {downloadProgress.toFixed(1)}%
          </p>
        </div>
      )}
    </div>
  )
}

interface ModelCardProps {
  model: ModelInfo
  isCurrent: boolean
  isDownloaded: boolean
  isDownloading: boolean
  downloadProgress: number
  onSelect: () => void
  onDelete: () => void
  onHover: (id: string | null) => void
  hoveredModel: string | null
}

function ModelCard({
  model,
  isCurrent,
  isDownloaded,
  isDownloading,
  downloadProgress,
  onSelect,
  onDelete,
  onHover,
  hoveredModel,
}: ModelCardProps) {
  return (
    <div
      className={cn(
        'relative p-4 rounded-xl border-2 transition-all duration-200',
        isCurrent
          ? 'border-brand-500 bg-brand-50 dark:bg-brand-900/20'
          : 'border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600'
      )}
      onMouseEnter={() => onHover(model.id)}
      onMouseLeave={() => onHover(null)}
    >
      {/* Current indicator */}
      {isCurrent && (
        <div className="absolute -top-2 -right-2 w-5 h-5 bg-brand-500 text-white rounded-full flex items-center justify-center">
          <Check className="w-3 h-3" />
        </div>
      )}

      <div className="flex items-start gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h4 className="font-medium text-slate-900 dark:text-slate-100 truncate">{model.name}</h4>
            {model.recommended && (
              <span className="badge-info">Recommended</span>
            )}
          </div>
          <p className="text-sm text-slate-600 dark:text-slate-400 mb-2">{model.description}</p>
          <div className="flex flex-wrap gap-1.5 mb-2">
            {model.tags.map(tag => (
              <span key={tag} className="badge bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400">
                {tag}
              </span>
            ))}
          </div>
          <div className="flex items-center gap-4 text-xs text-slate-500 dark:text-slate-400">
            <span className="flex items-center gap-1">
              <Info className="w-3 h-3" />
              {formatBytes(model.sizeGB * 1024 * 1024 * 1024)}
            </span>
            <span className="flex items-center gap-1">
              <Zap className="w-3 h-3" />
              ~{model.ramRequiredGB}GB RAM
            </span>
            <span className="flex items-center gap-1">
              <ExternalLink className="w-3 h-3" />
              {model.contextLength / 1024}K ctx
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2 flex-shrink-0">
          {isDownloading ? (
            <button className="btn-secondary px-3" disabled>
              <Loader2 className="w-4 h-4 animate-spin" />
            </button>
          ) : isDownloaded ? (
            <>
              {isCurrent ? (
                <button className="btn-primary px-3" disabled>
                  <Check className="w-4 h-4" />
                  <span className="hidden sm:inline">Active</span>
                </button>
              ) : (
                <button onClick={onSelect} className="btn-secondary px-3">
                  <span className="hidden sm:inline">Use</span>
                </button>
              )}
              {hoveredModel === model.id && (
                <button
                  onClick={onDelete}
                  className="btn-ghost p-2 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
                  title="Delete model"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              )}
            </>
          ) : (
            <button onClick={onSelect} className="btn-primary px-3">
              <Download className="w-4 h-4" />
              <span className="hidden sm:inline">Download</span>
            </button>
          )}
        </div>
      </div>

      {isDownloading && (
        <div className="mt-3 h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-brand-500 transition-all duration-300"
            style={{ width: `${downloadProgress}%` }}
          />
        </div>
      )}
    </div>
  )
}