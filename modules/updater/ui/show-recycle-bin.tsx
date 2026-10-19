import { useEffect, useState } from 'react'
import { postRequest } from '../../transport/ui/send-host-command'
import { Empty } from '../../ui-resources/show-empty-state'
import { Icon } from '../../ui-resources/show-icon'
import { PageHeader } from '../../ui-resources/show-page-header'
import { useI18n } from '../../ui-resources/i18n'
import { UpdaterAvailability } from './show-updater-availability'
import type { UpdaterCommand } from './update-types'

type Entry = {
  id: string
  reason: string
  createdAt: number
  fileCount: number
  sizeBytes: number
  canRestore: boolean
  restoreModId?: string
}

type RecycleBinResult = {
  action: string
  success: boolean
  value?: { entries?: Entry[]; removed?: number }
  error?: string
}

export function RecycleBin({
  retentionDays,
  automaticCleanup,
  updaterAvailable,
  enabled,
}: {
  retentionDays: number
  automaticCleanup: boolean
  updaterAvailable: boolean
  enabled: boolean
}) {
  const { t } = useI18n()
  const [entries, setEntries] = useState<Entry[]>([])
  const [loading, setLoading] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  const [message, setMessage] = useState('')
  const actionsEnabled = updaterAvailable && enabled
  const onCommand: UpdaterCommand = (command, payload) =>
    postRequest(command, payload)

  const refresh = () => {
    if (!actionsEnabled) return
    setLoading(true)
    postRequest('updater-request', { action: 'recycle-bin-list' })
  }

  useEffect(() => {
    const receive = (event: Event) => {
      const result = (event as CustomEvent<RecycleBinResult>).detail
      if (result.action === 'recycle-bin-list') {
        setLoading(false)
        if (result.success) setEntries(result.value?.entries || [])
        else setMessage(result.error || t('recycleBin.error'))
        return
      }
      setBusy(null)
      if (!result.success) {
        setMessage(result.error || t('recycleBin.error'))
        return
      }
      setMessage(
        result.action === 'recycle-bin-empty'
          ? t('recycleBin.emptied')
          : result.action === 'recycle-bin-delete'
            ? t('recycleBin.deleted')
            : t('recycleBin.restored'),
      )
      refresh()
    }
    window.addEventListener('shroudforge-recycle-bin-result', receive)
    if (actionsEnabled) refresh()
    return () =>
      window.removeEventListener('shroudforge-recycle-bin-result', receive)
  }, [t, actionsEnabled])

  const restore = (entry: Entry) => {
    if (!actionsEnabled) return
    setBusy(entry.id)
    setMessage('')
    postRequest('updater-request', {
      action: 'recycle-bin-restore',
      id: entry.id,
    })
  }

  const empty = () => {
    if (!actionsEnabled) return
    if (!window.confirm(t('recycleBin.emptyConfirm'))) return
    setBusy('empty')
    setMessage('')
    postRequest('updater-request', { action: 'recycle-bin-empty' })
  }

  const remove = (entry: Entry) => {
    if (!actionsEnabled) return
    const name = entry.restoreModId || entry.reason
    if (!window.confirm(t('recycleBin.deleteConfirm', { name }))) return
    setBusy(`delete-${entry.id}`)
    setMessage('')
    postRequest('updater-request', {
      action: 'recycle-bin-delete',
      id: entry.id,
    })
  }

  return (
    <>
      <PageHeader
        eyebrow={t('nav.mods')}
        title={t('recycleBin.title')}
        subtitle={
          automaticCleanup
            ? t('recycleBin.subtitle', { days: retentionDays })
            : t('recycleBin.subtitle.manual')
        }
        actions={
          <>
            <button className="button secondary" onClick={refresh} disabled={!actionsEnabled || loading || busy !== null}>
              <Icon name="refresh" /> {t('common.refresh')}
            </button>
            <button
              className="button danger"
              onClick={empty}
              disabled={!actionsEnabled || entries.length === 0 || busy !== null}
            >
              <Icon name="trash" /> {busy === 'empty' ? t('recycleBin.emptying') : t('recycleBin.empty')}
            </button>
          </>
        }
      />
      <UpdaterAvailability
        updaterAvailable={updaterAvailable}
        enabled={enabled}
        onCommand={onCommand}
      />
      {!actionsEnabled ? null : (
        <>
          {message && <p className="notice">{message}</p>}
          {loading ? (
            <div className="card">{t('recycleBin.loading')}</div>
          ) : entries.length === 0 ? (
            <Empty
              title={t('recycleBin.empty.title')}
              text={t('recycleBin.empty.text')}
              icon="trash"
            />
          ) : (
            <div className="card">
          {entries.map((entry) => (
            <article className="queue-item" key={entry.id}>
              <div>
                <strong>{entry.restoreModId || entry.reason}</strong>
                <small>
                  {entry.restoreModId ? <>{entry.reason} · </> : null}
                  {new Date(entry.createdAt * 1000).toLocaleString()} ·{' '}
                  {t('recycleBin.files', { count: entry.fileCount })} ·{' '}
                  {formatSize(entry.sizeBytes)}
                </small>
              </div>
              <div className="actions">
                {entry.canRestore ? (
                  <button
                    className="button secondary"
                    onClick={() => restore(entry)}
                    disabled={busy !== null}
                  >
                    {busy === entry.id
                      ? t('recycleBin.restoring')
                      : t('recycleBin.restore')}
                  </button>
                ) : (
                  <span className="badge">
                    {t(
                      automaticCleanup
                        ? 'recycleBin.automaticCleanup'
                        : 'recycleBin.manualCleanup',
                    )}
                  </span>
                )}
                <button
                  className="button danger"
                  onClick={() => remove(entry)}
                  disabled={busy !== null}
                >
                  {busy === `delete-${entry.id}`
                    ? t('recycleBin.deleting')
                    : t('recycleBin.delete')}
                </button>
              </div>
            </article>
          ))}
            </div>
          )}
        </>
      )}
    </>
  )
}

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  if (size < 1024 * 1024 * 1024)
    return `${(size / (1024 * 1024)).toFixed(1)} MB`
  return `${(size / (1024 * 1024 * 1024)).toFixed(1)} GB`
}
