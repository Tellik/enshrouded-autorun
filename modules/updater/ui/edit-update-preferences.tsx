import { useI18n } from '../../ui-resources/i18n'
import {
  ModuleEnabledRow,
  SettingsRow,
  SettingSwitch,
  SettingsNumber,
} from '../../ui-resources/edit-setting'
import { Card } from '../../ui-resources/show-card'

export function SystemUpdatePreferences({
  system,
  enabled,
  changeEnabled,
  changeSystem,
}: {
  system: Record<string, any>
  enabled: boolean
  changeEnabled(value: boolean): void
  changeSystem(key: string, value: unknown): void
}) {
  const { t } = useI18n()
  return (
    <Card title={t('settings.modules.updates.title')}>
      <p className="hint settings-module-description">
        {t('settings.modules.updates.description')}
      </p>
      <div className="settings-options">
        <ModuleEnabledRow
          label={t('settings.modules.field.active')}
          enabled={enabled}
          onChange={changeEnabled}
        />
        <SettingsRow
          label={t('settings.modules.updates.auto')}
          control={
            <SettingSwitch
              label={t('settings.modules.updates.auto')}
              checked={system.enabled}
              onChange={(value) => changeSystem('enabled', value)}
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.updates.prereleases')}
          control={
            <SettingSwitch
              label={t('settings.modules.updates.prereleases')}
              checked={Boolean(system.includePrereleases)}
              onChange={(value) => changeSystem('includePrereleases', value)}
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.updates.interval')}
          control={
            <SettingsNumber
              value={system.checkMinutes}
              minimum={5}
              maximum={1440}
              onCommit={(value) => changeSystem('checkMinutes', value)}
            />
          }
        />
      </div>
    </Card>
  )
}
