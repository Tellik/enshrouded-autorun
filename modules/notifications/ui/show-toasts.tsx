import { useRef, useState } from 'react'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import { EcosystemBadge } from '../../ui-resources/show-ecosystem-badge'
import type { CommandResult } from './notification-types'

type ToastMessage = {
  success: boolean
  message: string
  ecosystem?: 'EML' | 'SF'
}

export function useCommandToasts() {
  const { t } = useI18n()
  const [message, setMessage] = useState<ToastMessage | null>(null)
  const tRef = useRef(t)
  tRef.current = t
  const toastTimer = useRef<number | undefined>(undefined)
  const settingsBatchTimer = useRef<number | undefined>(undefined)
  const settingsBatchCount = useRef(0)
  const notifyCommandResult = (result: CommandResult) => {
    let message = result.message
    if (result.success && result.batchKey === 'settings') {
      settingsBatchCount.current += 1
      message =
        settingsBatchCount.current === 1
          ? tRef.current('settings.settingSaved')
          : tRef.current('settings.settingsSaved', {
              count: settingsBatchCount.current,
            })
      if (settingsBatchTimer.current !== undefined)
        window.clearTimeout(settingsBatchTimer.current)
      settingsBatchTimer.current = window.setTimeout(() => {
        settingsBatchCount.current = 0
        settingsBatchTimer.current = undefined
      }, 900)
    } else {
      settingsBatchCount.current = 0
      if (settingsBatchTimer.current !== undefined) {
        window.clearTimeout(settingsBatchTimer.current)
        settingsBatchTimer.current = undefined
      }
      if (result.success)
        message = tRef.current(result.message as TranslationKey)
    }
    setMessage({
      success: result.success,
      message,
      ecosystem: result.ecosystem || undefined,
    })
    if (toastTimer.current !== undefined)
      window.clearTimeout(toastTimer.current)
    toastTimer.current = window.setTimeout(
      () => {
        setMessage(null)
        toastTimer.current = undefined
      },
      result.success ? 4500 : 8000,
    )
  }
  return { message, notifyCommandResult }
}

export function Toasts({
  errors,
  message,
}: {
  errors?: string[]
  message: ToastMessage | null
}) {
  const { t } = useI18n()
  return Boolean(errors?.length) || Boolean(message) ? (
    <div className="toast-stack" aria-live="polite">
      {Boolean(errors?.length) && (
        <div role="alert" className="settings-save-message persistent error">
          <strong>{t('settings.configuration.title')}</strong>
          {errors?.map((error, index) => (
            <p key={error || index}>{error}</p>
          ))}
        </div>
      )}
      {message && (
        <div
          role={message.success ? 'status' : 'alert'}
          className={`settings-save-message ${message.success ? 'success' : 'error'}`}
        >
          <EcosystemBadge ecosystem={message.ecosystem} />
          {message.message}
        </div>
      )}
    </div>
  ) : null
}
