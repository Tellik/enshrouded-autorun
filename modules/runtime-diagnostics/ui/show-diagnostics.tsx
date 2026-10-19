import { useState } from 'react'
import { postRequest } from '../../transport/ui/send-host-command'
import { writeClipboardText } from '../../ui-resources/copy-to-clipboard'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { DiagnosticStatus } from './diagnostic-status'
import type { DiagnosticJob } from '../../loader-config/ui/loader-settings'

const builtInJobLabels: Record<string, [TranslationKey, TranslationKey]> = {
  'runtime.overview': [
    'settings.modules.diagnosticJob.overview',
    'settings.modules.diagnosticJob.overviewDescription',
  ],
  'runtime.health': [
    'settings.modules.diagnosticJob.health',
    'settings.modules.diagnosticJob.healthDescription',
  ],
  'runtime.queue': [
    'settings.modules.diagnosticJob.queue',
    'settings.modules.diagnosticJob.queueDescription',
  ],
  'runtime.callbacks': [
    'settings.modules.diagnosticJob.callbacks',
    'settings.modules.diagnosticJob.callbacksDescription',
  ],
  'runtime.operations': [
    'settings.modules.diagnosticJob.operations',
    'settings.modules.diagnosticJob.operationsDescription',
  ],
  'world-editor.api-read-ab': [
    'settings.modules.diagnosticJob.worldEditorReadAb',
    'settings.modules.diagnosticJob.worldEditorReadAbDescription',
  ],
  'world-editor.typed-inputs': [
    'settings.modules.diagnosticJob.worldEditorTypedInputs',
    'settings.modules.diagnosticJob.worldEditorTypedInputsDescription',
  ],
  'world-editor.registered-impact-ab': [
    'settings.modules.diagnosticJob.worldEditorImpactAb',
    'settings.modules.diagnosticJob.worldEditorImpactAbDescription',
  ],
  'world-editor.voxel-write-ab': [
    'settings.modules.diagnosticJob.worldEditorVoxelWriteAb',
    'settings.modules.diagnosticJob.worldEditorVoxelWriteAbDescription',
  ],
}

export function DiagnosticsControls({
  data,
  prefs,
  module,
  change,
  jobs,
  jobErrors,
  jobDirectory,
  selectedJobId,
}: {
  data: { diagnostics?: DiagnosticStatus }
  prefs: Record<string, Record<string, any>>
  module: string
  change(module: string, key: string, value: unknown): void
  jobs: DiagnosticJob[]
  jobErrors: string[]
  jobDirectory: string
  selectedJobId: string
}) {
  const { t } = useI18n()
  const [copyState, setCopyState] = useState<
    'idle' | 'copying' | 'copied' | 'error'
  >('idle')
  const preferredJobId = prefs.runtimeDiagnostics?.jobId
  const moduleEnabled = prefs.runtimeDiagnostics?.enabled === true
  const jobId = jobs.some((job) => job.id === preferredJobId)
    ? preferredJobId
    : selectedJobId
  const selectedJob = jobs.find((job) => job.id === jobId)
  const jobLabel = (job: DiagnosticJob, description = false) => {
    const keys = builtInJobLabels[job.id]
    return keys ? t(keys[description ? 1 : 0]) : description ? job.description : job.name
  }
  const statusText = !moduleEnabled
    ? t('settings.moduleUnavailable')
    : data.diagnostics?.fresh
    ? data.diagnostics.active
      ? `${t('settings.modules.diagnosticActive')} · ${data.diagnostics.remainingSeconds}s`
      : data.diagnostics.reason
    : t('settings.modules.diagnosticUnavailable')
  const reportText = data.diagnostics?.report
    ? JSON.stringify(data.diagnostics.report, null, 2)
    : undefined
  const copyLabel =
    copyState === 'copying'
      ? t('activity.copying')
      : copyState === 'copied'
        ? t('activity.copied')
        : copyState === 'error'
          ? t('activity.copyFailed')
          : t('activity.copyLogs')
  return (
    <div className="settings-diagnostics">
      <label className="settings-diagnostic-job">
        <span>{t('settings.modules.diagnosticJobSelector')}</span>
        <select
          value={jobId}
          onChange={(event) => change(module, 'jobId', event.target.value)}
        >
          {!jobs.some((job) => job.id === jobId) && (
            <option value={jobId} disabled>
              {t('settings.modules.diagnosticJobMissing', { jobId })}
            </option>
          )}
          {jobs.map((job) => (
            <option key={job.id} value={job.id}>
              {jobLabel(job)}
            </option>
          ))}
        </select>
      </label>
      {selectedJob && <p className="hint">{jobLabel(selectedJob, true)}</p>}
      <p
        className={moduleEnabled ? 'hint' : 'builder-notice warning'}
        role={!moduleEnabled ? 'status' : undefined}
      >
        {statusText}
        {data.diagnostics?.active && data.diagnostics.jobId && (
          <>
            {' · '}
            {t('settings.modules.diagnosticJobActive')}: {data.diagnostics.jobId}
          </>
        )}
        {' · '}
        {t('settings.modules.lastSample')}
        {': '}
        {data.diagnostics?.lastSampleAt
          ? new Date(data.diagnostics.lastSampleAt * 1000).toLocaleString()
          : '—'}
      </p>
      <p className="hint">{t('settings.modules.saveHint')}</p>
      <p className="hint">
        {t('settings.modules.diagnosticJobFolder')}: <code>{jobDirectory}</code>
      </p>
      <div className="settings-diagnostic-actions">
        <button
          className="button ghost"
          onClick={() => postRequest('open-path', { key: 'diagnostic-jobs' })}
        >
          {t('settings.modules.openDiagnosticJobFolder')}
        </button>
        <button
          className="button ghost"
          onClick={() => postRequest('open-path', { key: 'diagnostic-exports' })}
        >
          {t('settings.modules.openDiagnosticExportFolder')}
        </button>
        <button
          className="button ghost"
          disabled={!reportText || copyState === 'copying'}
          onClick={() => {
            if (!reportText) return
            setCopyState('copying')
            void writeClipboardText(reportText)
              .then(() => {
                setCopyState('copied')
                window.setTimeout(
                  () =>
                    setCopyState((current) =>
                      current === 'copied' ? 'idle' : current,
                    ),
                  1800,
                )
              })
              .catch(() => setCopyState('error'))
          }}
        >
          {copyLabel}
        </button>
      </div>
      {jobErrors.length > 0 && (
        <details className="settings-diagnostic-job-errors">
          <summary>{t('settings.modules.diagnosticJobErrors', { count: jobErrors.length })}</summary>
          <ul>
            {jobErrors.map((error) => <li key={error}>{error}</li>)}
          </ul>
        </details>
      )}
      <div className="settings-diagnostic-actions">
        {(['start', 'stop', 'snapshot'] as const).map((action) => (
          <button
            className="button ghost"
            key={action}
            disabled={!moduleEnabled && action !== 'stop'}
            onClick={() => postRequest('diagnostics', { action, jobId })}
          >
            {t(`settings.modules.${action}`)}
          </button>
        ))}
      </div>
      {reportText && <pre className="diagnostic-report">{reportText}</pre>}
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
  ['runtimeDiagnostics', 'onlyChanges', 'settings.modules.field.onlyChanges'],
]
