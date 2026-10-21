import { useI18n } from '../../ui-resources/i18n'
import type { TranslationKey } from '../../ui-resources/i18n'
import type { Release, UpdateQueue, UpdaterCommand } from './update-types'
import { Card } from '../../ui-resources/show-card'
import { UpdateProgress } from './show-update-progress'

export function UpdateQueuePanel({
  queue,
  mode,
  release,
  disabled = false,
  onCommand,
}: {
  queue: UpdateQueue
  mode: string
  release: Release
  disabled?: boolean
  onCommand: UpdaterCommand
}) {
  const { t } = useI18n()
  const items = (queue.items || []).filter((item) => item.state !== 'complete')
  const actionable = items.filter(
    (item) => !['complete', 'downloading', 'installing'].includes(item.state),
  )
  const isUninstall = (item: (typeof queue.items)[number]) =>
    item.kind === 'installer' && item.operation === 'uninstall'
  const uninstallPending = actionable.some(
    (item) =>
      isUninstall(item) && !['error', 'cancelled'].includes(item.state),
  )
  const selected = actionable
    .filter((item) => item.selected)
    .map((item) => item.id)
  const selectable = actionable.filter(
    (item) => !item.required && (!uninstallPending || isUninstall(item)),
  )
  const running = queue.run?.state === 'running'
  const runItemIds = new Set(queue.run?.itemIds || [])
  const isSystemInstall = (item: (typeof queue.items)[number] | undefined) =>
    item?.kind === 'system' ||
    (item?.kind === 'installer' && item.operation === 'install-latest')
  const activeSystemInstall = isSystemInstall(
    queue.items.find((item) => item.id === queue.run?.activeItemId) ||
      queue.items.find((item) => runItemIds.has(item.id) && isSystemInstall(item)),
  )
  const showProgress =
    (running && activeSystemInstall) ||
    (queue.run?.state === 'complete' &&
      activeSystemInstall &&
      release.state === 'installed')
  const queueError = queue.error || queue.run?.error
  const displayedMessages = new Set(queueError ? [queueError] : [])
  const desktopMode = mode === 'DESKTOP'
  const allSelected =
    selectable.length > 0 && selectable.every((item) => item.selected)
  const stateLabel = (state: string) => {
    const labels: Record<string, TranslationKey> = {
      queued: 'updates.queue.state.queued',
      downloading: 'updates.queue.state.downloading',
      restoring: 'updates.queue.state.restoring',
      uninstalling: 'updates.queue.state.uninstalling',
      waitingForGame: 'updater.waiting',
      verifying: 'updater.verifying',
      extracting: 'updater.extracting',
      staged: 'updater.staged',
      installing: 'updater.installing',
      complete: 'updater.complete',
      error: 'updater.failed',
      cancelled: 'updates.queue.state.cancelled',
    }
    return t(labels[state] || 'updates.queue.state.queued')
  }
  const setAll = () =>
    onCommand('select-update-queue-items', {
      ids: selectable.map((item) => item.id),
      selected: !allSelected,
    })
  const start = (waitForGame: boolean) =>
    onCommand('start-update-queue', { ids: selected, waitForGame })
  const installing =
    release.state === 'installing' ||
    items.some((item) => item.state === 'installing')
  const cancellationPending =
    release.state === 'cancelled' || queue.run?.cancelRequested === true
  const cancellationBlocked =
    queue.run?.installing === true && queue.run.cancelable !== true
  const clear = () => {
    if (window.confirm(t('updates.queue.clearConfirm')))
      onCommand('clear-update-queue')
  }
  return (
    <Card title={t('updates.queue.title')}>
      {queueError && <p className="notice">{queueError}</p>}
      <div className="update-queue-summary">
        <span>{t('updates.queue.count', { count: items.length })}</span>
        {selectable.length > 0 && (
          <button
            className="button ghost"
            onClick={setAll}
            disabled={disabled || running}
          >
            {allSelected
              ? t('updates.queue.selectNone')
              : t('updates.queue.selectAll')}
          </button>
        )}
      </div>
      {showProgress && <UpdateProgress release={release} />}
      {items.length === 0 ? (
        <p className="hint">{t('updates.queue.empty')}</p>
      ) : (
        <div className="update-queue-list">
          {items.map((item) => {
            const uninstallTitleKey = item.payload?.purgeData
              ? item.payload?.removeMods
                ? 'updater.uninstall.queuePurgeModsTitle'
                : 'updater.uninstall.queuePurgeTitle'
              : item.payload?.removeMods
                ? item.payload?.resetUserSettings
                  ? 'updater.uninstall.queueAllTitle'
                  : 'updater.uninstall.queueModsTitle'
                : item.payload?.resetUserSettings
                  ? 'updater.uninstall.queueSettingsTitle'
                  : 'updater.uninstall.queueTitle'
            const title =
              item.kind === 'installer' && item.operation === 'uninstall'
                ? t(uninstallTitleKey)
                : item.kind === 'installer'
                  ? t('updater.installation.queueTitle')
                  : item.title
            const itemMessage =
              item.message && !displayedMessages.has(item.message)
                ? item.message
                : undefined
            if (itemMessage) displayedMessages.add(itemMessage)
            return (
              <div className="update-queue-row" key={item.id}>
                <label className="update-queue-item">
                  <input
                    type="checkbox"
                    checked={item.selected}
                    disabled={
                      disabled ||
                      running ||
                      item.required ||
                      !actionable.includes(item) ||
                      (uninstallPending && !isUninstall(item))
                    }
                    onChange={(event) =>
                      onCommand('select-update-queue-items', {
                        ids: [item.id],
                        selected: event.target.checked,
                      })
                    }
                  />
                  <span>
                    <strong>
                      {title}
                      {item.prerelease && (
                        <em className="release-channel-chip">
                          {t('updates.prerelease.badge')}
                        </em>
                      )}
                    </strong>
                    <small>
                      {item.version || item.operation || item.kind}
                      <> · {stateLabel(item.state)}</>
                    </small>
                    {isUninstall(item) && item.payload?.restoreGamefiles && (
                      <small>{t('updater.uninstall.queueRestoreLabel')}</small>
                    )}
                    {itemMessage && <small>{itemMessage}</small>}
                  </span>
                </label>
                <button
                  className="button ghost"
                  disabled={
                    disabled ||
                    running ||
                    item.required ||
                    ['downloading', 'installing'].includes(item.state)
                  }
                  onClick={() =>
                    onCommand('remove-update-queue-item', { key: item.id })
                  }
                >
                  {t('updates.queue.remove')}
                </button>
              </div>
            )
          })}
        </div>
      )}
      <div className="inline-actions update-queue-actions">
        {desktopMode && (
          <button
            className="button"
            disabled={disabled || !selected.length || running}
            onClick={() => start(false)}
          >
            {t('updates.queue.downloadSelected')}
          </button>
        )}
        <button
          className={`button ${desktopMode ? 'ghost' : ''}`}
          disabled={disabled || !selected.length || running}
          onClick={() => start(true)}
        >
          {t('updates.queue.afterGame')}
        </button>
        {running && (
          <button
            className="button danger"
            disabled={disabled || cancellationPending || cancellationBlocked}
            onClick={() => onCommand('cancel-update')}
          >
            {cancellationBlocked || (installing && queue.run?.cancelable !== true)
              ? t('updater.installing')
              : t('updater.cancel')}
          </button>
        )}
        {items.length > 0 && !running && (
          <button
            className="button ghost"
            disabled={disabled || installing}
            onClick={clear}
          >
            {t('updates.queue.clear')}
          </button>
        )}
      </div>
    </Card>
  )
}
