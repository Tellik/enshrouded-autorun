import { readUiStorage, writeUiStorage } from './store-ui-preferences'
import type { Settings } from '../../loader-config/ui/loader-settings'
import { post, postRequest } from '../../transport/ui/send-host-command'
import { useEffect, useRef } from 'react'
import type { Dispatch, SetStateAction } from 'react'
import { Snapshot, PageId, startPageOptions } from './ui-snapshot'
import type { CommandResult } from '../../notifications/ui/notification-types'

type SetValue<T> = Dispatch<SetStateAction<T>>
type UiUpdateActions = {
  data: Snapshot
  setData: SetValue<Snapshot>
  setSettings: SetValue<Settings>
  setRead: SetValue<string[]>
  setLocale(locale: string): void
  setPage: SetValue<PageId>
  setSelectedMod: SetValue<string | null>
  notifyCommandResult(result: CommandResult): void
}

export function useUiUpdates({
  data,
  setData,
  setSettings,
  setRead,
  setLocale,
  setPage,
  setSelectedMod,
  notifyCommandResult,
}: UiUpdateActions) {
  const previousStartPage = useRef<string | undefined>(undefined)
  const pendingStartPage = useRef<string | undefined>(undefined)
  const previousWindowVisible = useRef<boolean | undefined>(undefined)
  const handledSettingsRequest = useRef<number>(
    Number(readUiStorage('shroudforge-world-editor-settings-request', '0')),
  )
  const localeHydrated = useRef(false)
  const dataRef = useRef(data)
  useEffect(() => {
    dataRef.current = data
  }, [data])

  useEffect(() => {
    post('ui-ready')
    window.__shroudforgeUpdate = (next) => {
      const snapshot = next as Snapshot
      const start = snapshot.settings.modulePreferences?.modloaderUi?.startPage
      const validStart = startPageOptions.includes(start)
      const visible = Boolean(snapshot.windows?.modloaderUi?.visible)
      if (!localeHydrated.current && snapshot.locale) {
        localeHydrated.current = true
        setLocale(snapshot.locale)
        setPage(validStart ? start : 'news')
      } else if (
        start &&
        previousStartPage.current &&
        start !== previousStartPage.current &&
        visible
      ) {
        pendingStartPage.current = start
      }
      if (previousWindowVisible.current === false && visible) {
        const nextPage = pendingStartPage.current || start
        setPage(startPageOptions.includes(nextPage) ? nextPage : 'news')
        pendingStartPage.current = undefined
      }
      if (start) previousStartPage.current = start
      previousWindowVisible.current = visible
      setData(snapshot)
      setSettings((current) =>
        JSON.stringify(current) === JSON.stringify(dataRef.current.settings)
          ? snapshot.settings
          : current,
      )
      setRead(snapshot.readNoticeIds || [])
    }
    window.__shroudforgeWindowState = (state) => {
      const windows = state as Snapshot['windows']
      const moduleWindow = windows?.modloaderUi
      const requestId = moduleWindow?.focusRequestId || 0
      if (
        moduleWindow?.focusTarget === 'worldEditorSettings' &&
        requestId > handledSettingsRequest.current
      ) {
        handledSettingsRequest.current = requestId
        writeUiStorage(
          'shroudforge-world-editor-settings-request',
          String(requestId),
        )
        if (!moduleWindow.visible) pendingStartPage.current = 'settings'
        setSelectedMod(null)
        setPage('settings')
        window.setTimeout(
          () =>
            document
              .getElementById('settings-module-worldEditor')
              ?.scrollIntoView({ behavior: 'smooth', block: 'center' }),
          120,
        )
      }
      setData((current) => ({ ...current, windows }))
    }
    window.__shroudforgeCommandResult = (result) => {
      if (!result.success && result.restoreSettings)
        setSettings(dataRef.current.settings)
      notifyCommandResult(result)
    }
    post('refresh')
    const drag = (event: MouseEvent) => {
      const target = event.target as HTMLElement
      if (
        !target.closest('button,input,select,a') &&
        target.closest('[data-drag-region]')
      )
        postRequest('drag')
    }
    document.addEventListener('mousedown', drag)
    const escape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') postRequest('hide')
    }
    document.addEventListener('keydown', escape)
    return () => {
      document.removeEventListener('mousedown', drag)
      document.removeEventListener('keydown', escape)
    }
  }, [])
}
