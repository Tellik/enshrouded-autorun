export type Release = {
  currentVersion: string
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
  kind: 'system' | 'mod'
  operation?: string
  title: string
  version?: string
  prerelease?: boolean
  state: string
  selected: boolean
  message?: string
  releaseUrl?: string
}

export type UpdateQueue = {
  items: UpdateQueueItem[]
  run?: { state: string; waitForGame?: boolean; itemIds?: string[] } | null
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
