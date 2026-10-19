import type { Settings } from '../../loader-config/ui/loader-settings'
import { post, postRequest } from '../../transport/ui/send-host-command'
import { useEffect, useState } from 'react'
import { I18nProvider, useI18n } from '../../ui-resources/i18n'
import { PageId, Snapshot, empty } from './ui-snapshot'
import type { NoticeAction } from '../../notifications/ui/notification-types'
import { Chirper } from '../../notifications/ui/show-notice-popup'
import { useNotifications } from '../../notifications/ui/read-notifications'
import { Toasts, useCommandToasts } from '../../notifications/ui/show-toasts'
import { Nav, NavGroup } from '../../ui-resources/show-navigation'
import { ModIcon } from './show-mod-icon'
import { ModPage } from './show-mod'
import { useUiUpdates } from './receive-ui-updates'
import { TopBar } from './show-top-bar'
import { Page } from './show-page'
import { UiErrorBoundary } from './show-ui-error'
import { createRoot } from 'react-dom/client'
import './styles.css'

export function App() {
  const { t, setLocale } = useI18n()
  const [data, setData] = useState<Snapshot>(empty)
  const [page, setPage] = useState<PageId>('news')
  const [selectedMod, setSelectedMod] = useState<string | null>(null)
  const [chirper, setChirper] = useState(false)
  const [settings, setSettings] = useState<Settings>(empty.settings)
  const [read, setRead] = useState<string[]>([])
  const { message: saveMessage, notifyCommandResult } = useCommandToasts()
  useUiUpdates({
    data,
    setData,
    setSettings,
    setRead,
    setLocale,
    setPage,
    setSelectedMod,
    notifyCommandResult,
  })
  useEffect(() => {
    document.body.classList.toggle('compact', settings.compactMode)
    document.body.classList.toggle('reduced-motion', settings.reducedMotion)
  }, [settings.compactMode, settings.reducedMotion])

  const {
    news,
    unread,
    chirperSeverity,
    chirperTitle,
    chirperItems,
    markRead,
  } = useNotifications(data, settings, read, setRead)
  const activeMod = data.mods.find((mod) => mod.id === selectedMod)
  const navigate = (next: PageId) => {
    setSelectedMod(null)
    setPage(next)
  }

  const runNoticeAction = (action: NoticeAction) => {
    if (action.kind === 'open-installed-mods') navigate('installed')
    else if (action.kind === 'open-mod' && action.target) {
      if (data.mods.some((mod) => mod.id === action.target)) {
        setPage('installed')
        setSelectedMod(action.target)
      } else navigate('installed')
    } else if (action.kind === 'open-debug-console')
      postRequest('set-window-visibility', {
        module: 'debugConsole',
        visible: true,
      })
    else if (action.kind === 'check-updates') postRequest('check-updates')
    else if (action.kind === 'queue-system-update')
      postRequest('queue-system-update')
  }

  return (
    <main className="shell">
      <Toasts errors={data.configuration?.errors} message={saveMessage} />
      <TopBar
        data={data}
        unread={unread.length}
        severity={chirperSeverity}
        crowded={unread.length >= 5}
        title={chirperTitle}
        onChirper={() => setChirper(true)}
      />
      <div className="workspace">
        <aside className="sidebar">
          <NavGroup title={t('nav.modloader')}>
            <Nav
              icon="activity"
              label={t('nav.activity')}
              active={!activeMod && page === 'activity'}
              onClick={() => navigate('activity')}
            />
            <Nav
              icon="news"
              label={t('nav.news')}
              badge={unread.length || undefined}
              active={!activeMod && page === 'news'}
              onClick={() => navigate('news')}
            />
            <Nav
              icon="compatibility"
              label={t('nav.compatibility')}
              active={!activeMod && page === 'compatibility'}
              onClick={() => navigate('compatibility')}
            />
            <Nav
              icon="settings"
              label={t('nav.settings')}
              active={!activeMod && page === 'settings'}
              onClick={() => navigate('settings')}
            />
          </NavGroup>
          <NavGroup title={t('nav.modsManagement')}>
            <Nav
              icon="discover"
              label={t('nav.discover')}
              active={!activeMod && page === 'discover'}
              onClick={() => navigate('discover')}
            />
            <Nav
              icon="installed"
              label={t('nav.installed')}
              badge={data.mods.length || undefined}
              active={!activeMod && page === 'installed'}
              onClick={() => navigate('installed')}
            />
            <Nav
              icon="updates"
              label={t('nav.updates')}
              badge={data.release.updateAvailable ? 1 : undefined}
              active={!activeMod && page === 'updates'}
              onClick={() => navigate('updates')}
            />
            <Nav
              icon="trash"
              label={t('nav.recycleBin')}
              active={!activeMod && page === 'recycle-bin'}
              onClick={() => navigate('recycle-bin')}
            />
          </NavGroup>
          <NavGroup className="nav-group-mods" title={t('nav.mods')}>
            <div className="sidebar-mods">
              {data.mods.map((mod) => (
                <button
                  key={mod.id}
                  className={`nav-item ${activeMod?.id === mod.id ? 'active' : ''}`}
                  onClick={() => setSelectedMod(mod.id)}
                >
                  <ModIcon mod={mod} variant="nav" />
                  <span>{mod.name}</span>
                </button>
              ))}
            </div>
          </NavGroup>
        </aside>
        <section className="content">
          {activeMod ? (
            <ModPage
              mod={activeMod}
              status={data.configuration?.modStates?.[activeMod.id]}
              dependencyStates={data.configuration?.modStates || {}}
            />
          ) : (
            <Page
              id={page}
              data={data}
              settings={settings}
              setSettings={setSettings}
              news={news}
              unread={unread}
              read={read}
              markRead={markRead}
              runNoticeAction={runNoticeAction}
              openNews={() => navigate('news')}
              selectMod={setSelectedMod}
              onUpdaterCommand={postRequest}
            />
          )}
        </section>
      </div>
      {chirper && (
        <Chirper
          items={chirperItems}
          close={() => setChirper(false)}
          all={() => {
            setChirper(false)
            navigate('news')
          }}
          markRead={markRead}
          runNoticeAction={runNoticeAction}
        />
      )}
    </main>
  )
}

window.addEventListener('error', (event) =>
  post('ui-error', {
    message: `${event.message} (${event.filename}:${event.lineno}:${event.colno})`,
  }),
)

window.addEventListener('unhandledrejection', (event) =>
  post('ui-error', {
    message: `Unhandled promise rejection: ${String(event.reason)}`,
  }),
)

createRoot(document.getElementById('root')!).render(
  <I18nProvider>
    <UiErrorBoundary>
      <App />
    </UiErrorBoundary>
  </I18nProvider>,
)
