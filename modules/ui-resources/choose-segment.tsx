import type React from 'react'

export function Segmented({
  value,
  onChange,
  items,
}: {
  value: string
  onChange(v: string): void
  items: Array<[string, React.ReactNode]>
}) {
  return (
    <div className="segments">
      {items.map(([id, label]) => (
        <button
          className={value === id ? 'active' : ''}
          key={id}
          onClick={() => onChange(id)}
        >
          {label}
        </button>
      ))}
    </div>
  )
}
