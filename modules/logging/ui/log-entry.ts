export type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace'

export type Activity = {
  id: string
  time: string
  source: string
  action: string
  result: string
  level: LogLevel
}
