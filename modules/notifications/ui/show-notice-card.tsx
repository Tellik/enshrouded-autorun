import { postRequest } from '../../transport/ui/send-host-command'
import { formatTime } from '../../ui-resources/format-time'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { NewsItem, NoticeAction } from './notification-types'

import { EcosystemBadge } from '../../ui-resources/show-ecosystem-badge'
import { Icon } from '../../ui-resources/show-icon'
import { ContributorList } from './show-contributors'
export function noticeIcon(item: NewsItem) {
  if (item.level === 'error') return 'issue'
  if (item.level === 'warning') return 'warning'
  if (item.level === 'update') return 'updates'
  if (item.level === 'success' && item.kind !== 'mod.remove.success')
    return 'check'
  switch (item.kind) {
    case 'notice.welcome':
      return 'heart'
    case 'notice.modsGuide':
      return 'installed'
    case 'notice.configuration':
      return 'settings'
    case 'notice.discord':
      return 'discord'
    case 'notice.applySettings':
      return 'refresh'
    case 'notice.feedback':
      return 'issue'
    case 'notice.support':
      return 'coffee'
    case 'mod.install.success':
      return 'module'
    case 'mod.update.success':
      return 'updates'
    case 'mod.remove.success':
      return 'trash'
    default:
      return 'news'
  }
}

export function NoticeCard({
  item,
  compact = false,
  unread = false,
  onRead,
  onAction,
}: {
  item: NewsItem
  compact?: boolean
  unread?: boolean
  onRead?(): void
  onAction?(action: NoticeAction): void
}) {
  const { t, locale } = useI18n()
  const actions = item.actions || []
  const links = item.links || []
  const localizedUrl = (url: string) =>
    url.replace('{locale}', locale === 'de' ? 'de' : 'en')
  return (
    <article
      data-kind={item.kind}
      className={`notice ${item.level} ${compact ? 'compact' : ''} ${unread ? 'unread' : ''}`}
    >
      <span className="notice-icon">
        <Icon name={noticeIcon(item)} />
      </span>
      <div>
        <div className="notice-title">
          <h3>{item.title}</h3>
          <EcosystemBadge ecosystem={item.ecosystem} />
          {unread && <i />}
        </div>
        <p>{item.message}</p>
        {!compact && item.kind === 'notice.welcome' && <ContributorList />}
        {!compact && item.changelog?.length ? (
          <ul className="notice-changelog">
            {item.changelog.map((entry, index) => (
              <li key={`${index}-${entry}`}>{entry}</li>
            ))}
          </ul>
        ) : null}
        {(!compact ||
          actions.length > 0 ||
          item.actionUrl ||
          links.length > 0 ||
          onRead) && (
          <footer>
            {item.updatedAt > 0 && (
              <time>{formatTime(item.updatedAt, locale)}</time>
            )}
            <span />
            {item.actionUrl && (
              <button
                onClick={() =>
                  postRequest('open-url', {
                    url: localizedUrl(item.actionUrl!),
                  })
                }
              >
                {item.actionLabelKey
                  ? t(item.actionLabelKey as TranslationKey)
                  : t('news.more')}
              </button>
            )}
            {links.map((link, index) => (
              <button
                key={`${link.labelKey}-${index}`}
                onClick={() =>
                  postRequest('open-url', { url: localizedUrl(link.url) })
                }
              >
                {t(link.labelKey as TranslationKey)}
              </button>
            ))}
            {actions.map((action) => (
              <button key={action.id} onClick={() => onAction?.(action)}>
                {t(`news.action.${action.kind}` as TranslationKey)}
              </button>
            ))}
            {onRead && (
              <button onClick={onRead}>
                {t(unread ? 'news.markRead' : 'news.markUnread')}
              </button>
            )}
          </footer>
        )}
      </div>
    </article>
  )
}
