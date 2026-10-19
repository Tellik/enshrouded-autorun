export type NoticeAction = { id: string; kind: string; target?: string }

export type NoticeLink = { labelKey?: string; label?: string; url: string }

export type NoticeBanner = {
  imageUrl: string
  linkUrl: string
  alt: string
}

export type GitHubContributor = {
  login: string
  html_url: string
  contributions: number
}

export type Notice = {
  id: string
  modId: string
  ecosystem?: 'EML' | 'SF'
  title: string
  message: string
  level: string
  actionUrl?: string
  actionLabelKey?: string
  links?: NoticeLink[]
  banner?: NoticeBanner
  actions?: NoticeAction[]
  updatedAt: number
  kind?: string
  values?: Record<string, string | number>
  changelog?: string[]
}

export type NewsItem = {
  id: string
  modId?: string
  ecosystem?: 'EML' | 'SF'
  level: string
  title: string
  message: string
  actionUrl?: string
  actionLabelKey?: string
  links?: NoticeLink[]
  banner?: NoticeBanner
  actions?: NoticeAction[]
  updatedAt: number
  kind?: string
  values?: Record<string, string | number>
  changelog?: string[]
}

export type NewsPersonalization = {
  usernameMode: 'auto' | 'manual'
  detectedUsername?: string | null
  messageNameFallback: string
}
export type NewsDisplaySource = {
  notices: Notice[]
  newsTemplates?: Record<string, { title: string; message: string }>
  mods: Array<{ id: string; ecosystem: 'EML' | 'SF' }>
  release: {
    updateAvailable: boolean
    latestVersion?: string
    message?: string
    releaseUrl?: string
    staged: boolean
  }
}
export type CommandResult = {
  requestId: string
  success: boolean
  message: string
  batchKey?: string
  restoreSettings?: boolean
  ecosystem?: 'EML' | 'SF' | null
}
