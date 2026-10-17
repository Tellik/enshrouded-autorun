import type { TranslationKey } from '../../ui-resources/i18n'
export const debugConsoleSettingsFields: Array<
  [string, string, TranslationKey, string[]?]
> = [
  ['debugConsole', 'enabled', 'settings.modules.field.active'],
  [
    'debugConsole',
    'defaultSource',
    'settings.modules.field.source',
    ['game', 'loader'],
  ],
  [
    'debugConsole',
    'levelFilter',
    'settings.modules.field.filter',
    ['ALL', 'TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR'],
  ],
  ['debugConsole', 'autoScroll', 'settings.modules.field.autoScroll'],
  ['debugConsole', 'toggleKey', 'settings.modules.field.hotkey'],
  ['debugConsole', 'refreshMilliseconds', 'settings.modules.field.refresh'],
  ['debugConsole', 'tailBytes', 'settings.modules.field.tail'],
]
