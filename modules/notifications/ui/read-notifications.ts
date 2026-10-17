import { postRequest } from '../../transport/ui/send-host-command'
import { useMemo } from 'react'
import type { Dispatch, SetStateAction } from 'react'
import { useI18n } from '../../ui-resources/i18n'

import type {
  NewsItem,
  NewsDisplaySource,
  NewsPersonalization,
} from './notification-types'
import { buildNewsItems } from './build-news-items'

export function useNotifications(
  data: NewsDisplaySource,
  settings: NewsPersonalization,
  read: string[],
  setRead: Dispatch<SetStateAction<string[]>>,
) {
  const { t } = useI18n()
  const news = useMemo<NewsItem[]>(
    () => buildNewsItems(data, settings, t),
    [data, t, settings.messageNameFallback],
  )
  const unread = news.filter((item) => !read.includes(item.id))
  const errorCount = unread.filter((item) => item.level === 'error').length
  const warningCount = unread.filter((item) => item.level === 'warning').length
  const updateCount = unread.filter((item) => item.level === 'update').length
  const chirperSeverity = errorCount
    ? 'error'
    : warningCount
      ? 'warning'
      : unread.length >= 5
        ? 'many'
        : updateCount
          ? 'update'
          : unread.length
            ? 'info'
            : 'quiet'
  const chirperTitle = errorCount
    ? t('header.chirper.errors', { count: errorCount })
    : warningCount
      ? t('header.chirper.warnings', { count: warningCount })
      : t('header.chirper.unread', { count: unread.length })
  const chirperItems = [...unread].sort((a, b) => {
    const priority: Record<string, number> = {
      error: 0,
      warning: 1,
      update: 2,
      success: 3,
      info: 4,
    }
    return (
      (priority[a.level] ?? 4) - (priority[b.level] ?? 4) ||
      b.updatedAt - a.updatedAt
    )
  })

  const markRead = (ids: string[], isRead = true) => {
    const next = isRead
      ? Array.from(new Set([...read, ...ids]))
      : read.filter((id) => !ids.includes(id))
    setRead(next)
    postRequest('set-news-read', { ids, read: isRead })
  }
  return { news, unread, chirperSeverity, chirperTitle, chirperItems, markRead }
}
