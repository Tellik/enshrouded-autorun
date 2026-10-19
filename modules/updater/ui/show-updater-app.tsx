import { useEffect, useState } from 'react'
import { postRequest } from '../../transport/ui/send-host-command'
import { useI18n } from '../../ui-resources/i18n'
import { Icon } from '../../ui-resources/show-icon'
import { Updates } from './show-updates'
import type {
  ModUpdate,
  ModUpdateCheckResult,
  Release,
  UpdateQueue,
  UpdaterViewData,
} from './update-types'

type UpdaterSnapshot = {
  root: string
  version: string
  release: Release
  queue: UpdateQueue
  modUpdates?: ModUpdate[]
  modCheckResult?: ModUpdateCheckResult | null
  modCheckRunning?: boolean
  locale?: string
  updatesEnabled?: boolean
}

type UpdaterWindow = Window & {
  __shroudforgeUpdaterSnapshot?: (snapshot: UpdaterSnapshot) => void
  __shroudforgeUpdaterResult?: (result: {
    error?: string
  }) => void
}

const initialSnapshot: UpdaterSnapshot = {
  root: '',
  version: '',
  release: {
    currentVersion: '',
    updaterAvailable: true,
    updateAvailable: false,
    state: 'idle',
    staged: false,
  },
  queue: { items: [] },
}

export function UpdaterApp() {
  const { t, setLocale } = useI18n()
  const [snapshot, setSnapshot] = useState(initialSnapshot)
  const [message, setMessage] = useState('')

  useEffect(() => {
    const updaterWindow = window as UpdaterWindow
    updaterWindow.__shroudforgeUpdaterSnapshot = setSnapshot
    updaterWindow.__shroudforgeUpdaterResult = (result) =>
      setMessage(result.error || '')
    return () => {
      delete updaterWindow.__shroudforgeUpdaterSnapshot
      delete updaterWindow.__shroudforgeUpdaterResult
    }
  }, [])

  useEffect(() => {
    if (snapshot.locale) setLocale(snapshot.locale)
  }, [snapshot.locale, setLocale])

  const data: UpdaterViewData = {
    mode: 'DESKTOP',
    release: { ...snapshot.release, updaterAvailable: true },
    enabled: snapshot.updatesEnabled === true,
    updateQueue: snapshot.queue,
    modUpdates: snapshot.modUpdates,
    modCheckResult: snapshot.modCheckResult,
    modCheckRunning: snapshot.modCheckRunning,
  }
  const command = (name: string, payload?: Record<string, unknown>) => {
    setMessage('')
    postRequest(name, payload)
  }

  return (
    <main className="shell updater-standalone-shell">
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark">SF</span>
          <span>
            <strong>SHROUDFORGE</strong>
            <small>UPDATER</small>
          </span>
        </div>
        <div className="updater-installation">
          <span title={snapshot.root}>
            {snapshot.root || t('updater.root.missing')}
          </span>
          <button
            className="header-link"
            onClick={() => command('choose-root')}
            title={t('updater.root.choose')}
          >
            <Icon name="folder" />
            <span>{t('updater.root.choose')}</span>
          </button>
        </div>
      </header>
      <div className="updater-standalone-workspace">
        <section className="content updater-standalone-content">
          <Updates data={data} onCommand={command} commandError={message} />
        </section>
        <footer className="updater-standalone-status">
          <span>{t('updater.installedVersion', { version: snapshot.version || '—' })}</span>
        </footer>
      </div>
    </main>
  )
}
