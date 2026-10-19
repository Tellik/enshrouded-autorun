import { useEffect, useState } from 'react'
import { useI18n } from '../../ui-resources/i18n'
import { Card } from '../../ui-resources/show-card'
import type { UpdateQueue, UpdaterCommand } from './update-types'

export function SystemInstallationActions({
  selectedRoot,
  rootSelected,
  updaterAvailable,
  installationDetected,
  queue,
  onCommand,
}: {
  selectedRoot: string
  rootSelected: boolean
  updaterAvailable: boolean
  installationDetected: boolean
  queue: UpdateQueue
  onCommand: UpdaterCommand
}) {
  const { t } = useI18n()
  const [removeMods, setRemoveMods] = useState(false)
  const [resetUserSettings, setResetUserSettings] = useState(false)
  const [purgeData, setPurgeData] = useState(false)
  useEffect(() => {
    setRemoveMods(false)
    setResetUserSettings(false)
    setPurgeData(false)
  }, [installationDetected, selectedRoot])
  const pending = (queue.items || []).some(
    (item) =>
      item.kind === 'installer' &&
      ['install-latest', 'uninstall'].includes(item.operation || '') &&
      !['complete', 'cancelled', 'error'].includes(item.state),
  )
  const busy = queue.run?.state === 'running'
  const queueInstallation = () => onCommand('queue-latest-installation')
  const queueUninstall = () => {
    const confirmation = t(
      purgeData
        ? removeMods
          ? 'updater.uninstall.confirmPurgeMods'
          : 'updater.uninstall.confirmPurge'
        : removeMods
          ? resetUserSettings
            ? 'updater.uninstall.confirmAll'
            : 'updater.uninstall.confirmMods'
          : resetUserSettings
            ? 'updater.uninstall.confirmSettings'
            : 'updater.uninstall.confirm',
    )
    if (window.confirm(confirmation))
      onCommand('queue-uninstall', {
        removeMods,
        resetUserSettings: resetUserSettings && !purgeData,
        purgeData,
      })
  }

  return (
    <Card title={t('updater.installation.title')}>
      <p className="hint">
        {!rootSelected
          ? t('updater.installation.selectRoot')
          : installationDetected
            ? t('updater.installation.detected')
            : t('updater.installation.notDetected')}
      </p>
      <div className="inline-actions">
        <button
          className="button"
          disabled={!rootSelected || !updaterAvailable || pending || busy}
          onClick={queueInstallation}
        >
          {t(
            installationDetected
              ? 'updater.installation.repair'
              : 'updater.installation.install',
          )}
        </button>
        {rootSelected && installationDetected && (
          <>
            <label className="updater-uninstall-option">
              <input
                type="checkbox"
                checked={removeMods}
                disabled={!updaterAvailable || pending || busy}
                onChange={(event) => setRemoveMods(event.target.checked)}
              />
              <span>{t('updater.uninstall.removeMods')}</span>
            </label>
            <label className="updater-uninstall-option">
              <input
                type="checkbox"
                checked={resetUserSettings}
                disabled={!updaterAvailable || pending || busy || purgeData}
                onChange={(event) => setResetUserSettings(event.target.checked)}
              />
              <span>{t('updater.uninstall.resetUserSettings')}</span>
            </label>
            <label className="updater-uninstall-option">
              <input
                type="checkbox"
                checked={purgeData}
                disabled={!updaterAvailable || pending || busy}
                onChange={(event) => {
                  setPurgeData(event.target.checked)
                  if (event.target.checked) setResetUserSettings(false)
                }}
              />
              <span>{t('updater.uninstall.purgeData')}</span>
            </label>
            <button
              className="button danger"
              disabled={!updaterAvailable || pending || busy}
              onClick={queueUninstall}
            >
              {t('updater.uninstall.button')}
            </button>
          </>
        )}
      </div>
      <p className="hint">
        {installationDetected
          ? t('updater.uninstall.safety')
          : t('updater.installation.safety')}
      </p>
    </Card>
  )
}
