import { createRoot } from 'react-dom/client'
import { I18nProvider } from '../../ui-resources/i18n'
import { UpdaterApp } from './show-updater-app'
import '../../modloader/ui/styles.css'

createRoot(document.getElementById('root')!).render(
  <I18nProvider>
    <UpdaterApp />
  </I18nProvider>,
)
