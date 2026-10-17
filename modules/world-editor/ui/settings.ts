import type { TranslationKey } from '../../ui-resources/i18n'
export const worldEditorSettingsFields: Array<
  [string, string, TranslationKey, string[]?]
> = [
  ['worldEditor', 'enabled', 'settings.modules.field.active'],
  ['worldEditor', 'toggleKey', 'settings.modules.field.hotkey'],
  ['worldEditor', 'refreshMilliseconds', 'settings.modules.field.refresh'],
]
