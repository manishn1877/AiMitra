import { useState, useEffect } from 'react'
import { Save, Loader2, Cpu, MemoryStick, Globe, Palette, Battery, Thermometer, Zap, Shield } from 'lucide-react'
import { cn } from '../lib/utils'
import { invoke } from '@tauri-apps/api/core'

interface Settings {
  // Model settings
  model: string
  contextLength: number
  temperature: number
  topP: number
  maxTokens: number

  // System settings
  threads: number
  gpuLayers: number
  batchSize: number
  quantizeKvCache: boolean
  lowRamMode: boolean

  // UI settings
  theme: 'light' | 'dark' | 'system'
  autoScroll: boolean
  showTiming: boolean

  // Network settings
  port: number
  enableApi: boolean

  // Power & Thermal settings
  powerProfile: 'battery_saver' | 'balanced' | 'performance'
  maxCpuPercent: number
  thermalThrottleEnabled: boolean
  autoUnloadIdleMinutes: number
  unloadOnMinimize: boolean
}

const DEFAULT_SETTINGS: Settings = {
  model: '',
  contextLength: 4096,
  temperature: 0.2,
  topP: 0.95,
  maxTokens: 2048,
  threads: 0, // 0 = auto
  gpuLayers: 0,
  batchSize: 512,
  quantizeKvCache: true,
  lowRamMode: false,
  theme: 'system',
  autoScroll: true,
  showTiming: true,
  port: 11434,
  enableApi: true,
  powerProfile: 'balanced',
  maxCpuPercent: 80,
  thermalThrottleEnabled: true,
  autoUnloadIdleMinutes: 30,
  unloadOnMinimize: false,
}

