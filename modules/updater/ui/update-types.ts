export type Release = {
  currentVersion: string
  updaterAvailable?: boolean
  latestVersion?: string
  updateAvailable: boolean
  mandatory?: boolean
  prerelease?: boolean
  state: string
  message?: string
  releaseUrl?: string
  staged: boolean
  downloadedBytes?: number
  totalBytes?: number
  bytesPerSecond?: number
}

export type UpdateQueueItem = {
  id: string
  kind: 'system' | 'mod' | 'maintenance'
  operation?: string
  title: string
  version?: string
  prerelease?: boolean
  state: string
  selected: boolean
  required?: boolean
  message?: string
  releaseUrl?: string
}

export type UpdateQueue = {
  items: UpdateQueueItem[]
  error?: string
  run?: {
    state: string
    waitForGame?: boolean
    itemIds?: string[]
    error?: string
  } | null
}

export type CatalogItem = {
  id: string
  slug: string
  name: string
  summary: string
  iconUrl?: string
  author?: string
  downloads?: number
  version?: string
}

export type Catalog = {
  state: string
  query: string
  message?: string
  items: CatalogItem[]
}

export type ModUpdate = {
  projectId: string
  modId: string
  name: string
  currentVersion?: string
  latestVersion?: string
  latestVersionId: string
}

export type ModUpdateCheckResult = {
  checked: number
  failed: number
  checkedAt?: number
}

export type UpdaterViewData = {
  mode: string
  release: Release
  enabled?: boolean
  updateQueue?: UpdateQueue
  modUpdates?: ModUpdate[]
  modCheckResult?: ModUpdateCheckResult | null
  modCheckRunning?: boolean
}

export type UpdaterCommand = (
  command: string,
  payload?: Record<string, unknown>,
) => void
