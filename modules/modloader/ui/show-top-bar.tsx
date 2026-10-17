import { postRequest } from '../../transport/ui/send-host-command'
import { useI18n } from '../../ui-resources/i18n'
import { Snapshot } from './ui-snapshot'
import { Brand } from '../../ui-resources/show-brand'
import { HeaderLink } from '../../ui-resources/show-header-link'
import { LanguageMenu } from '../../ui-resources/choose-language'
import { ModeBadge } from '../../ui-resources/show-mode-badge'

export function TopBar({
  data,
  unread,
  severity,
  crowded,
  title,
  onChirper,
}: {
  data: Snapshot
  unread: number
  severity: string
  crowded: boolean
  title: string
  onChirper(): void
}) {
  const { t } = useI18n()
  const external = (url: string) => postRequest('open-url', { url })
  return (
    <header className="topbar" data-drag-region>
      <Brand />
      <nav className="top-actions">
        <HeaderLink
          label={t('header.github')}
          icon="github"
          tone="github"
          onClick={() => external('https://github.com/bonsaibauer/shroudforge')}
        />
        <HeaderLink
          label={t('header.reportIssue')}
          icon="issue"
          tone="danger"
          onClick={() =>
            external(
              'https://github.com/bonsaibauer/shroudforge/issues/new/choose',
            )
          }
        />
        <HeaderLink
          label={t('header.discord')}
          icon="discord"
          tone="discord"
          onClick={() => external('https://discord.com/invite/jH2gNJYnt2')}
        />
        <HeaderLink
          label={t('header.support')}
          icon="coffee"
          tone="support"
          onClick={() => external('https://buymeacoffee.com/bonsaibauer')}
        />
        <HeaderLink
          label={t('header.chirper')}
          icon="chirper"
          tone="chirper"
          badge={unread || undefined}
          severity={severity}
          crowded={crowded}
          title={title}
          onClick={onChirper}
        />
      </nav>
      <LanguageMenu />
      <ModeBadge mode={data.mode} connected={data.connected} />
      <button
        className="window-close"
        onClick={() => postRequest('hide')}
        aria-label={t('common.close')}
      >
        ×
      </button>
    </header>
  )
}
