import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { Release } from './update-types'

export function UpdateProgress({ release }: { release: Release }) {
  const { t } = useI18n()
  const totalFiles = release.totalFiles || 0
  const completedFiles = release.completedFiles || 0
  const currentFileProgress = release.currentFileTotalBytes
    ? Math.min(
        1,
        (release.currentFileBytes || 0) / release.currentFileTotalBytes,
      )
    : 0
  const fileProgress = totalFiles
    ? Math.max(
        0,
        Math.min(100, ((completedFiles + currentFileProgress) / totalFiles) * 100),
      )
    : undefined
  const total = release.totalBytes || 0
  const downloaded = release.downloadedBytes || 0
  const installPhase = ['installing', 'restoring'].includes(release.state)
  const cancellationPending = release.state === 'cancelled'
  const finished = release.state === 'installed'
  const downloadProgress = total
    ? Math.max(0, Math.min(80, (downloaded / total) * 80))
    : undefined
  const progress = finished
    ? 100
    : cancellationPending
      ? undefined
      : installPhase
      ? 85 + (fileProgress || 0) * 0.14
      : release.state === 'verifying'
        ? 81
        : release.state === 'extracting'
          ? 82
          : release.state === 'staged'
            ? 84
            : release.state === 'waitingForGame'
              ? total && downloaded >= total
                ? 84
                : undefined
              : downloadProgress
  const speed = release.state === 'downloading' && release.bytesPerSecond
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
    restoring: 'updater.step.installing',
    installed: 'updater.complete',
  }
  return (
    <div className="updater-progress-card">
      <div className="updater-progress-label">
        <strong>
          {cancellationPending
            ? release.message || t('updater.preparing')
            : progress === undefined
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
        {!cancellationPending && !installPhase && (total || downloaded) > 0 && (
          <span>
            {total
              ? `${(downloaded / 1024 / 1024).toFixed(1)} / ${(total / 1024 / 1024).toFixed(1)} MB`
              : `${(downloaded / 1024 / 1024).toFixed(1)} MB`}
          </span>
        )}
        {installPhase && release.currentFile && (
          <span>{release.currentFile}</span>
        )}
        {!cancellationPending && (
          <span>{t(stepKeys[release.state] || 'updater.preparing')}</span>
        )}
      </div>
    </div>
  )
}
