import type React from 'react'
import { Icon } from './show-icon'

export type NavigationStatus = 'error' | 'warning' | 'ok' | 'neutral'

export function NavGroup({
  title,
  children,
  className,
}: {
  title: string
  children: React.ReactNode
  className?: string
}) {
  return (
    <section className={`nav-group ${className || ''}`}>
      <h2>{title}</h2>
      {children}
    </section>
  )
}

export function Nav({
  icon,
  label,
  badge,
  status,
  active,
  onClick,
}: {
  icon: string
  label: string
  badge?: number
  status?: NavigationStatus
  active: boolean
  onClick(): void
}) {
  return (
    <button className={`nav-item ${active ? 'active' : ''}`} onClick={onClick}>
      <Icon name={icon} />
      <span>{label}</span>
      {status && status !== 'neutral' ? (
        <i className={`nav-status ${status}`} aria-label={status}>
          <Icon
            name={status === 'error' ? 'issue' : status === 'warning' ? 'warning' : 'check'}
          />
        </i>
      ) : badge !== undefined ? (
        <em>{badge}</em>
      ) : null}
    </button>
  )
}
