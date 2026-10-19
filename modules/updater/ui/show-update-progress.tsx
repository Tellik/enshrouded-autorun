import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { Release } from './update-types'

export function UpdateProgress({ release }: { release: Release }) {
  const { t } = useI18n()
  const total = release.totalBytes || 0
  const downloaded = release.downloadedBytes || 0
  const progress = total
    ? Math.max(0, Math.min(100, (downloaded / total) * 100))
    : undefined
  const speed = release.bytesPerSecond
    ? `${(release.bytesPerSecond / 1024 / 1024).toFixed(1)} MB/s`
    : ''
  const stepKeys: Record<string, TranslationKey> = {
    queued: 'updater.step.queued',
    downloading: 'updater.step.downloading',
    verifying: 'updater.step.verifying',
    extracting: 'updater.step.extracting',
    staged: 'updater.step.staged',
    waitingForGame: 'updater.step.waitingForGame',
    installing: 'updater.step.installing',
  }
  return (
    <div className="updater-progress-card">
      <div className="updater-progress-label">
        <strong>
          {progress === undefined
            ? t('updater.preparing')
            : t('updater.progress', { percent: Math.floor(progress) })}
        </strong>
        <span>{speed}</span>
      </div>
      <div
        className={`updater-progress-track ${progress === undefined ? 'indeterminate' : ''}`}
      >
        <i
          style={progress === undefined ? undefined : { width: `${progress}%` }}
        />
      </div>
      <div className="updater-progress-meta">
        {(total || downloaded) > 0 && (
          <span>
            {total
              ? `${(downloaded / 1024 / 1024).toFixed(1)} / ${(total / 1024 / 1024).toFixed(1)} MB`
              : `${(downloaded / 1024 / 1024).toFixed(1)} MB`}
          </span>
        )}
        <span>{t(stepKeys[release.state] || 'updater.preparing')}</span>
      </div>
    </div>
  )
}
