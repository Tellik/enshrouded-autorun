import type { Settings } from '../../loader-config/ui/loader-settings'
import { PageId, Snapshot } from './ui-snapshot'
import type {
  NewsItem,
  NoticeAction,
} from '../../notifications/ui/notification-types'
import { NewsPage } from '../../notifications/ui/show-news'
import { Updates } from '../../updater/ui/show-updates'
import { Discover } from '../../updater/ui/show-mod-catalog'
import { Installed } from './show-installed-mods'
import { SettingsPage } from '../../loader-config/ui/show-loader-settings'
import { ActivityPage } from '../../logging/ui/show-activity'
import { Compatibility } from './show-compatibility'

export function Page({
  id,
  data,
  settings,
  setSettings,
  news,
  unread,
  read,
  markRead,
  runNoticeAction,
  openNews,
  selectMod,
}: {
  id: PageId
  data: Snapshot
  settings: Settings
  setSettings(v: Settings): void
  news: NewsItem[]
  unread: NewsItem[]
  read: string[]
  markRead(ids: string[], isRead?: boolean): void
  runNoticeAction(action: NoticeAction): void
  openNews(): void
  selectMod(id: string): void
}) {
  if (id === 'activity')
    return <ActivityPage activity={data.activity} mods={data.mods} />
  if (id === 'news')
    return (
      <NewsPage
        items={news}
        unread={unread}
        read={read}
        markRead={markRead}
        runNoticeAction={runNoticeAction}
      />
    )
  if (id === 'discover') return <Discover data={data} />
  if (id === 'installed')
    return <Installed mods={data.mods} select={selectMod} />
  if (id === 'updates') return <Updates data={data} />
  if (id === 'compatibility')
    return (
      <Compatibility
        data={data}
        news={news}
        openNews={openNews}
        selectMod={selectMod}
        openLog={() =>
          runNoticeAction({
            id: 'open-debug-console',
            kind: 'open-debug-console',
          })
        }
      />
    )
  return (
    <SettingsPage data={data} settings={settings} setSettings={setSettings} />
  )
}
