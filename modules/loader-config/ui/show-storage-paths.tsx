import { postRequest } from '../../transport/ui/send-host-command'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import type { Snapshot } from '../../modloader/ui/ui-snapshot'
import { Card } from '../../ui-resources/show-card'

export function StoragePathsEditor({ data }: { data: Snapshot }) {
  const { t } = useI18n()
  const paths = data.settings.paths || []
  const reset = (id: string) =>
    postRequest('save-setting', { scope: 'paths', key: id, value: null })
  return (
    <Card title={t('settings.paths.title')}>
      <p className="hint">{t('settings.paths.description')}</p>
      <div className="settings-module-list">
        {paths.map((item) => (
          <section className="settings-module" key={item.id}>
            <header>
              <div>
                <h3>{t(`settings.paths.${item.id}` as TranslationKey)}</h3>
                <p className="hint">
                  {item.path === item.defaultPath
                    ? t('settings.paths.default')
                    : t('settings.paths.custom')}
                </p>
                {item.id === 'mods' && (
                  <p className="hint">{t('settings.paths.modsWarning')}</p>
                )}
              </div>
            </header>
            <div className="storage-path-row">
              <input
                aria-label={t(`settings.paths.${item.id}` as TranslationKey)}
                value={item.path}
                readOnly
                title={item.path}
              />
              <button
                className="button secondary"
                onClick={() => postRequest('browse-path', { key: item.id })}
              >
                {t('settings.paths.browse')}
              </button>
              <button
                className="button ghost"
                onClick={() => postRequest('open-path', { key: item.id })}
              >
                {t('settings.paths.open')}
              </button>
              {item.path !== item.defaultPath && (
                <button className="button ghost" onClick={() => reset(item.id)}>
                  {t('settings.paths.reset')}
                </button>
              )}
            </div>
          </section>
        ))}
      </div>
    </Card>
  )
}
