import { useI18n } from './i18n'

export function EcosystemBadge({ ecosystem }: { ecosystem?: 'EML' | 'SF' }) {
  const { t } = useI18n()
  return ecosystem ? (
    <span className={`ecosystem-badge ${ecosystem.toLowerCase()}`}>
      {t(ecosystem === 'SF' ? 'mod.ecosystem.sf' : 'mod.ecosystem.eml')}
    </span>
  ) : null
}
