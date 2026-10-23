import { useEffect, useState } from 'react'
import { useI18n } from '../../ui-resources/i18n'
import type { TranslationKey } from '../../ui-resources/i18n'
import { Card } from '../../ui-resources/show-card'
import type {
  BackupManagement,
  UpdateQueue,
  UpdaterCommand,
} from './update-types'

type UninstallDataAction = 'keep' | 'reset' | 'purge'

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
  const [dataAction, setDataAction] = useState<UninstallDataAction>('keep')
  const originalStatus = backupManagement?.originals?.status || 'missing'
  const restoreAvailable = originalStatus === 'ready'
  const resetUserSettings = dataAction === 'reset'
  const purgeData = dataAction === 'purge'

  useEffect(() => {
    setOptionsOpen(false)
    setRestoreGamefiles(backupManagement?.originals?.status === 'ready')
    setRemoveMods(false)
    setDataAction('keep')
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
  const actionsDisabled = !updaterAvailable || busy
  const restoreDisabled = !restoreAvailable || actionsDisabled
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
    const confirmation = [
      t(confirmationKey as TranslationKey),
      ...(!purgeData && removeMods
        ? [t('updater.uninstall.removeMods')]
        : []),
      ...(!purgeData
        ? [
            t(
              dataAction === 'keep'
                ? 'updater.uninstall.keepData'
                : 'updater.uninstall.resetUserSettings',
            ),
          ]
        : []),
    ].join('\n\n')
    if (window.confirm(confirmation)) {
      onCommand('queue-uninstall', {
        restoreGamefiles,
        removeMods,
        resetUserSettings,
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
              <h3>{t('updater.uninstall.groupGame')}</h3>
              <label
                className={`updater-uninstall-option${restoreDisabled ? ' is-disabled' : ''}`}
              >
                <input
                  type="checkbox"
                  checked={restoreGamefiles}
                  disabled={restoreDisabled}
                  onChange={(event) =>
                    setRestoreGamefiles(event.target.checked)
                  }
                />
                <span>{t('updater.uninstall.restoreGamefiles')}</span>
              </label>
              <p className="hint settings-module-description">
                {t(
                  `settings.backups.status.${statusKey}` as TranslationKey,
                )}
              </p>
            </section>

            <section className="settings-module">
              <h3>{t('updater.uninstall.groupMods')}</h3>
              <label className="updater-uninstall-option">
                <input
                  type="checkbox"
                  checked={removeMods}
                  disabled={actionsDisabled}
                  onChange={(event) => setRemoveMods(event.target.checked)}
                />
                <span>{t('updater.uninstall.removeMods')}</span>
              </label>
              <p className="hint settings-module-description">
                {t('updater.uninstall.removeModsHint')}
              </p>
            </section>

            <section className="settings-module">
              <fieldset
                className="updater-uninstall-data-options"
                disabled={actionsDisabled}
              >
                <legend>{t('updater.uninstall.groupData')}</legend>
                <p className="hint settings-module-description">
                  {t('updater.uninstall.dataChoiceHint')}
                </p>
                <div className="updater-uninstall-data-choices">
                  <label className="updater-uninstall-option">
                    <input
                      type="radio"
                      name="shroudforge-uninstall-data-action"
                      value="keep"
                      checked={dataAction === 'keep'}
                      onChange={() => setDataAction('keep')}
                    />
                    <span>{t('updater.uninstall.keepData')}</span>
                  </label>
                  <label className="updater-uninstall-option">
                    <input
                      type="radio"
                      name="shroudforge-uninstall-data-action"
                      value="reset"
                      checked={dataAction === 'reset'}
                      onChange={() => setDataAction('reset')}
                    />
                    <span>{t('updater.uninstall.resetUserSettings')}</span>
                  </label>
                  <label className="updater-uninstall-option">
                    <input
                      type="radio"
                      name="shroudforge-uninstall-data-action"
                      value="purge"
                      checked={dataAction === 'purge'}
                      onChange={() => setDataAction('purge')}
                    />
                    <span>{t('updater.uninstall.purgeData')}</span>
                  </label>
                </div>
              </fieldset>
            </section>
          </div>
          <p className="hint">{t('updater.uninstall.safety')}</p>
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
