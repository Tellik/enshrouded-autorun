import { useI18n } from '../../ui-resources/i18n'
import type { Release, UpdaterCommand } from './update-types'
import { Card } from '../../ui-resources/show-card'
import { Empty } from '../../ui-resources/show-empty-state'
import { ReleaseNotes } from './show-release-notes'

export function SystemUpdate({
  release,
  queued,
  disabled = false,
  onCommand,
}: {
  release: Release
  queued: boolean
  disabled?: boolean
  onCommand: UpdaterCommand
}) {
  const { t } = useI18n()
  if (release.updateAvailable)
    return (
      <Card title={`ShroudForge ${release.latestVersion}`} tone="update">
        {release.mandatory && (
          <span className="release-channel-chip">{t('updater.required')}</span>
        )}
        {release.prerelease && (
          <span className="release-channel-chip">
            {t('updates.prerelease.badge')}
          </span>
        )}
        <ReleaseNotes
          text={release.message || t('updates.version.available')}
          onCommand={onCommand}
        />
        <div className="inline-actions">
          <button
            className="button"
            onClick={() => onCommand('queue-system-update')}
            disabled={disabled || queued}
          >
            {queued ? t('updates.queue.added') : t('updates.queue.addSystem')}
          </button>
          {release.releaseUrl && (
            <button
              className="button ghost"
              onClick={() => onCommand('open-url', { url: release.releaseUrl })}
            >
              {t('updates.release')}
            </button>
          )}
        </div>
        <p className="hint">{t('updates.safety')}</p>
      </Card>
    )
  const message =
    release.state === 'ready' && !release.latestVersion
      ? t('updates.none.notPublished')
      : release.state === 'ready'
        ? t('updates.none.message')
        : release.state === 'checking'
          ? t('updates.checking')
          : release.state === 'idle'
            ? t('updates.none.notChecked')
            : release.message || t('updates.none.message')
  return <Empty title={t('updates.none.title')} text={message} icon="check" />
}
