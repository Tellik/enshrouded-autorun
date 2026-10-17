import type React from 'react'

export function Card({
  title,
  children,
  flush = false,
  tone = '',
  actions,
}: {
  title?: string
  children: React.ReactNode
  flush?: boolean
  tone?: string
  actions?: React.ReactNode
}) {
  return (
    <section className={`card ${flush ? 'flush' : ''} ${tone}`}>
      {title && (
        <header>
          <h2>{title}</h2>
          {actions}
        </header>
      )}
      <div className="card-body">{children}</div>
    </section>
  )
}
