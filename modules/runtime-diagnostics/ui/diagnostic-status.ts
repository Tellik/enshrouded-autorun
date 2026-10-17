export type DiagnosticStatus = {
  active: boolean
  fresh: boolean
  reason: string
  remainingSeconds?: number
  lastSampleAt?: number
  detail?: string
  report?: Record<string, unknown>
}
