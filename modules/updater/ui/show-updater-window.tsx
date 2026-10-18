import { postRequest } from '../../transport/ui/send-host-command'
import { useI18n } from '../../ui-resources/i18n'
import type { Release, UpdateQueue } from './update-types'
import { ReleaseNotes } from './show-release-notes'
import { UpdateProgress } from './show-update-progress'
import { UpdateQueuePanel } from './show-update-queue'

export function UpdaterWindow({
  release,
  version,
  queue,
  mode,
}: {
  release: Release
  version: string
  queue: UpdateQueue
  mode: string
}) {
  const { t } = useI18n()
  const active =
    queue.run?.state === 'running' ||
    [
      'queued',
      'downloading',
      'verifying',
      'extracting',
      'staged',
      'waitingForGame',
      'installing',
    ].includes(release.state)
  const complete = release.state === 'installed'
  const failed = release.state === 'error'
  const queuedSystem = queue.items.some(
    (item) =>
      item.kind === 'system' &&
      item.version === release.latestVersion &&
      !['complete', 'cancelled'].includes(item.state),
  )
  return (
    <main className="updater-shell">
      <header className="updater-topbar" data-drag-region>
        <div className="updater-brand">
          <span className="brand-mark">SF</span>
          <span>
            <strong>SHROUDFORGE</strong>
            <small>UPDATER</small>
          </span>
        </div>
        <div className="window-controls">
          <button
            className="window-control"
            onClick={() => postRequest('minimize-window')}
            aria-label={t('common.minimize')}
            title={t('common.minimize')}
          >
            −
          </button>
          <button
            className="window-control"
            onClick={() => postRequest('toggle-maximize-window')}
            aria-label={t('common.maximize')}
            title={t('common.maximize')}
          >
            □
          </button>
          <button
            className="window-control window-close"
            onClick={() => postRequest('hide')}
            aria-label={t('common.close')}
            title={t('common.close')}
          >
            ×
          </button>
        </div>
      </header>
      <section className="updater-content">
        <div className="updater-heading">
          <div>
            <small>{t('updater.system')}</small>
            <h1>
              {release.latestVersion
                ? `ShroudForge ${release.latestVersion}`
                : t('updater.title')}
            </h1>
            {release.prerelease && (
              <span className="release-channel-chip">
                {t('updates.prerelease.badge')}
              </span>
            )}
          </div>
        </div>
        {release.updateAvailable && !failed && !complete ? (
          <>
            <ReleaseNotes text={release.message || t('updater.starting')} />
            {!queuedSystem && (
              <button
                className="button"
                onClick={() => postRequest('queue-system-update')}
              >
                {t(
                  release.mandatory
                    ? 'updater.installRequired'
                    : 'updater.updateNow',
                )}
              </button>
            )}
            {!release.mandatory && (
              <button
                className="button ghost"
                onClick={() => postRequest('hide')}
              >
                {t('updater.later')}
              </button>
            )}
          </>
        ) : (
          !complete && (
            <p className="updater-message">
              {failed
                ? release.message || t('updater.failedDetail')
                : release.message || t('updater.starting')}
            </p>
          )
        )}
        {active && <UpdateProgress release={release} />}
        {!active &&
          !complete &&
          !failed &&
          release.state === 'ready' &&
          !release.updateAvailable && (
            <div className="updater-result success">
              <i>✓</i>
              <span>{t('updater.current')}</span>
            </div>
          )}
        {complete && (
          <div className="updater-result success">
            <i>✓</i>
            <span>{t('updater.completeDetail')}</span>
          </div>
        )}
        <UpdateQueuePanel
          queue={queue}
          mode={mode}
          release={release}
          hideWhenEmpty
        />
        <footer className="updater-footer">
          <span>{t('updater.installedVersion', { version })}</span>
          {failed && (
            <button
              className="button ghost"
              onClick={() => postRequest('check-updates')}
            >
              {t('updater.retry')}
            </button>
          )}
          {release.state === 'ready' && !release.updateAvailable && (
            <button
              className="button ghost"
              onClick={() => postRequest('check-updates')}
            >
              {t('updater.checkAgain')}
            </button>
          )}
        </footer>
      </section>
    </main>
  )
}
