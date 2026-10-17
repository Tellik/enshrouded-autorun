import type { ModDependencyInfo } from './mod-types'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'

export function DependencyList({
  dependencies,
  dependencyStates,
}: {
  dependencies: ModDependencyInfo[]
  dependencyStates: Record<string, { state: string; detail: string }>
}) {
  const { t } = useI18n()
  if (!dependencies.length)
    return <p className="hint">{t('mod.dependencies.empty.text')}</p>
  const labels: Record<string, TranslationKey> = {
    installed: 'mod.dependencies.status.installed',
    missing: 'mod.dependencies.status.missing',
    'optional-missing': 'mod.dependencies.status.optionalMissing',
    disabled: 'mod.dependencies.status.disabled',
    'optional-disabled': 'mod.dependencies.status.optionalDisabled',
    'version-mismatch': 'mod.dependencies.status.versionMismatch',
    'invalid-version': 'mod.dependencies.status.invalidVersion',
    'loader-api': 'mod.dependencies.status.loaderApi',
    blocked: 'mod.dependencies.status.blocked',
  }
  return (
    <div className="settings-options dependency-list">
      {dependencies.map((dependency) => {
        const resolved = dependencyStates[dependency.id]
        const state =
          resolved?.state === 'blocked' || resolved?.state === 'failed'
            ? 'blocked'
            : dependency.status
        return (
          <div
            className={`settings-option dependency-row ${state}`}
            key={`${dependency.id}-${dependency.version}`}
            title={resolved?.detail}
          >
            <span>
              <strong>
                {dependency.id}
                {dependency.optional && (
                  <small>{t('mod.dependencies.optional')}</small>
                )}
              </strong>
              <small>
                {t('mod.dependencies.requirement', {
                  version: dependency.version,
                })}
                {dependency.installedVersion
                  ? ` · ${t('mod.dependencies.installedVersion', { version: dependency.installedVersion })}`
                  : ''}
              </small>
            </span>
            <span className="dependency-status">
              {t(labels[state] || 'mod.dependencies.status.invalidVersion')}
            </span>
          </div>
        )
      })}
    </div>
  )
}
