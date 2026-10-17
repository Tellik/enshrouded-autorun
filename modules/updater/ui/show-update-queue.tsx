import { postRequest } from '../../transport/ui/send-host-command'
import { useI18n } from '../../ui-resources/i18n'
import type { TranslationKey } from '../../ui-resources/i18n'
import type { Release, UpdateQueue } from './update-types'
import { Card } from '../../ui-resources/show-card'
import { UpdateProgress } from './show-update-progress'

export function UpdateQueuePanel({
  queue,
  mode,
  release,
  hideWhenEmpty = false,
}: {
  queue: UpdateQueue
  mode: string
  release: Release
  hideWhenEmpty?: boolean
}) {
  const { t } = useI18n()
  const items = (queue.items || []).filter((item) => item.state !== 'complete')
  const actionable = items.filter(
    (item) => !['complete', 'downloading', 'installing'].includes(item.state),
  )
  const selected = actionable
    .filter((item) => item.selected)
    .map((item) => item.id)
  const running = queue.run?.state === 'running'
  if (hideWhenEmpty && !items.length && !running) return null
  const desktopMode = mode === 'DESKTOP' || mode === 'UPDATER'
  const allSelected =
    actionable.length > 0 && actionable.every((item) => item.selected)
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
    postRequest('select-update-queue-items', {
      ids: actionable.map((item) => item.id),
      selected: !allSelected,
    })
  const start = (waitForGame: boolean) =>
    postRequest('start-update-queue', { ids: selected, waitForGame })
  const installing =
    release.state === 'installing' ||
    items.some((item) => item.state === 'installing')
  const clear = () => {
    if (window.confirm(t('updates.queue.clearConfirm')))
      postRequest('clear-update-queue')
  }
  return (
    <Card title={t('updates.queue.title')}>
      <div className="update-queue-summary">
        <span>{t('updates.queue.count', { count: items.length })}</span>
        {actionable.length > 0 && (
          <button className="button ghost" onClick={setAll} disabled={running}>
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
          {items.map((item) => (
            <div className="update-queue-row" key={item.id}>
              <label className="update-queue-item">
                <input
                  type="checkbox"
                  checked={item.selected}
                  disabled={running || !actionable.includes(item)}
                  onChange={(event) =>
                    postRequest('select-update-queue-items', {
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
                    {item.version || item.operation || item.kind} ·{' '}
                    {stateLabel(item.state)}
                  </small>
                  {item.message && <small>{item.message}</small>}
                </span>
              </label>
              {item.state === 'downloading' && item.kind === 'system' && (
                <UpdateProgress release={release} />
              )}
              <button
                className="button ghost"
                disabled={
                  running || ['downloading', 'installing'].includes(item.state)
                }
                onClick={() =>
                  postRequest('remove-update-queue-item', { key: item.id })
                }
              >
                {t('updates.queue.remove')}
              </button>
            </div>
          ))}
        </div>
      )}
      <div className="inline-actions update-queue-actions">
        {desktopMode && (
          <button
            className="button"
            disabled={!selected.length || running}
            onClick={() => start(false)}
          >
            {t('updates.queue.downloadSelected')}
          </button>
        )}
        <button
          className={`button ${desktopMode ? 'ghost' : ''}`}
          disabled={!selected.length || running}
          onClick={() => start(true)}
        >
          {t('updates.queue.afterGame')}
        </button>
        {running && (
          <button
            className="button danger"
            disabled={installing}
            onClick={() => postRequest('cancel-update')}
          >
            {t('updater.cancel')}
          </button>
        )}
        {items.length > 0 && !running && (
          <button
            className="button ghost"
            disabled={installing}
            onClick={clear}
          >
            {t('updates.queue.clear')}
          </button>
        )}
      </div>
    </Card>
  )
}
