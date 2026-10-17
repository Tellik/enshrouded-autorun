export type ModulePreferences = Record<string, Record<string, any>>

export type StoragePath = { id: string; path: string; defaultPath: string }

export type Settings = {
  configRevision?: string
  modulePreferences?: ModulePreferences
  compactMode: boolean
  reducedMotion: boolean
  usernameMode: 'auto' | 'manual'
  detectedUsername?: string | null
  messageNameFallback: string
  logLevel: string
  exportEnabled: boolean
  runtimeProfileId?: string | null
  activeRuntimeProfile?: string
  runtimeProfiles?: string[]
  runtimeProfileDirectory?: string
  paths?: StoragePath[]
  updateEnabled: boolean
  baseUrl: string
  projectId: string
  checkMinutes: number
}
