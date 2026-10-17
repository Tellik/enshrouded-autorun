import type React from 'react'
import { Icon } from './show-icon'

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
  active,
  onClick,
}: {
  icon: string
  label: string
  badge?: number
  active: boolean
  onClick(): void
}) {
  return (
    <button className={`nav-item ${active ? 'active' : ''}`} onClick={onClick}>
      <Icon name={icon} />
      <span>{label}</span>
      {badge !== undefined && <em>{badge}</em>}
    </button>
  )
}