export function SettingsPanel() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS)
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)
  const [systemInfo, setSystemInfo] = useState<{ ramGB: number; cpuCores: number; gpu: string } | null>(null)
  const [thermalState, setThermalState] = useState<{ thermal_state: string; cpu_temp: number; cpu_usage: number } | null>(null)

  useEffect(() => {
    loadSettings()
    loadSystemInfo()
  }, [])

  const loadSettings = async () => {
    try {
      const stored = await invoke<Settings>('get_settings')
      setSettings(stored)
    } catch {
      // Use defaults
    }
  }

  const loadThermalState = async () => {
    try {
      const thermal = await invoke<{ thermal_state: string; cpu_temp: number; cpu_usage: number }>('get_thermal_state')
      setThermalState(thermal)
    } catch {
      // Thermal monitoring may not be available
    }
  }

  // Refresh thermal state periodically
  useEffect(() => {
    loadThermalState()
    const interval = setInterval(loadThermalState, 5000)
    return () => clearInterval(interval)
  }, [])

  const loadSystemInfo = async () => {
    try {
      const info = await invoke<{ ramGB: number; cpuCores: number; gpu: string }>('get_system_info')
      setSystemInfo(info)
      if (settings.threads === 0) {
        setSettings(prev => ({ ...prev, threads: info.cpuCores }))
      }
    } catch {
      console.error('Failed to load system info')
    }
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      await invoke('save_settings', { settings })
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (err) {
      console.error('Save failed:', err)
    } finally {
      setSaving(false)
    }
  }

  const handleChange = (key: keyof Settings, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }))
  }

  return (
    <div className="card p-4 max-w-2xl">
      <div className="flex items-center justify-between mb-6">
        <h3 className="font-semibold text-slate-900 dark:text-slate-100">Settings</h3>
        <button
          onClick={handleSave}
          disabled={saving}
          className="btn-primary"
        >
          {saving ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Saving...
            </>
          ) : saved ? (
            <>
              <Save className="w-4 h-4" />
              Saved
            </>
          ) : (
            <>
              <Save className="w-4 h-4" />
              Save Settings
            </>
          )}
        </button>
      </div>

      {/* System Info */}
      {systemInfo && (
        <div className="mb-6 p-3 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
          <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-2 flex items-center gap-2">
            <Cpu className="w-4 h-4" />
            System Information
          </h4>
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <span className="text-slate-500 dark:text-slate-400">RAM</span>
              <div className="font-mono text-slate-900 dark:text-slate-100">{systemInfo.ramGB} GB</div>
            </div>
            <div>
              <span className="text-slate-500 dark:text-slate-400">CPU Cores</span>
              <div className="font-mono text-slate-900 dark:text-slate-100">{systemInfo.cpuCores}</div>
            </div>
            <div>
              <span className="text-slate-500 dark:text-slate-400">GPU</span>
              <div className="font-mono text-slate-900 dark:text-slate-100 truncate max-w-[120px]">{systemInfo.gpu || 'None detected'}</div>
            </div>
          </div>
        </div>
      )}

      <div className="space-y-6">
        {/* Model Parameters */}
        <section>
          <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-4 flex items-center gap-2">
            <Palette className="w-4 h-4" />
            Model Parameters
          </h4>
          <div className="space-y-4">
            <SettingSlider
              label="Context Length"
              value={settings.contextLength}
              min={512}
              max={16384}
              step={512}
              onChange={(v) => handleChange('contextLength', v)}
              marks={[
                { value: 512, label: '512' },
                { value: 2048, label: '2K' },
                { value: 4096, label: '4K' },
                { value: 8192, label: '8K' },
                { value: 16384, label: '16K' },
              ]}
              help="Maximum tokens in conversation context. Larger = more memory."
            />
            <SettingSlider
              label="Temperature"
              value={settings.temperature}
              min={0}
              max={2}
              step={0.1}
              onChange={(v) => handleChange('temperature', v)}
              marks={[
                { value: 0, label: '0.0' },
                { value: 0.5, label: '0.5' },
                { value: 1, label: '1.0' },
                { value: 1.5, label: '1.5' },
                { value: 2, label: '2.0' },
              ]}
              help="Creativity vs determinism. Lower = more focused code."
            />
            <SettingSlider
              label="Top-P"
              value={settings.topP}
              min={0.1}
              max={1}
              step={0.05}
              onChange={(v) => handleChange('topP', v)}
              help="Nucleus sampling. Lower = more focused."
            />
            <SettingSlider
              label="Max Tokens"
              value={settings.maxTokens}
              min={128}
              max={8192}
              step={128}
              onChange={(v) => handleChange('maxTokens', v)}
              marks={[
                { value: 512, label: '512' },
                { value: 1024, label: '1K' },
                { value: 2048, label: '2K' },
                { value: 4096, label: '4K' },
              ]}
              help="Maximum tokens to generate per response."
            />
          </div>
        </section>

        {/* System Performance */}
        <section>
          <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-4 flex items-center gap-2">
            <MemoryStick className="w-4 h-4" />
            System Performance
          </h4>
          <div className="space-y-4">
            <SettingSelect
              label="CPU Threads"
              value={settings.threads}
              options={systemInfo ? Array.from({ length: systemInfo.cpuCores }, (_, i) => i + 1).concat(0) : [0, 1, 2, 4, 8, 16]}
              optionLabels={systemInfo ? Array.from({ length: systemInfo.cpuCores }, (_, i) => `${i + 1}`).concat(['Auto']) : ['Auto', '1', '2', '4', '8', '16']}
              onChange={(v) => handleChange('threads', v)}
              help={systemInfo ? `Your CPU has ${systemInfo.cpuCores} cores. 0 = auto.` : '0 = auto-detect'}
            />
            <SettingSlider
              label="GPU Layers"
              value={settings.gpuLayers}
              min={0}
              max={systemInfo && systemInfo.gpu !== 'None detected' ? 99 : 0}
              step={1}
              onChange={(v) => handleChange('gpuLayers', v)}
              disabled={!systemInfo || systemInfo.gpu === 'None detected'}
              help="Layers to offload to GPU. 0 = CPU only. Requires compatible GPU."
            />
            <SettingSlider
              label="Batch Size"
              value={settings.batchSize}
              min={128}
              max={2048}
              step={128}
              onChange={(v) => handleChange('batchSize', v)}
              marks={[
                { value: 128, label: '128' },
                { value: 512, label: '512' },
                { value: 1024, label: '1K' },
                { value: 2048, label: '2K' },
              ]}
              help="Tokens processed in parallel. Higher = faster prompt processing, more VRAM/RAM."
            />
            <SettingToggle
              label="Quantize KV Cache"
              value={settings.quantizeKvCache}
              onChange={(v) => handleChange('quantizeKvCache', v)}
              help="Quantize key-value cache to 8-bit. Reduces RAM usage ~30% with minimal quality loss."
            />
            <SettingToggle
              label="Low RAM Mode"
              value={settings.lowRamMode}
              onChange={(v) => handleChange('lowRamMode', v)}
              help="Optimize for low memory systems. Uses smaller buffers and disables some features."
            />
          </div>
        </section>

        {/* Power & Thermal Management */}
        <section>
          <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-4 flex items-center gap-2">
            <Battery className="w-4 h-4" />
            Power & Thermal Management
          </h4>
          {thermalState && (
            <div className="mb-4 p-3 bg-slate-50 dark:bg-slate-800/50 rounded-lg">
              <h5 className="font-medium text-slate-900 dark:text-slate-100 mb-2 flex items-center gap-2">
                <Thermometer className="w-4 h-4" />
                Current Thermal State
              </h5>
              <div className="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <span className="text-slate-500 dark:text-slate-400">State</span>
                  <div className="font-mono text-slate-900 dark:text-slate-100 flex items-center gap-2">
                    <span className={cn(
                      'px-2 py-0.5 rounded text-xs font-medium',
                      thermalState.thermal_state === 'throttling' && 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
                      thermalState.thermal_state === 'hot' && 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400',
                      thermalState.thermal_state === 'warm' && 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
                      thermalState.thermal_state === 'normal' && 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                    )}>
                      {thermalState.thermal_state.toUpperCase()}
                    </span>
                  </div>
                </div>
                <div>
                  <span className="text-slate-500 dark:text-slate-400">CPU Temp</span>
                  <div className="font-mono text-slate-900 dark:text-slate-100">{thermalState.cpu_temp.toFixed(1)}°C</div>
                </div>
                <div>
                  <span className="text-slate-500 dark:text-slate-400">CPU Usage</span>
                  <div className="font-mono text-slate-900 dark:text-slate-100">{thermalState.cpu_usage.toFixed(1)}%</div>
                </div>
              </div>
            </div>
          )}
          <div className="space-y-4">
            <SettingSelect
              label="Power Profile"
              value={settings.powerProfile}
              options={['battery_saver', 'balanced', 'performance']}
              optionLabels={['🔋 Battery Saver', '⚖️ Balanced', '⚡ Performance']}
              onChange={(v) => {
                handleChange('powerProfile', v as Settings['powerProfile'])
                // Apply profile immediately
                invoke('set_power_profile', { profile: v })
              }}
              help="Quick presets that adjust multiple settings for different use cases."
            />
            <SettingSlider
              label="Max CPU Usage"
              value={settings.maxCpuPercent}
              min={10}
              max={100}
              step={5}
              onChange={(v) => {
                handleChange('maxCpuPercent', v)
                invoke('set_max_cpu_percent', { percent: v })
              }}
              marks={[
                { value: 25, label: '25%' },
                { value: 50, label: '50%' },
                { value: 75, label: '75%' },
                { value: 100, label: '100%' },
              ]}
              help="Limit CPU usage for llama.cpp process. 100 = unlimited. Lower = cooler, slower."
            />
            <SettingToggle
              label="Enable Thermal Throttling"
              value={settings.thermalThrottleEnabled}
              onChange={(v) => {
                handleChange('thermalThrottleEnabled', v)
                invoke('set_thermal_throttle', { enabled: v })
              }}
              help="Automatically reduce CPU priority/threads when laptop gets hot."
            />
            <SettingSlider
              label="Auto-Unload After Idle (minutes)"
              value={settings.autoUnloadIdleMinutes}
              min={0}
              max={120}
              step={5}
              onChange={(v) => {
                handleChange('autoUnloadIdleMinutes', v)
                invoke('set_auto_unload', { minutes: v })
              }}
              marks={[
                { value: 0, label: 'Off' },
                { value: 15, label: '15m' },
                { value: 30, label: '30m' },
                { value: 60, label: '1h' },
                { value: 120, label: '2h' },
              ]}
              help="Unload model from RAM after idle time. 0 = disabled. Saves RAM when not coding."
            />
            <SettingToggle
              label="Unload on Window Minimize"
              value={settings.unloadOnMinimize}
              onChange={(v) => {
                handleChange('unloadOnMinimize', v)
                invoke('set_unload_on_minimize', { enabled: v })
              }}
              help="Free RAM immediately when app is minimized. Reloads when restored."
            />
          </div>
        </section>

        {/* UI Preferences */}
        <section>
          <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-4 flex items-center gap-2">
            <Globe className="w-4 h-4" />
            Interface
          </h4>
          <div className="space-y-4">
            <SettingSelect
              label="Theme"
              value={settings.theme}
              options={['light', 'dark', 'system']}
              optionLabels={['Light', 'Dark', 'System']}
              onChange={(v) => handleChange('theme', v as Settings['theme'])}
            />
            <SettingToggle
              label="Auto-scroll to latest message"
              value={settings.autoScroll}
              onChange={(v) => handleChange('autoScroll', v)}
            />
            <SettingToggle
              label="Show generation timing"
              value={settings.showTiming}
              onChange={(v) => handleChange('showTiming', v)}
            />
          </div>
        </section>

        {/* API Server */}
        <section>
          <h4 className="font-medium text-slate-900 dark:text-slate-100 mb-4 flex items-center gap-2">
            <Cpu className="w-4 h-4" />
            Local API Server
          </h4>
          <div className="space-y-4">
            <SettingToggle
              label="Enable OpenAI-compatible API"
              value={settings.enableApi}
              onChange={(v) => handleChange('enableApi', v)}
              help="Allows other tools (VS Code extensions, etc.) to connect at http://localhost:PORT"
            />
            <SettingInput
              label="Port"
              type="number"
              value={settings.port}
              onChange={(v) => handleChange('port', v)}
              min={1024}
              max={65535}
              disabled={!settings.enableApi}
            />
          </div>
        </section>
      </div>
    </div>
  )
}

