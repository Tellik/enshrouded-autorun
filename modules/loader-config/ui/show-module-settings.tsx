import type { Settings } from './loader-settings'
import { postRequest } from '../../transport/ui/send-host-command'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { Snapshot } from '../../modloader/ui/ui-snapshot'
import { Card } from '../../ui-resources/show-card'
import {
  SettingSwitch,
  SettingsNumber,
  SettingsRow,
  Toggle,
} from '../../ui-resources/edit-setting'
import {
  DiagnosticsControls,
  diagnosticSettingsFields,
} from '../../runtime-diagnostics/ui/show-diagnostics'
import { debugConsoleSettingsFields } from '../../debug-console/ui/settings'
import { worldEditorSettingsFields } from '../../world-editor/ui/settings'
import { NetworkSettings } from '../../transport/ui/settings'
import { SystemUpdatePreferences } from '../../updater/ui/edit-update-preferences'

export function ModulePreferencesEditor({
  data,
  settings,
  setSettings,
}: {
  data: Snapshot
  settings: Settings
  setSettings(v: Settings): void
}) {
  const { t } = useI18n()
  const prefs = settings.modulePreferences || {}
  const system = prefs.updates?.system || {
    enabled: true,
    checkMinutes: 60,
    channel: 'stable',
    includePrereleases: false,
  }
  const saveModule = (module: string, key: string, value: unknown) =>
    postRequest('save-setting', { scope: 'module', module, key, value })
  const changeSystem = (key: string, value: unknown) => {
    setSettings({
      ...settings,
      modulePreferences: {
        ...prefs,
        updates: { ...prefs.updates, system: { ...system, [key]: value } },
      },
    })
    saveModule('updates.system', key, value)
  }
  const change = (module: string, key: string, value: unknown) => {
    const nextModule =
      key === 'window.position'
        ? {
            ...prefs[module],
            window: { ...prefs[module]?.window, position: value },
          }
        : { ...prefs[module], [key]: value }
    setSettings({
      ...settings,
      modulePreferences: { ...prefs, [module]: nextModule },
    })
    saveModule(module, key, value)
  }
  const fields: Array<[string, string, TranslationKey, string[]?]> = [
    ...diagnosticSettingsFields,
    ...debugConsoleSettingsFields,
    ...worldEditorSettingsFields,
    ['modloaderUi', 'enabled', 'settings.modules.field.active'],
    ['modloaderUi', 'toggleKey', 'settings.modules.field.hotkey'],
    ['modloaderUi', 'refreshMilliseconds', 'settings.modules.field.refresh'],
  ]
  const controlVisibility = (
    module: 'debugConsole' | 'modloaderUi' | 'worldEditor',
    visible: boolean,
  ) => postRequest('set-window-visibility', { module, visible })
  const settingEnabled = (module: string, key: string) =>
    Boolean(
      prefs[module]?.[key] ?? (module === 'worldEditor' && key === 'enabled'),
    )
  const ranges: Record<string, [number, number]> = {
    intervalMilliseconds: [500, 60000],
    maximumDurationSeconds: [1, 3600],
    slowCallbackMilliseconds: [0.1, 60000],
    toggleKey: [1, 255],
    refreshMilliseconds: [100, 10000],
    tailBytes: [1024, 1048576],
  }
  return (
    <>
      <NetworkSettings prefs={prefs} change={change} />
      <Card title={t('settings.modules')}>
        <div className="settings-module-list">
          {(
            [
              'debugConsole',
              'worldEditor',
              'modloaderUi',
              'runtimeDiagnostics',
            ] as const
          ).map((module) => (
            <section
              className="settings-module"
              id={`settings-module-${module}`}
              key={module}
            >
              <header>
                <div>
                  <h3>{t(`settings.modules.${module}.title`)}</h3>
                  <p className="hint">
                    {t(`settings.modules.${module}.description`)}
                  </p>
                </div>
              </header>
              {(module === 'debugConsole' ||
                module === 'modloaderUi' ||
                module === 'worldEditor') && (
                <div className="settings-window-control">
                  <Toggle
                    label={t('settings.modules.windowVisible')}
                    checked={Boolean(data.windows?.[module]?.visible)}
                    onChange={(visible) => controlVisibility(module, visible)}
                  />
                  <span
                    className={`state-pill ${data.windows?.[module]?.visible ? 'ok' : 'neutral'}`}
                  >
                    {data.windows?.[module]?.visible
                      ? t('settings.modules.windowOpen')
                      : t('settings.modules.windowClosed')}
                  </span>
                </div>
              )}
              <div className="settings-options">
                {fields
                  .filter((field) => field[0] === module)
                  .map(([, key, label, options]) => (
                    <SettingsRow
                      key={key}
                      label={t(label)}
                      control={
                        options ? (
                          <select
                            value={prefs[module]?.[key] ?? options[0]}
                            onChange={(e) =>
                              change(module, key, e.target.value)
                            }
                          >
                            {options.map((option) => (
                              <option key={option}>{option}</option>
                            ))}
                          </select>
                        ) : typeof prefs[module]?.[key] === 'boolean' ? (
                          <SettingSwitch
                            label={t(label)}
                            checked={settingEnabled(module, key)}
                            onChange={(value) => change(module, key, value)}
                          />
                        ) : (
                          <SettingsNumber
                            value={prefs[module]?.[key] ?? 0}
                            minimum={ranges[key]?.[0]}
                            maximum={
                              module === 'worldEditor' &&
                              key === 'refreshMilliseconds'
                                ? 5000
                                : ranges[key]?.[1]
                            }
                            onCommit={(value) => change(module, key, value)}
                          />
                        )
                      }
                    />
                  ))}
              </div>
              {module !== 'runtimeDiagnostics' && module !== 'worldEditor' && (
                <button
                  className="button ghost settings-module-action"
                  onClick={() => change(module, 'window.position', null)}
                >
                  {t('settings.modules.resetPosition')}
                </button>
              )}
              {module === 'runtimeDiagnostics' && (
                <DiagnosticsControls
                  data={data}
                  prefs={prefs}
                  module={module}
                  change={change}
                />
              )}
            </section>
          ))}
          <SystemUpdatePreferences
            system={system}
            changeSystem={changeSystem}
          />
        </div>
      </Card>
    </>
  )
}
