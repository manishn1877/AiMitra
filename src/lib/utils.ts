import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`
  const minutes = Math.floor(ms / 60000)
  const seconds = ((ms % 60000) / 1000).toFixed(0)
  return `${minutes}m ${seconds}s`
}

export function getModelDisplayName(id: string): string {
  const names: Record<string, string> = {
    'qwen2.5-coder-1.5b': 'Qwen2.5 Coder 1.5B',
    'deepseek-coder-1.3b': 'DeepSeek Coder 1.3B',
    'phi-3-mini': 'Phi-3 Mini 4K',
  }
  return names[id] || id
}

export function estimateRAM(modelSizeGB: number): number {
  // Rough estimate: model size * 1.5 (for KV cache, context, overhead)
  return Math.ceil(modelSizeGB * 1.5)
}