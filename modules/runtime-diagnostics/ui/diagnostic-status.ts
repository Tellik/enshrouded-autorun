export type DiagnosticStatus = {
  active: boolean
  fresh: boolean
  reason: string
  jobId?: string | null
  remainingSeconds?: number
  lastSampleAt?: number
  detail?: string
  report?: Record<string, unknown>
}
