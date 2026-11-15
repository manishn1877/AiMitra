export interface ModelInfo {
  id: string
  name: string
  description: string
  sizeGB: number
  ramRequiredGB: number
  url: string
  mirrors: string[]      // Fallback mirrors for restricted networks
  sha256: string         // Empty string = skip verification
  filename: string
  contextLength: number
  tags: string[]
  recommended: boolean
}

export const MODEL_REGISTRY: ModelInfo[] = [
  {
    id: 'codeqwen-1.5b',
    name: 'CodeQwen 1.5B',
    description: 'Fast, efficient code model from Alibaba. Great for Python, JS, TS, Java.',
    sizeGB: 1.1,
    ramRequiredGB: 3,
    url: 'https://huggingface.co/Qwen/CodeQwen1.5-1.5B-GGUF/resolve/main/codeqwen1.5-1.5b-q4_k_m.gguf',
    mirrors: [
      'https://hf-mirror.com/Qwen/CodeQwen1.5-1.5B-GGUF/resolve/main/codeqwen1.5-1.5b-q4_k_m.gguf',
      'https://mirror.ghproxy.com/https://huggingface.co/Qwen/CodeQwen1.5-1.5B-GGUF/resolve/main/codeqwen1.5-1.5b-q4_k_m.gguf',
    ],
    sha256: '', // TODO: Run `sha256sum <file>` after download and update
    filename: 'codeqwen1.5-1.5b-q4_k_m.gguf',
    contextLength: 8192,
    tags: ['code', 'fast', 'low-ram'],
    recommended: true,
  },
  {
    id: 'deepseek-coder-1.3b',
    name: 'DeepSeek Coder 1.3B',
    description: 'Strong code completion from DeepSeek. Excellent for infilling.',
    sizeGB: 0.9,
    ramRequiredGB: 2.5,
    url: 'https://huggingface.co/deepseek-ai/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct-q4_k_m.gguf',
    mirrors: [
      'https://hf-mirror.com/deepseek-ai/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct-q4_k_m.gguf',
      'https://mirror.ghproxy.com/https://huggingface.co/deepseek-ai/deepseek-coder-1.3b-instruct-GGUF/resolve/main/deepseek-coder-1.3b-instruct-q4_k_m.gguf',
    ],
    sha256: '', // TODO: Run `sha256sum <file>` after download and update
    filename: 'deepseek-coder-1.3b-instruct-q4_k_m.gguf',
    contextLength: 16384,
    tags: ['code', 'infilling', 'low-ram'],
    recommended: true,
  },
  {
    id: 'phi-3-mini',
    name: 'Phi-3 Mini 4K',
    description: "Microsoft's small but mighty model. Best reasoning in class.",
    sizeGB: 2.3,
    ramRequiredGB: 4,
    url: 'https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf',
    mirrors: [
      'https://hf-mirror.com/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf',
      'https://mirror.ghproxy.com/https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf',
    ],
    sha256: '', // TODO: Run `sha256sum <file>` after download and update
    filename: 'phi-3-mini-4k-instruct-q4_k_m.gguf',
    contextLength: 4096,
    tags: ['reasoning', 'chat', 'general'],
    recommended: true,
  },
  {
    id: 'stable-code-3b',
    name: 'Stable Code 3B',
    description: "Stability AI's code model. Good balance of size and capability.",
    sizeGB: 1.8,
    ramRequiredGB: 3.5,
    url: 'https://huggingface.co/stabilityai/stable-code-3b-GGUF/resolve/main/stable-code-3b-q4_k_m.gguf',
    mirrors: [
      'https://hf-mirror.com/stabilityai/stable-code-3b-GGUF/resolve/main/stable-code-3b-q4_k_m.gguf',
      'https://mirror.ghproxy.com/https://huggingface.co/stabilityai/stable-code-3b-GGUF/resolve/main/stable-code-3b-q4_k_m.gguf',
    ],
    sha256: '', // TODO: Run `sha256sum <file>` after download and update
    filename: 'stable-code-3b-q4_k_m.gguf',
    contextLength: 16384,
    tags: ['code', 'fill-in-middle'],
    recommended: false,
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