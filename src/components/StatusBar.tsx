import { useEffect, useState } from 'react'
import { Cpu, MemoryStick, Wifi, WifiOff, CheckCircle, Loader2, Zap, Thermometer, AlertTriangle } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { cn, formatBytes } from '../lib/utils'

interface SystemStats {
  ramUsed: number
  ramTotal: number
  cpuUsage: number
  modelLoaded: boolean
  modelName?: string
  tokensPerSecond?: number
}

interface ThermalState {
  thermal_state: string
  cpu_temp: number
  cpu_usage: number
  power_profile: string
  max_cpu_percent: number
}

export function StatusBar() {
  const [stats, setStats] = useState<SystemStats>({
    ramUsed: 0,
    ramTotal: 0,
    cpuUsage: 0,
    modelLoaded: false,
  })
  const [online, setOnline] = useState(true)
  const [thermal, setThermal] = useState<ThermalState | null>(null)

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const [sysStats, isOnline, thermalState] = await Promise.all([
          invoke<SystemStats>('get_system_stats'),
          invoke<boolean>('check_internet'),
          invoke<ThermalState>('get_thermal_state').catch(() => null),
        ])
        setStats(sysStats)
        setOnline(isOnline)
        if (thermalState) setThermal(thermalState)
      } catch {
        // Ignore errors
      }
    }, 2000)

    // Initial load
    invoke<SystemStats>('get_system_stats').then(setStats).catch(console.error)
    invoke<boolean>('check_internet').then(setOnline).catch(console.error)
    invoke<ThermalState>('get_thermal_state').then(setThermal).catch(() => {})

    return () => clearInterval(interval)
  }, [])

  const ramPercent = stats.ramTotal > 0 ? (stats.ramUsed / stats.ramTotal) * 100 : 0
  const ramColor = ramPercent > 85 ? 'text-red-500' : ramPercent > 70 ? 'text-amber-500' : 'text-green-500'
  const cpuColor = stats.cpuUsage > 85 ? 'text-red-500' : stats.cpuUsage > 70 ? 'text-amber-500' : 'text-green-500'

  // Thermal state colors and icons
  const getThermalColors = (state: string) => {
    switch (state) {
      case 'throttling': return { text: 'text-red-500', bg: 'bg-red-500', icon: <AlertTriangle className="w-3.5 h-3.5" /> }
      case 'hot': return { text: 'text-orange-500', bg: 'bg-orange-500', icon: <Thermometer className="w-3.5 h-3.5" /> }
      case 'warm': return { text: 'text-amber-500', bg: 'bg-amber-500', icon: <Thermometer className="w-3.5 h-3.5" /> }
      default: return { text: 'text-green-500', bg: 'bg-green-500', icon: <Thermometer className="w-3.5 h-3.5" /> }
    }
  }

  const thermalColors = thermal ? getThermalColors(thermal.thermal_state) : { text: 'text-slate-500', bg: 'bg-slate-500', icon: <Thermometer className="w-3.5 h-3.5" /> }

  return (
    <div className="flex items-center justify-between px-4 py-2 border-t border-slate-200 dark:border-slate-700 bg-white/50 dark:bg-slate-900/50 backdrop-blur-sm">
      <div className="flex items-center gap-4 text-xs">
        {/* Model Status */}
        <div className={cn('flex items-center gap-1.5', stats.modelLoaded ? 'text-green-600 dark:text-green-400' : 'text-slate-500 dark:text-slate-400')}>
          {stats.modelLoaded ? (
            <>
              <CheckCircle className="w-3.5 h-3.5" />
              <span className="font-mono text-xs truncate max-w-[150px]">{stats.modelName}</span>
            </>
          ) : (
            <>
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              <span>No model loaded</span>
            </>
          )}
        </div>

        {/* Performance */}
        {stats.modelLoaded && stats.tokensPerSecond && (
          <div className="flex items-center gap-1.5 text-slate-600 dark:text-slate-400">
            <Zap className="w-3.5 h-3.5" />
            <span className="font-mono">{stats.tokensPerSecond.toFixed(1)} tok/s</span>
          </div>
        )}

        {/* Thermal State */}
        <div className="hidden md:flex items-center gap-1.5" title={thermal ? `Thermal: ${thermal.thermal_state} | CPU: ${thermal.cpu_temp.toFixed(1)}°C | Profile: ${thermal.power_profile}` : 'Thermal monitoring...'}>
          {thermalColors.icon}
          <span className={cn('font-mono font-medium', thermalColors.text)}>
            {thermal ? thermal.thermal_state.toUpperCase() : 'NORMAL'}
          </span>
          {thermal && thermal.cpu_temp > 0 && (
            <span className={cn('font-mono', thermalColors.text)}>
              {thermal.cpu_temp.toFixed(0)}°C
            </span>
          )}
        </div>

        <div className="hidden md:flex items-center gap-1.5">
          <MemoryStick className={cn('w-3.5 h-3.5', ramColor)} />
          <span className="font-mono">{formatBytes(stats.ramUsed)} / {formatBytes(stats.ramTotal)}</span>
          <div className="w-20 h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden ml-1">
            <div
              className={cn('h-full rounded-full transition-all duration-500', ramColor.replace('text-', 'bg-'))}
              style={{ width: `${Math.min(ramPercent, 100)}%` }}
            />
          </div>
        </div>

        <div className="hidden lg:flex items-center gap-1.5">
          <Cpu className={cn('w-3.5 h-3.5', cpuColor)} />
          <span className="font-mono">{stats.cpuUsage.toFixed(0)}%</span>
          <div className="w-16 h-1.5 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden ml-1">
            <div
              className={cn('h-full rounded-full transition-all duration-500', cpuColor.replace('text-', 'bg-'))}
              style={{ width: `${Math.min(stats.cpuUsage, 100)}%` }}
            />
          </div>
        </div>
      </div>

      <div className="flex items-center gap-3 text-xs">
        <div className={cn('flex items-center gap-1.5', online ? 'text-green-600 dark:text-green-400' : 'text-slate-500 dark:text-slate-400')}>
          {online ? <Wifi className="w-3.5 h-3.5" /> : <WifiOff className="w-3.5 h-3.5" />}
          <span className="hidden sm:inline">{online ? 'Online' : 'Offline'}</span>
        </div>
        {thermal && (
          <div className="hidden sm:flex items-center gap-1.5 text-slate-500 dark:text-slate-400">
            <span className="font-mono text-xs">{thermal.power_profile.replace('_', ' ')}</span>
          </div>
        )}
      </div>
    </div>
  )
}