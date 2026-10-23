import { postRequest } from '../../transport/ui/send-host-command'
import { Card } from '../../ui-resources/show-card'
import { SettingsRow } from '../../ui-resources/edit-setting'
import { useI18n } from '../../ui-resources/i18n'
import type { TranslationKey } from '../../ui-resources/i18n'
import type { Snapshot } from '../../modloader/ui/ui-snapshot'
import { settingIsEnabled } from '../../modloader/ui/edit-mod-setting'
import { KeybindInput, keybindText } from './edit-keybind'
import { WikiHelp } from '../../wiki/ui/show-wiki-help'

type HotkeyRow = {
  id: string
  owner: string
  label: string
  description?: string
  binding: string
  enabled: boolean
  save(binding: string): void
  reset?(): void
}

function hotkeyRows(data: Snapshot): HotkeyRow[] {
  const client = data.mode !== 'SERVER'
  const modules = Object.entries(data.settings.modulePreferences || {})
    .filter(([, preferences]) => Object.prototype.hasOwnProperty.call(preferences, 'toggleKey'))
    .map(([module, preferences]): HotkeyRow => ({
      id: `module:${module}`,
      owner: module,
      label: '',
      binding: keybindText(preferences.toggleKey),
      enabled: client && preferences.enabled !== false,
      save: (binding) => postRequest('save-setting', {
        scope: 'module', module, key: 'toggleKey', value: binding,
      }),
      reset: () => postRequest('save-setting', {
        scope: 'module', module, key: 'toggleKey', value: null,
      }),
    }))
  const mods = data.mods.flatMap((mod) => {
    const values = Object.fromEntries(mod.settings.map((definition) => [
      definition.key, mod.settingValues[definition.key] ?? definition.default,
    ]))
    return mod.settings
    .filter((definition) => definition.control === 'keybind')
    .map((definition): HotkeyRow => ({
      id: `mod:${mod.id}:${definition.key}`,
      owner: mod.name,
      label: definition.label,
      description: definition.description,
      binding: keybindText(mod.settingValues[definition.key] ?? definition.default),
      enabled: client && mod.enabled && mod.target !== 'server' &&
        (mod.id !== 'world-editor' || data.settings.modulePreferences?.worldEditor?.enabled !== false) &&
        settingIsEnabled(definition, mod.settings, values),
      save: (binding) => postRequest('save-mod-settings', {
        modId: mod.id, values: { [definition.key]: binding }, revision: mod.revision,
      }),
    }))
  })
  return [...modules, ...mods]
}

function conflictsFor(rows: HotkeyRow[], row: HotkeyRow): HotkeyRow[] {
  const alternatives = row.binding.split('|').filter(Boolean)
  return row.enabled && row.binding ? rows.filter((other) =>
    other.id !== row.id && other.enabled &&
      other.binding.split('|').some((binding) => alternatives.includes(binding)),
  ) : []
}

export function hasHotkeyConflicts(data: Snapshot): boolean {
  const rows = hotkeyRows(data)
  return rows.some((row) => conflictsFor(rows, row).length > 0)
}

export function Hotkeys({ data }: { data: Snapshot }) {
  const { t } = useI18n()
  const rows = hotkeyRows(data)
  const ownerLabel = (row: HotkeyRow) => row.id.startsWith('module:')
    ? t(`settings.modules.${row.owner}.title` as TranslationKey) : row.owner
  const actionLabel = (row: HotkeyRow) => row.label || t('hotkeys.toggleWindow')
  const label = (row: HotkeyRow) => `${ownerLabel(row)}: ${actionLabel(row)}`
  return (
    <div className="settings-module-list compatibility-sections">
      <WikiHelp articleId="hotkeys" />
      <Card title={t('compatibility.tab.hotkeys')}>
        <p className="hint settings-module-description">{t('hotkeys.description')}</p>
        <p className="hint settings-module-description">{t('hotkeys.captureHint')}</p>
        {hasHotkeyConflicts(data) && <p className="builder-notice warning" role="status">
          {t('hotkeys.conflicts')}
        </p>}
        <div className="settings-options">
          {rows.map((row) => {
            const conflicts = conflictsFor(rows, row)
            const detail = [
              row.description,
              !row.enabled ? t('hotkeys.inactive') : '',
              conflicts.length ? t('hotkeys.conflict', { actions: conflicts.map(label).join(', ') }) : '',
            ].filter(Boolean).join(' ')
            return <SettingsRow
              key={row.id}
              label={label(row)}
              detail={detail || undefined}
              rowClass={conflicts.length ? 'warning' : ''}
              control={<KeybindInput value={row.binding} label={label(row)} onChange={row.save} onReset={row.reset} />}
            />
          })}
        </div>
        {!rows.length && <p className="hint">{t('hotkeys.empty')}</p>}
      </Card>
    </div>
  )
}
