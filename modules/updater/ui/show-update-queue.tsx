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
  const selected = actionable
    .filter((item) => item.selected)
    .map((item) => item.id)
  const selectable = actionable.filter((item) => !item.required)
  const running = queue.run?.state === 'running'
  const queueError = queue.error || queue.run?.error
  const displayedMessages = new Set(queueError ? [queueError] : [])
  const desktopMode = mode === 'DESKTOP'
  const allSelected =
    selectable.length > 0 && selectable.every((item) => item.selected)
  const stateLabel = (state: string) => {
    const labels: Record<string, TranslationKey> = {
      queued: 'updates.queue.state.queued',
      downloading: 'updates.queue.state.downloading',
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
      {items.length === 0 ? (
        <p className="hint">{t('updates.queue.empty')}</p>
      ) : (
        <div className="update-queue-list">
          {items.map((item) => {
            const hasProgress = item.state === 'downloading' && item.kind === 'system'
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
                      disabled || running || item.required || !actionable.includes(item)
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
                      {item.title}
                      {item.prerelease && (
                        <em className="release-channel-chip">
                          {t('updates.prerelease.badge')}
                        </em>
                      )}
                    </strong>
                    <small>
                      {item.version || item.operation || item.kind}
                      {!hasProgress && <> · {stateLabel(item.state)}</>}
                    </small>
                    {itemMessage && <small>{itemMessage}</small>}
                  </span>
                </label>
                {hasProgress && <UpdateProgress release={release} />}
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
            disabled={disabled || installing}
            onClick={() => onCommand('cancel-update')}
          >
            {t('updater.cancel')}
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
