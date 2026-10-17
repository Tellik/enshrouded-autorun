import { useI18n } from './i18n'

export function ModeBadge({
  mode,
  connected,
}: {
  mode: string
  connected: boolean
}) {
  const { t } = useI18n()
  const normalized = mode === 'OFFLINE' ? 'DESKTOP' : mode
  const desktop = normalized === 'DESKTOP'
  const stateClass = desktop ? 'desktop' : connected ? 'online' : 'offline'
  return (
    <div
      className={`mode-badge ${stateClass}`}
      title={
        desktop
          ? t('status.runtime.desktop')
          : connected
            ? t('status.runtime.connected')
            : t('status.runtime.disconnected')
      }
    >
      <i />
      <span>{normalized}</span>
    </div>
  )
}
