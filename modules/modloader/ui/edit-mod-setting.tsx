import type { SettingDefinition } from './mod-types'
import type React from 'react'

import { SettingSwitch, SettingsRow } from '../../ui-resources/edit-setting'

export function ModSetting({
  definition,
  value,
  onChange,
}: {
  definition: SettingDefinition
  value: unknown
  onChange(value: unknown): void
}) {
  const control =
    definition.control ||
    (definition.type === 'boolean'
      ? 'toggle'
      : definition.type === 'number' || definition.type === 'integer'
        ? 'number'
        : 'text')
  const numeric = definition.type === 'number' || definition.type === 'integer'
  let input: React.ReactNode
  if (control === 'toggle' || control === 'checkbox')
    input = (
      <SettingSwitch
        label={definition.label}
        checked={Boolean(value)}
        onChange={(checked) => onChange(checked)}
      />
    )
  else if (control === 'slider')
    input = (
      <div className="slider-control">
        <input
          type="range"
          value={Number(value ?? definition.default)}
          min={definition.minimum}
          max={definition.maximum}
          step={definition.step}
          onChange={(event) => onChange(Number(event.target.value))}
        />
        <output>{String(value ?? definition.default)}</output>
      </div>
    )
  else if (control === 'select')
    input = (
      <select
        value={String(value ?? '')}
        onChange={(event) =>
          onChange(optionValue(definition, event.target.value))
        }
      >
        {(definition.options || []).map((option) => (
          <option key={String(option.value)} value={String(option.value)}>
            {option.label}
          </option>
        ))}
      </select>
    )
  else if (control === 'radio' || control === 'segmented')
    input = (
      <div className={'choice-control ' + control}>
        {(definition.options || []).map((option) => (
          <button
            type="button"
            className={String(value) === String(option.value) ? 'active' : ''}
            key={String(option.value)}
            onClick={() => onChange(option.value)}
          >
            {option.label}
          </button>
        ))}
      </div>
    )
  else if (control === 'multiselect')
    return (
      <div className="mod-multiselect">
        <div className="mod-multiselect-title">
          <strong>{definition.label}</strong>
          {definition.description && <small>{definition.description}</small>}
        </div>
        <div className="settings-options">
          {(definition.options || []).map((option) => {
            const selected =
              Array.isArray(value) &&
              value.some((item) => String(item) === String(option.value))
            return (
              <SettingsRow
                key={String(option.value)}
                label={option.label}
                control={
                  <SettingSwitch
                    label={option.label}
                    checked={selected}
                    onChange={() =>
                      onChange(
                        selected
                          ? (value as unknown[]).filter(
                              (item) => String(item) !== String(option.value),
                            )
                          : [
                              ...(Array.isArray(value) ? value : []),
                              option.value,
                            ],
                      )
                    }
                  />
                }
              />
            )
          })}
        </div>
      </div>
    )
  else if (control === 'textarea')
    input = (
      <textarea
        className="mod-input"
        value={String(value ?? '')}
        minLength={definition.minimumLength}
        maxLength={definition.maximumLength}
        onChange={(event) => onChange(event.target.value)}
      />
    )
  else if (control === 'color')
    input = (
      <input
        className="mod-input mod-color"
        type="color"
        value={String(value ?? definition.default)}
        onChange={(event) => onChange(event.target.value)}
      />
    )
  else if (control === 'keybind')
    input = (
      <input
        className="mod-input keybind-input"
        value={String(value ?? '')}
        readOnly
        aria-label={definition.label}
        onKeyDown={(event) => {
          event.preventDefault()
          onChange(event.key)
        }}
        onClick={(event) => event.currentTarget.focus()}
      />
    )
  else
    input = (
      <input
        className="mod-input"
        type={numeric ? 'number' : 'text'}
        value={String(value ?? '')}
        min={definition.minimum}
        max={definition.maximum}
        step={definition.step}
        minLength={definition.minimumLength}
        maxLength={definition.maximumLength}
        onChange={(event) =>
          onChange(numeric ? Number(event.target.value) : event.target.value)
        }
      />
    )
  return (
    <SettingsRow
      label={definition.label}
      detail={definition.description}
      control={<div className="mod-setting-control">{input}</div>}
      rowClass={
        control === 'radio' || control === 'segmented' ? 'expanded' : ''
      }
    />
  )
}

export function optionValue(definition: SettingDefinition, value: string) {
  return (
    definition.options?.find((option) => String(option.value) === value)
      ?.value ?? value
  )
}
