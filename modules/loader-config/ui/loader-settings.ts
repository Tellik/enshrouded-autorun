export type ModulePreferences = Record<string, Record<string, any>>

export type StoragePath = { id: string; path: string; defaultPath: string }
export type DiagnosticJob = {
  id: string
  name: string
  description: string
  capabilities: string[]
}

export type Settings = {
  configRevision?: string
  modulePreferences?: ModulePreferences
  compactMode: boolean
  reducedMotion: boolean
  zoomPercent: number
  usernameMode: 'auto' | 'manual'
  detectedUsername?: string | null
  effectiveUsername: string
  usernameSource: 'live' | 'saved' | 'fallback'
  messageNameFallback: string
  logLevel: string
  exportEnabled: boolean
  runtimeProfileId?: string | null
  activeRuntimeProfile?: string
  runtimeProfiles?: string[]
  runtimeProfileDirectory?: string
  diagnosticJobs?: DiagnosticJob[]
  diagnosticJobErrors?: string[]
  diagnosticJobDirectory?: string
  diagnosticSelectedJobId?: string
  paths?: StoragePath[]
  baseUrl: string
  projectId: string
}
