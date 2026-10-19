import { useI18n } from '../../ui-resources/i18n'
import type { UpdaterCommand } from './update-types'
import { Icon } from '../../ui-resources/show-icon'
import { FeatureBanner } from './show-feature-banner'

const EXPECTED_UPDATER_PATH = 'shroudforge/shroudforge-updater.exe'
const LATEST_RELEASE_URL =
  'https://github.com/bonsaibauer/shroudforge/releases/latest'

export function UpdaterAvailability({
  updaterAvailable,
  enabled,
  onCommand,
}: {
  updaterAvailable: boolean
  enabled: boolean
  onCommand: UpdaterCommand
}) {
  const { t } = useI18n()
  if (updaterAvailable && enabled) return null

  return (
    <FeatureBanner
      variant={updaterAvailable ? 'info' : 'warning'}
      brand="SHROUDFORGE UPDATER"
      title={
        updaterAvailable
          ? t('updates.availability.disabled.title')
          : t('updates.availability.missing.title')
      }
      titleId="updater-availability-title"
      description={
        updaterAvailable
          ? t('updates.availability.disabled.description')
          : t('updates.availability.missing.description')
      }
      media={<Icon name="updates" />}
      badge={
        updaterAvailable
          ? t('updates.availability.disabled.badge')
          : t('updates.availability.missing.badge')
      }
      role="status"
    >
      {!updaterAvailable && (
        <>
          <span className="feature-banner-path-label">
            {t('updates.availability.pathLabel')}
          </span>
          <code className="feature-banner-path">
            {EXPECTED_UPDATER_PATH}
          </code>
          <p>{t('updates.availability.reinstall')}</p>
          <div className="feature-banner-actions">
            <button
              className="button ghost"
              onClick={() => onCommand('open-url', { url: LATEST_RELEASE_URL })}
            >
              {t('updates.availability.release')}
            </button>
          </div>
        </>
      )}
    </FeatureBanner>
  )
}
