import { postRequest } from '../../transport/ui/send-host-command'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { DiagnosticStatus } from './diagnostic-status'

export function DiagnosticsControls({
  data,
  prefs,
  module,
  change,
}: {
  data: { diagnostics?: DiagnosticStatus }
  prefs: Record<string, Record<string, any>>
  module: string
  change(module: string, key: string, value: unknown): void
}) {
  const { t } = useI18n()
  const diagnosticAreas = Array.isArray(prefs.runtimeDiagnostics?.areas)
    ? prefs.runtimeDiagnostics.areas
    : []
  const statusText = data.diagnostics?.fresh
    ? data.diagnostics.active
      ? `${t('settings.modules.diagnosticActive')} · ${data.diagnostics.remainingSeconds}s`
      : data.diagnostics.reason
    : t('settings.modules.diagnosticUnavailable')
  return (
    <div className="settings-diagnostics">
      <p className="hint">{t('settings.modules.diagnosticAreas')}</p>
      <div className="settings-area-list">
        {['runtime', 'queue', 'mods'].map((area) => (
          <label key={area}>
            <input
              className="settings-checkbox"
              type="checkbox"
              checked={diagnosticAreas.includes(area)}
              onChange={(e) =>
                change(
                  module,
                  'areas',
                  e.target.checked
                    ? [...diagnosticAreas, area]
                    : diagnosticAreas.filter((value: string) => value !== area),
                )
              }
            />
            {area}
          </label>
        ))}
      </div>
      <p className="hint">
        {statusText}
        {' · '}
        {t('settings.modules.lastSample')}
        {': '}
        {data.diagnostics?.lastSampleAt
          ? new Date(data.diagnostics.lastSampleAt * 1000).toLocaleString()
          : '—'}
      </p>
      <p className="hint">{t('settings.modules.saveHint')}</p>
      <div className="settings-diagnostic-actions">
        {(['start', 'stop', 'snapshot'] as const).map((action) => (
          <button
            className="button ghost"
            key={action}
            onClick={() => postRequest('diagnostics', { action })}
          >
            {t(`settings.modules.${action}`)}
          </button>
        ))}
      </div>
      {data.diagnostics?.report && (
        <pre className="diagnostic-report">
          {JSON.stringify(data.diagnostics.report, null, 2)}
        </pre>
      )}
    </div>
  )
}

export const diagnosticSettingsFields: Array<
  [string, string, TranslationKey, string[]?]
> = [
  [
    'runtimeDiagnostics',
    'intervalMilliseconds',
    'settings.modules.field.interval',
  ],
  [
    'runtimeDiagnostics',
    'maximumDurationSeconds',
    'settings.modules.field.duration',
  ],
  [
    'runtimeDiagnostics',
    'slowCallbackMilliseconds',
    'settings.modules.field.threshold',
  ],
  ['runtimeDiagnostics', 'continuous', 'settings.modules.field.continuous'],
  ['runtimeDiagnostics', 'onlyChanges', 'settings.modules.field.onlyChanges'],
]
