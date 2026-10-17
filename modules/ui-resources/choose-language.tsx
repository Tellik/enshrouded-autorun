import { useState } from 'react'
import { Locale, useI18n } from './i18n'
import { postRequest } from '../transport/ui/send-host-command'
import { Flag } from './show-flag'
import { Icon } from './show-icon'

export function LanguageMenu() {
  const { locale, locales, setLocale } = useI18n()
  const [open, setOpen] = useState(false)
  const current = locales.find((option) => option.locale === locale)!
  const choose = (next: Locale) => {
    setLocale(next)
    postRequest('save-language', { locale: next })
    setOpen(false)
  }
  return (
    <div className="language">
      <button onClick={() => setOpen(!open)}>
        <Flag value={current.flag} />
        <span>{current.code}</span>
        <Icon name="chevron" />
      </button>
      {open && (
        <div className="language-popover">
          {locales.map((option) => (
            <button key={option.locale} onClick={() => choose(option.locale)}>
              <Flag value={option.flag} />
              <span>{option.name}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
