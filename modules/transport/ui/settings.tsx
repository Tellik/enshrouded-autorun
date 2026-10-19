import type { ModulePreferences } from '../../loader-config/ui/loader-settings'
import { useI18n } from '../../ui-resources/i18n'

import { Card } from '../../ui-resources/show-card'
import {
  ModuleEnabledRow,
  SettingsRow,
  SettingSwitch,
} from '../../ui-resources/edit-setting'

export function NetworkSettings({
  prefs,
  enabled,
  change,
}: {
  prefs: ModulePreferences
  enabled: boolean
  change(module: string, key: string, value: unknown): void
}) {
  const { t } = useI18n()
  return (
    <Card title={t('settings.modules.network.title')}>
      <p className="hint settings-module-description">
        {t('settings.modules.network.description')}
      </p>
      <div className="settings-options">
        <ModuleEnabledRow
          label={t('settings.modules.field.active')}
          enabled={enabled}
          onChange={(value) => change('network', 'enabled', value)}
        />
        {!enabled && (
          <p className="builder-notice warning" role="status">
            {t('settings.moduleUnavailable')}
          </p>
        )}
        <SettingsRow
          label={t('settings.modules.network.serverSteamId')}
          control={
            <input
              className="settings-number"
              type="text"
              inputMode="numeric"
              maxLength={20}
              value={prefs.network?.serverSteamId || ''}
              onChange={(event) =>
                change(
                  'network',
                  'serverSteamId',
                  event.target.value.replace(/\D/g, ''),
                )
              }
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.network.allowedClients')}
          control={
            <textarea
              className="settings-module-textarea"
              rows={3}
              maxLength={4096}
              value={prefs.network?.allowedClientSteamIds || ''}
              onChange={(event) =>
                change('network', 'allowedClientSteamIds', event.target.value)
              }
            />
          }
        />
      </div>
      <p className="hint">{t('settings.modules.network.help')}</p>
    </Card>
  )
}
