import { useI18n } from '../../ui-resources/i18n'
import {
  SettingsRow,
  SettingSwitch,
  SettingsNumber,
} from '../../ui-resources/edit-setting'

export function SystemUpdatePreferences({
  system,
  changeSystem,
}: {
  system: Record<string, any>
  changeSystem(key: string, value: unknown): void
}) {
  const { t } = useI18n()
  return (
    <section className="settings-module">
      <header>
        <div>
          <h3>{t('settings.modules.updates.title')}</h3>
          <p className="hint">{t('settings.modules.updates.description')}</p>
        </div>
      </header>
      <div className="settings-options">
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
    </section>
  )
}
