import { postRequest } from '../../modules/transport/ui/send-host-command'
import { TranslationKey, useI18n } from '../../modules/ui-resources/i18n'
import type { Snapshot } from '../../modules/modloader/ui/ui-snapshot'
import { Card } from '../../modules/ui-resources/show-card'

export function GamefileBackupManagement({ data }: { data: Snapshot }) {
  const { t } = useI18n()
  const backup = data.backupManagement
  const originals = backup?.originals
  const state = originals?.status || 'missing'
  const restore = backup?.restore
  const busy = ['queued', 'waitingForGame', 'restoring'].includes(
    restore?.status || '',
  )
  const requestRestore = () => {
    if (state !== 'ready' || busy) return
    if (
      window.confirm(
        t('settings.backups.restoreConfirm', {
          target:
            backup?.target === 'server' ? 'Dedicated Server' : 'Enshrouded',
        }),
      )
    )
      postRequest('restore-gamefiles')
  }
  const capture = () => {
    if (!['missing', 'verification-required'].includes(state)) return
    if (window.confirm(t('settings.backups.captureConfirm')))
      postRequest('capture-gamefile-originals')
  }
  const statusKey =
    state === 'ready'
      ? 'ready'
      : state === 'verification-required'
        ? 'verificationRequired'
        : state === 'invalid'
          ? 'invalid'
          : 'missing'
  return (
    <Card title={t('settings.backups.title')}>
      <p className="hint">
        {t('settings.backups.description', {
          target:
            backup?.target === 'server' ? 'Dedicated Server' : 'Enshrouded',
        })}
      </p>
      <div className="settings-module-list">
        <section className="settings-module">
          <header>
            <div>
              <h3>{t('settings.backups.originals')}</h3>
              <p className="hint">
                {t(`settings.backups.status.${statusKey}` as TranslationKey)}
              </p>
              {originals?.capturedAt && (
                <p className="hint">
                  {t('settings.backups.capturedAt', {
                    date: new Date(
                      originals.capturedAt * 1000,
                    ).toLocaleString(),
                  })}{' '}
                  · {originals.source}
                </p>
              )}
            </div>
            <span
              className={`state-pill ${state === 'ready' ? 'ok' : state === 'invalid' ? 'error' : 'neutral'}`}
            >
              {state === 'ready'
                ? t('settings.backups.verified')
                : t(`settings.backups.status.${statusKey}` as TranslationKey)}
            </span>
          </header>
          <p className="hint">
            {(originals?.files || [])
              .map((file) => `${file.valid ? '✓' : '!'} ${file.name}`)
              .join(' · ')}
          </p>
          <div className="inline-actions">
            {state === 'ready' ? (
              <button
                className="button danger"
                disabled={busy}
                onClick={requestRestore}
              >
                {t('settings.backups.restore')}
              </button>
            ) : ['missing', 'verification-required'].includes(state) ? (
              <button className="button secondary" onClick={capture}>
                {t('settings.backups.capture')}
              </button>
            ) : null}
          </div>
        </section>
        <section className="settings-module">
          <header>
            <div>
              <h3>{t('settings.backups.working')}</h3>
              <p className="hint">
                {t(
                  backup?.workingBaseline?.complete
                    ? 'settings.backups.workingReady'
                    : 'settings.backups.workingPartial',
                )}
              </p>
            </div>
          </header>
          <p className="hint">
            {t('settings.backups.workingFiles', {
              kfc: backup?.workingBaseline?.kfc ? '✓' : '—',
              resources: backup?.workingBaseline?.resources ? '✓' : '—',
            })}
          </p>
        </section>
        <section className="settings-module">
          <header>
            <div>
              <h3>{t('settings.backups.other')}</h3>
              <p className="hint">
                {t('settings.backups.otherDetail', {
                  updates: backup?.rollback?.systemUpdates || 0,
                  mods: backup?.rollback?.modPackages || 0,
                  transaction: backup?.transaction?.pending
                    ? t('settings.backups.transactionPending')
                    : t('settings.backups.transactionIdle'),
                })}
              </p>
            </div>
          </header>
        </section>
        {restore?.status && restore.status !== 'idle' && (
          <p className="hint">
            {t(
              `settings.backups.restoreStatus.${restore.status}` as TranslationKey,
            )}
            {restore.message ? ` · ${restore.message}` : ''}
          </p>
        )}
      </div>
    </Card>
  )
}
