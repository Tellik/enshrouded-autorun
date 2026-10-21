import { useEffect, useState } from 'react'
import { useI18n } from '../../ui-resources/i18n'
import type { TranslationKey } from '../../ui-resources/i18n'
import { Card } from '../../ui-resources/show-card'
import type {
  BackupManagement,
  UpdateQueue,
  UpdaterCommand,
} from './update-types'

export function SystemInstallationActions({
  selectedRoot,
  rootSelected,
  updaterAvailable,
  installationDetected,
  latestVersion,
  backupManagement,
  queue,
  onCommand,
}: {
  selectedRoot: string
  rootSelected: boolean
  updaterAvailable: boolean
  installationDetected: boolean
  latestVersion?: string
  backupManagement?: BackupManagement
  queue: UpdateQueue
  onCommand: UpdaterCommand
}) {
  const { t } = useI18n()
  const [optionsOpen, setOptionsOpen] = useState(false)
  const [restoreGamefiles, setRestoreGamefiles] = useState(
    backupManagement?.originals?.status === 'ready',
  )
  const [removeMods, setRemoveMods] = useState(false)
  const [resetUserSettings, setResetUserSettings] = useState(false)
  const [purgeData, setPurgeData] = useState(false)
  const originalStatus = backupManagement?.originals?.status || 'missing'
  const restoreAvailable = originalStatus === 'ready'

  useEffect(() => {
    setOptionsOpen(false)
    setRestoreGamefiles(backupManagement?.originals?.status === 'ready')
    setRemoveMods(false)
    setResetUserSettings(false)
    setPurgeData(false)
  }, [backupManagement?.originals?.status, installationDetected, selectedRoot])

  const activeItem = (state: string) =>
    !['complete', 'cancelled', 'error'].includes(state)
  const installPending = (queue.items || []).some(
    (item) =>
      activeItem(item.state) &&
      (item.kind === 'installer' ||
        (item.kind === 'system' &&
          (!latestVersion || item.version === latestVersion))),
  )
  const uninstallPending = (queue.items || []).some(
    (item) =>
      item.kind === 'installer' &&
      item.operation === 'uninstall' &&
      activeItem(item.state),
  )
  const busy = queue.run?.state === 'running'
  const statusKey =
    originalStatus === 'ready'
      ? 'ready'
      : originalStatus === 'verification-required'
        ? 'verificationRequired'
        : originalStatus === 'invalid'
          ? 'invalid'
          : 'missing'
  const queueInstallation = () => onCommand('queue-latest-installation')
  const queueUninstall = () => {
    const confirmationKey = restoreGamefiles
      ? purgeData
        ? removeMods
          ? 'updater.uninstall.confirmPurgeMods'
          : 'updater.uninstall.confirmPurge'
        : 'updater.uninstall.confirm'
      : purgeData
        ? removeMods
          ? 'updater.uninstall.confirmPurgeWithoutRestoreMods'
          : 'updater.uninstall.confirmPurgeWithoutRestore'
        : 'updater.uninstall.confirmWithoutRestore'
    if (
      window.confirm(
        t(confirmationKey as TranslationKey),
      )
    ) {
      onCommand('queue-uninstall', {
        restoreGamefiles,
        removeMods,
        resetUserSettings: resetUserSettings && !purgeData,
        purgeData,
      })
      setOptionsOpen(false)
    }
  }

  return (
    <div className="updater-installation-actions">
      <div className="inline-actions updater-installation-action-row">
        <button
          className="button"
          disabled={!rootSelected || !updaterAvailable || installPending || busy}
          onClick={queueInstallation}
        >
          {t(
            installationDetected
              ? 'updater.installation.repair'
              : 'updater.installation.install',
          )}
        </button>
        <button
          className="button danger"
          disabled={
            !rootSelected ||
            !installationDetected ||
            !updaterAvailable ||
            uninstallPending ||
            busy
          }
          aria-expanded={optionsOpen}
          onClick={() => setOptionsOpen((open) => !open)}
        >
          {t('updater.uninstall.button')}
        </button>
      </div>

      {optionsOpen && (
        <Card title={t('updater.uninstall.optionsTitle')}>
          <div className="settings-module-list">
            <section className="settings-module">
              <label className="updater-uninstall-option">
                <input
                  type="checkbox"
                  checked={restoreGamefiles}
                  disabled={!restoreAvailable || !updaterAvailable || busy}
                  onChange={(event) =>
                    setRestoreGamefiles(event.target.checked)
                  }
                />
                <span>{t('updater.uninstall.restoreGamefiles')}</span>
              </label>
              <p className="hint">
                {t(
                  `settings.backups.status.${statusKey}` as TranslationKey,
                )}
              </p>
              <label className="updater-uninstall-option">
                <input
                  type="checkbox"
                  checked={removeMods}
                  disabled={!updaterAvailable || busy}
                  onChange={(event) => setRemoveMods(event.target.checked)}
                />
                <span>{t('updater.uninstall.removeMods')}</span>
              </label>
              <label className="updater-uninstall-option">
                <input
                  type="checkbox"
                  checked={resetUserSettings}
                  disabled={!updaterAvailable || busy || purgeData}
                  onChange={(event) =>
                    setResetUserSettings(event.target.checked)
                  }
                />
                <span>{t('updater.uninstall.resetUserSettings')}</span>
              </label>
              <label className="updater-uninstall-option">
                <input
                  type="checkbox"
                  checked={purgeData}
                  disabled={!updaterAvailable || busy}
                  onChange={(event) => {
                    setPurgeData(event.target.checked)
                    if (event.target.checked) setResetUserSettings(false)
                  }}
                />
                <span>{t('updater.uninstall.purgeData')}</span>
              </label>
              <p className="hint">{t('updater.uninstall.safety')}</p>
            </section>
          </div>
          <div className="inline-actions">
            <button
              className="button danger"
              disabled={!updaterAvailable || uninstallPending || busy}
              onClick={queueUninstall}
            >
              {t('updater.uninstall.queueButton')}
            </button>
            <button
              className="button ghost"
              disabled={busy}
              onClick={() => setOptionsOpen(false)}
            >
              {t('common.cancel')}
            </button>
          </div>
        </Card>
      )}
    </div>
  )
}
