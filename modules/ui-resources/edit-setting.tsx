import type React from 'react'
import { useEffect, useState } from 'react'

export function SettingsRow({
  label,
  detail,
  control,
  rowClass = '',
}: {
  label: React.ReactNode
  detail?: string
  control: React.ReactNode
  rowClass?: string
}) {
  return (
    <div className={`settings-option ${rowClass}`}>
      <span>
        {label}
        {detail && <small>{detail}</small>}
      </span>
      {control}
    </div>
  )
}

export function SettingSwitch({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string
  checked: boolean
  onChange(value: boolean): void
  disabled?: boolean
}) {
  return (
    <label
      className={`switch-control ${checked ? 'checked' : ''} ${disabled ? 'disabled' : ''}`}
    >
      <input
        type="checkbox"
        aria-label={label}
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.target.checked)}
      />
      <i />
    </label>
  )
}

export function ModuleEnabledRow({
  label,
  enabled,
  onChange,
  disabled = false,
}: {
  label: string
  enabled: boolean
  onChange(value: boolean): void
  disabled?: boolean
}) {
  return (
    <SettingsRow
      label={label}
      control={
        <SettingSwitch
          label={label}
          checked={enabled}
          onChange={onChange}
          disabled={disabled}
        />
      }
    />
  )
}

export function SettingsNumber({
  value,
  minimum,
  maximum,
  onCommit,
  disabled = false,
}: {
  value: number
  minimum?: number
  maximum?: number
  onCommit(value: number): void
  disabled?: boolean
}) {
  const [draft, setDraft] = useState(String(value))
  useEffect(() => setDraft(String(value)), [value])
  return (
    <input
      className="settings-number"
      type="number"
      min={minimum}
      max={maximum}
      disabled={disabled}
      value={draft}
      onChange={(event) => setDraft(event.target.value)}
      onBlur={() => {
        const parsed = Number(draft)
        if (draft.trim() !== '' && Number.isFinite(parsed)) {
          const next = Math.min(
            maximum ?? Number.MAX_SAFE_INTEGER,
            Math.max(minimum ?? Number.MIN_SAFE_INTEGER, parsed),
          )
          setDraft(String(next))
          if (next !== value) onCommit(next)
        } else setDraft(String(value))
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter') event.currentTarget.blur()
      }}
    />
  )
}

export function Toggle({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string
  checked: boolean
  onChange(v: boolean): void
  disabled?: boolean
}) {
  return (
    <label className="toggle-row">
      <span>{label}</span>
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <i />
    </label>
  )
}

export function Field({
  label,
  children,
}: {
  label: string
  children: React.ReactNode
}) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
  )
}
