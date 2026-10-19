import { useI18n } from '../../ui-resources/i18n'
import type { ReactNode } from 'react'
import {
  ModuleEnabledRow,
  SettingsRow,
  SettingSwitch,
  SettingsNumber,
  Toggle,
} from '../../ui-resources/edit-setting'
import { Card } from '../../ui-resources/show-card'
import type { UpdaterCommand } from './update-types'
import { UpdaterAvailability } from './show-updater-availability'

export function UpdatePreferences({
  system,
  automatic,
  enabled,
  updaterAvailable,
  onCommand,
  windowVisible,
  changeEnabled,
  changeWindowVisibility,
  changeSystem,
  changeAutomatic,
  recycleBin,
  changeRecycleBin,
  geometryControls,
}: {
  system: Record<string, any>
  automatic: Record<string, any>
  enabled: boolean
  updaterAvailable: boolean
  onCommand: UpdaterCommand
  windowVisible: boolean
  changeEnabled(value: boolean): void
  changeWindowVisibility(value: boolean): void
  changeSystem(key: string, value: unknown): void
  changeAutomatic(key: string, value: unknown): void
  recycleBin: Record<string, any>
  changeRecycleBin(key: string, value: unknown): void
  geometryControls: ReactNode
}) {
  const { t } = useI18n()
  return (
    <Card title={t('settings.modules.updates.title')}>
      <p className="hint settings-module-description">
        {t('settings.modules.updates.description')}
      </p>
      <UpdaterAvailability
        updaterAvailable={updaterAvailable}
        enabled={enabled}
        onCommand={onCommand}
      />
      <div className="settings-options">
        <div className="settings-window-control">
          <Toggle
            label={t('settings.modules.windowVisible')}
            checked={windowVisible}
            disabled={!enabled || !updaterAvailable}
            onChange={changeWindowVisibility}
          />
          <span className={`state-pill ${windowVisible ? 'ok' : 'neutral'}`}>
            {windowVisible
              ? t('settings.modules.windowOpen')
              : t('settings.modules.windowClosed')}
          </span>
        </div>
        <ModuleEnabledRow
          label={t('settings.modules.field.active')}
          enabled={enabled}
          onChange={changeEnabled}
          disabled={!updaterAvailable}
        />
        <SettingsRow
          label={t('settings.modules.updates.auto')}
          control={
            <SettingSwitch
              label={t('settings.modules.updates.auto')}
              checked={automatic.enabled !== false}
              disabled={!enabled}
              onChange={(value) => changeAutomatic('enabled', value)}
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.updates.autoQueue')}
          control={
            <SettingSwitch
              label={t('settings.modules.updates.autoQueue')}
              checked={automatic.queueUpdates !== false}
              disabled={!enabled || automatic.enabled === false}
              onChange={(value) => changeAutomatic('queueUpdates', value)}
            />
          }
        />
        <p className="hint settings-module-hint">
          {t('settings.modules.updates.autoQueueHint')}
        </p>
        <SettingsRow
          label={t('settings.modules.updates.interval')}
          control={
            <SettingsNumber
              value={automatic.checkMinutes ?? 60}
              minimum={5}
              maximum={1440}
              disabled={!enabled || automatic.enabled === false}
              onCommit={(value) => changeAutomatic('checkMinutes', value)}
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.updates.prereleases')}
          control={
            <SettingSwitch
              label={t('settings.modules.updates.prereleases')}
              checked={Boolean(system.includePrereleases)}
              disabled={!enabled}
              onChange={(value) => changeSystem('includePrereleases', value)}
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.updates.recycleBinCleanup')}
          control={
            <SettingSwitch
              label={t('settings.modules.updates.recycleBinCleanup')}
              checked={recycleBin.automaticCleanup !== false}
              onChange={(value) => changeRecycleBin('automaticCleanup', value)}
            />
          }
        />
        <SettingsRow
          label={t('settings.modules.updates.recycleBinRetention')}
          control={
            <SettingsNumber
              value={recycleBin.retentionDays ?? 30}
              minimum={1}
              maximum={3650}
              onCommit={(value) => changeRecycleBin('retentionDays', value)}
            />
          }
        />
        <p className="hint settings-module-hint">
          {t('settings.modules.updates.recycleBinHint')}
        </p>
        {geometryControls}
      </div>
    </Card>
  )
}
