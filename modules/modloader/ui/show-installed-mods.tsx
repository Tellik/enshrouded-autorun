import type { ModInfo } from './mod-types'
import { useI18n } from '../../ui-resources/i18n'

import { Card } from '../../ui-resources/show-card'
import { EcosystemBadge } from '../../ui-resources/show-ecosystem-badge'
import { Empty } from '../../ui-resources/show-empty-state'
import { PageHeader } from '../../ui-resources/show-page-header'
import { ModIcon } from './show-mod-icon'

export function Installed({
  mods,
  select,
}: {
  mods: ModInfo[]
  select(id: string): void
}) {
  const { t } = useI18n()
  return (
    <>
      <PageHeader
        eyebrow={t('nav.mods')}
        title={t('installed.title')}
        subtitle={t('installed.subtitle')}
      />
      <Card flush>
        {mods.length ? (
          <div className="mod-list">
            {mods.map((mod) => (
              <button
                className="mod-row"
                key={mod.id}
                onClick={() => select(mod.id)}
              >
                <ModIcon mod={mod} variant="list" />
                <span className="mod-name">
                  <strong title={mod.name}>
                    {mod.name}
                    <EcosystemBadge ecosystem={mod.ecosystem} />
                  </strong>
                  <small title={mod.description || mod.id}>
                    {mod.description || mod.id}
                  </small>
                </span>
                <span className="mod-version">{mod.version}</span>
                <span className="chevron">›</span>
              </button>
            ))}
          </div>
        ) : (
          <Empty
            title={t('installed.empty.title')}
            text={t('installed.empty.text')}
            icon="installed"
          />
        )}
      </Card>
    </>
  )
}
