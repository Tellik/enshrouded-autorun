import type React from 'react'

export function PageHeader({
  eyebrow,
  title,
  subtitle,
  actions,
}: {
  eyebrow: string
  title: React.ReactNode
  subtitle: string
  actions?: React.ReactNode
}) {
  return (
    <header className="page-header">
      <div>
        <small>{eyebrow}</small>
        <h1>{title}</h1>
        <p>{subtitle}</p>
      </div>
      {actions && <div className="page-actions">{actions}</div>}
    </header>
  )
}
