export interface ModelInfo {
  id: string
  name: string
  description: string
  sizeGB: number
  ramRequiredGB: number
  url: string
  mirrors: string[]      // Extra origin URLs tried in order if the first fails
  sha256: string         // Empty string = skip checksum (GGUF header and size are always checked)
  filename: string
  contextLength: number
  tags: string[]
  recommended: boolean
}

// Keep this list in sync with get_model_registry() in src-tauri/src/main.rs.
// The backend is the source of truth for download URLs; the UI only sends the id.
// Every URL was checked against the public Hugging Face file listing.
export const MODEL_REGISTRY: ModelInfo[] = [
  {
    id: 'qwen2.5-coder-1.5b',
    name: 'Qwen2.5 Coder 1.5B',
    description: 'Fast, efficient code model from Alibaba (Apache-2.0). Great for Python, JS, TS, Java.',
    sizeGB: 1.12,
    ramRequiredGB: 3,
    url: 'https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf',
    mirrors: [],
    sha256: '',
    filename: 'qwen2.5-coder-1.5b-instruct-q4_k_m.gguf',
    contextLength: 32768,
    tags: ['code', 'fast', 'low-ram'],
    recommended: true,
  },
  {
    id: 'deepseek-coder-1.3b',
    name: 'DeepSeek Coder 1.3B',
    description: 'Strong code completion from DeepSeek (community GGUF by TheBloke, DeepSeek license).',
    sizeGB: 0.87,
    ramRequiredGB: 2.5,
    url: 'https://huggingface.co/TheBloke/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct.Q4_K_M.gguf',
    mirrors: [],
    sha256: '',
    filename: 'deepseek-coder-1.3b-instruct.Q4_K_M.gguf',
    contextLength: 16384,
    tags: ['code', 'infilling', 'low-ram'],
    recommended: true,
  },
  {
    id: 'phi-3-mini',
    name: 'Phi-3 Mini 4K',
    description: "Microsoft's small general-purpose model (4-bit GGUF from the official repo).",
    sizeGB: 2.39,
    ramRequiredGB: 4,
    url: 'https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf',
    mirrors: [],
    sha256: '',
    filename: 'Phi-3-mini-4k-instruct-q4.gguf',
    contextLength: 4096,
    tags: ['reasoning', 'chat', 'general'],
    recommended: true,
  },
]

export function getModelById(id: string): ModelInfo | undefined {
  return MODEL_REGISTRY.find(m => m.id === id)
}

export function getRecommendedModels(): ModelInfo[] {
  return MODEL_REGISTRY.filter(m => m.recommended)
}

export function getModelsForRAM(ramGB: number): ModelInfo[] {
  return MODEL_REGISTRY.filter(m => m.ramRequiredGB <= ramGB * 0.7) // Leave 30% for OS
}

export function getAllModelUrls(model: ModelInfo): string[] {
  return [model.url, ...model.mirrors]
}