function SettingSlider({
  label,
  value,
  min,
  max,
  step,
  onChange,
  marks,
  help,
  disabled = false,
}: {
  label: string
  value: number
  min: number
  max: number
  step: number
  onChange: (value: number) => void
  marks?: { value: number; label: string }[]
  help?: string
  disabled?: boolean
}) {
  return (
    <div className={cn('space-y-2', disabled && 'opacity-50')}>
      <label className="flex items-center justify-between text-sm">
        <span className="text-slate-700 dark:text-slate-300">{label}</span>
        <span className="font-mono text-brand-600 dark:text-brand-400">{value}</span>
      </label>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        disabled={disabled}
        className="w-full h-2 bg-slate-200 dark:bg-slate-700 rounded-lg appearance-none accent-brand-500 cursor-pointer disabled:cursor-not-allowed"
      />
      {marks && (
        <div className="flex justify-between text-xs text-slate-500 dark:text-slate-400">
          {marks.map((mark, i) => (
            <span key={i} style={{ left: `${((mark.value - min) / (max - min)) * 100}%` }}>{mark.label}</span>
          ))}
        </div>
      )}
      {help && <p className="text-xs text-slate-500 dark:text-slate-400">{help}</p>}
    </div>
  )
}

function SettingSelect({
  label,
  value,
  options,
  optionLabels,
  onChange,
  help,
  disabled = false,
}: {
  label: string
  value: string | number
  options: (string | number)[]
  optionLabels: string[]
  onChange: (value: string | number) => void
  help?: string
  disabled?: boolean
}) {
  return (
    <div className={cn('space-y-2', disabled && 'opacity-50')}>
      <label className="text-sm text-slate-700 dark:text-slate-300">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        className="input w-full max-w-xs"
      >
        {options.map((opt, i) => (
          <option key={i} value={opt}>{optionLabels[i]}</option>
        ))}
      </select>
      {help && <p className="text-xs text-slate-500 dark:text-slate-400">{help}</p>}
    </div>
  )
}

function SettingToggle({
  label,
  value,
  onChange,
  help,
}: {
  label: string
  value: boolean
  onChange: (value: boolean) => void
  help?: string
}) {
  return (
    <div className="space-y-1">
      <label className="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          checked={value}
          onChange={(e) => onChange(e.target.checked)}
          className="w-4 h-4 rounded border-slate-300 dark:border-slate-600 text-brand-600 focus:ring-brand-500"
        />
        <span className="text-sm text-slate-700 dark:text-slate-300">{label}</span>
      </label>
      {help && <p className="ml-7 text-xs text-slate-500 dark:text-slate-400">{help}</p>}
    </div>
  )
}

function SettingInput({
  label,
  type = 'text',
  value,
  onChange,
  min,
  max,
  disabled = false,
}: {
  label: string
  type?: string
  value: string | number
  onChange: (value: number) => void
  min?: number
  max?: number
  disabled?: boolean
}) {
  return (
    <div className={cn('space-y-2', disabled && 'opacity-50')}>
      <label className="text-sm text-slate-700 dark:text-slate-300">{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        min={min}
        max={max}
        disabled={disabled}
        className="input w-full max-w-xs"
      />
    </div>
  )
}