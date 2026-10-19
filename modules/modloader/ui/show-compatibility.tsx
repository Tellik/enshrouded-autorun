import { postRequest } from '../../transport/ui/send-host-command'
import { TranslationKey, useI18n } from '../../ui-resources/i18n'
import { Snapshot } from './ui-snapshot'
import type { NewsItem } from '../../notifications/ui/notification-types'
import { EcosystemBadge } from '../../ui-resources/show-ecosystem-badge'
import { Icon } from '../../ui-resources/show-icon'
import { PageHeader } from '../../ui-resources/show-page-header'

export type CompatibilityStatus = 'ok' | 'warning' | 'error' | 'neutral'

export type CompatibilityRowData = {
  id: string
  title: string
  detail: string
  status: CompatibilityStatus
  technicalDetail?: string
  modId?: string
  ecosystem?: 'EML' | 'SF'
  action?: { label: string; run(): void }
}

export function Compatibility({
  data,
  news,
  openNews,
  selectMod,
  openLog,
}: {
  data: Snapshot
  news: NewsItem[]
  openNews(): void
  selectMod(id: string): void
  openLog(): void
}) {
  const { t } = useI18n()
  const checks = data.configuration?.checks || []
  const check = (id: string) => checks.find((item) => item.id === id)
  const statusOf = (value?: { state: string }): CompatibilityStatus =>
    value?.state === 'ok'
      ? 'ok'
      : value?.state === 'warning'
        ? 'warning'
        : 'neutral'
  const gameCheck = check('game'),
    runtimeCheck = check('runtime'),
    apiCheck = check('api'),
    parserCheck = check('parser'),
    modsCheck = check('mods')
  const gameRows: CompatibilityRowData[] = [
    {
      id: 'game',
      title: t('compatibility.row.game'),
      status: statusOf(gameCheck),
      detail: t(
        gameCheck?.state === 'ok'
          ? 'compatibility.detail.game.ok'
          : gameCheck?.state === 'warning'
            ? 'compatibility.detail.game.warning'
            : 'compatibility.detail.game.pending',
        { version: data.gameVersion },
      ),
      technicalDetail: gameCheck?.detail,
    },
    {
      id: 'runtime',
      title: t('compatibility.row.runtime'),
      status: statusOf(runtimeCheck),
      detail: t(
        runtimeCheck?.phase === 'waiting-world'
          ? 'compatibility.detail.runtime.waitingWorld'
          : runtimeCheck?.state === 'ok'
            ? 'compatibility.detail.runtime.ok'
            : runtimeCheck?.state === 'warning'
              ? 'compatibility.detail.runtime.warning'
              : data.mode === 'DESKTOP'
                ? 'compatibility.detail.runtime.desktop'
                : 'compatibility.detail.runtime.pending',
      ),
      technicalDetail: runtimeCheck?.detail,
      action:
        runtimeCheck?.state === 'warning'
          ? { label: t('compatibility.action.openLog'), run: openLog }
          : undefined,
    },
  ]
  const assets = data.configuration?.assets
  if (assets) {
    const assetStatus: CompatibilityStatus =
      assets.state === 'failed'
        ? 'error'
        : assets.state === 'applied' || assets.state === 'not-required'
          ? 'ok'
          : assets.state === 'prepare-required' || assets.state === 'partial'
            ? 'warning'
            : 'neutral'
    gameRows.push({
      id: 'assets',
      title: t('compatibility.row.assets'),
      status: assetStatus,
      detail: t(
        assets.state === 'applied'
          ? 'compatibility.detail.assets.applied'
          : assets.state === 'partial'
            ? 'compatibility.detail.assets.partial'
            : assets.state === 'not-required'
              ? 'compatibility.detail.assets.notRequired'
              : assets.state === 'prepare-required'
                ? 'compatibility.detail.assets.required'
                : assets.state === 'failed'
                  ? 'compatibility.detail.assets.failed'
                  : 'compatibility.detail.assets.unknown',
      ),
      technicalDetail: assets.detail,
    })
  }
  for (const [index, error] of (data.configuration?.errors || []).entries())
    gameRows.push({
      id: `config-error-${index}`,
      title: t('compatibility.row.configuration'),
      status: 'error',
      detail: t('compatibility.detail.configuration.error'),
      technicalDetail: error,
      action: { label: t('compatibility.action.openLog'), run: openLog },
    })
  const apiRows: CompatibilityRowData[] = [
    {
      id: 'api',
      title: t('compatibility.row.api'),
      status: statusOf(apiCheck),
      detail: t(
        apiCheck?.state === 'ok'
          ? 'compatibility.detail.api.ok'
          : apiCheck?.state === 'warning'
            ? 'compatibility.detail.api.warning'
            : 'compatibility.detail.api.pending',
      ),
      technicalDetail: apiCheck?.detail,
      action:
        apiCheck?.state === 'warning'
          ? { label: t('compatibility.action.openLog'), run: openLog }
          : undefined,
    },
  ]
  const parserRows: CompatibilityRowData[] = [
    {
      id: 'parser',
      title: t('compatibility.row.parser'),
      status: statusOf(parserCheck),
      detail: t(
        parserCheck?.state === 'ok'
          ? 'compatibility.detail.parser.ok'
          : parserCheck?.state === 'warning'
            ? 'compatibility.detail.parser.warning'
            : 'compatibility.detail.parser.pending',
      ),
      technicalDetail: parserCheck?.detail,
      action:
        parserCheck?.state === 'warning'
          ? { label: t('common.refresh'), run: () => postRequest('refresh') }
          : undefined,
    },
  ]
  const modRows: CompatibilityRowData[] = [
    {
      id: 'mods-summary',
      title: t('compatibility.row.modCheck'),
      status: statusOf(modsCheck),
      detail: t(
        modsCheck?.state === 'warning'
          ? 'compatibility.detail.mods.warning'
          : 'compatibility.detail.mods.ok',
        { count: data.mods.length },
      ),
      technicalDetail: modsCheck?.detail,
    },
  ]
  const knownStates = data.configuration?.modStates || {}
  const modStateLabels: Record<string, string> = {
    failed: 'compatibility.modState.failed',
    blocked: 'compatibility.modState.blocked',
    'restart-required': 'compatibility.modState.restart',
    'not-running': 'compatibility.modState.notRunning',
    'no-target': 'compatibility.modState.noTarget',
    waiting: 'compatibility.modState.waiting',
    'waiting-world': 'compatibility.modState.waitingWorld',
  }
  for (const mod of data.mods) {
    const state = knownStates[mod.id]
    const labelKey = state && modStateLabels[state.state]
    const failed = state?.state === 'failed'
    modRows.push({
      id: `mod-${mod.id}-${state?.state || 'loaded'}`,
      title: mod.name,
      status: labelKey
        ? failed
          ? 'error'
          : state?.state === 'waiting' || state?.state === 'waiting-world'
            ? 'neutral'
            : 'warning'
        : 'ok',
      detail: labelKey
        ? t(labelKey as TranslationKey)
        : t(
            mod.ecosystem === 'SF'
              ? 'compatibility.mod.sf.loaded'
              : 'compatibility.mod.eml.loaded',
          ),
      technicalDetail: state?.detail,
      modId: mod.id,
      ecosystem: mod.ecosystem,
      action: {
        label: t('compatibility.action.openMod'),
        run: () => selectMod(mod.id),
      },
    })
  }
  const sfMods = data.mods.filter((mod) => mod.ecosystem === 'SF')
  if (sfMods.length) {
    modRows.push({
      id: 'sf-extension-check',
      title: t('compatibility.row.sfExtension'),
      status: 'ok',
      detail: t('compatibility.detail.sfExtension.ok', {
        count: sfMods.length,
      }),
    })
    modRows.push({
      id: 'sf-api-check',
      title: t('compatibility.row.sfApi'),
      status: statusOf(apiCheck),
      detail: t(
        apiCheck?.state === 'ok'
          ? 'compatibility.detail.sfApi.ok'
          : apiCheck?.state === 'warning'
            ? 'compatibility.detail.sfApi.warning'
            : 'compatibility.detail.sfApi.pending',
      ),
      technicalDetail: apiCheck?.detail,
      action:
        apiCheck?.state === 'warning'
          ? { label: t('compatibility.action.openLog'), run: openLog }
          : undefined,
    })
  }
  const allRows = [...gameRows, ...apiRows, ...parserRows, ...modRows]
  const errorCount = allRows.filter((row) => row.status === 'error').length
  const warningCount = allRows.filter((row) => row.status === 'warning').length
  const pendingCount = allRows.filter((row) => row.status === 'neutral').length
  const overall: CompatibilityStatus = errorCount
    ? 'error'
    : warningCount
      ? 'warning'
      : pendingCount
        ? 'neutral'
        : 'ok'
  const headlineKey =
    overall === 'error'
      ? 'compatibility.summary.error'
      : overall === 'warning'
        ? 'compatibility.summary.warning'
        : overall === 'neutral'
          ? 'compatibility.summary.pending'
          : 'compatibility.summary.ok'
  const unreadAlerts = news.filter(
    (item) =>
      (item.level === 'warn' || item.level === 'error') &&
      !data.readNoticeIds.includes(item.id),
  ).length
  const groups = [
    {
      id: 'game',
      title: t('compatibility.section.game'),
      description: t('compatibility.section.game.description'),
      rows: gameRows,
    },
    {
      id: 'api',
      title: t('compatibility.section.api'),
      description: t('compatibility.section.api.description'),
      rows: apiRows,
    },
    {
      id: 'parser',
      title: t('compatibility.section.parser'),
      description: t('compatibility.section.parser.description'),
      rows: parserRows,
    },
    {
      id: 'mods',
      title: t('compatibility.section.mods'),
      description: t('compatibility.section.mods.description'),
      rows: modRows,
    },
  ]
  return (
    <>
      <PageHeader
        eyebrow={t('nav.modloader')}
        title={t('compatibility.title')}
        subtitle={t('compatibility.subtitle')}
        actions={
          <>
            {unreadAlerts > 0 && (
              <button className="button secondary" onClick={openNews}>
                {t('compatibility.action.openNews', { count: unreadAlerts })}
              </button>
            )}
            <button className="button" onClick={() => postRequest('refresh')}>
              <Icon name="refresh" />
              {t('compatibility.action.refresh')}
            </button>
          </>
        }
      />
      <section className={`compatibility-summary ${overall}`}>
        <span className={`compatibility-summary-icon ${overall}`}>
          <Icon
            name={
              overall === 'ok'
                ? 'check'
                : overall === 'neutral'
                  ? 'compatibility'
                  : 'warning'
            }
          />
        </span>
        <div className="compatibility-summary-copy">
          <strong>{t(`${headlineKey}.title` as TranslationKey)}</strong>
          <p>
            {t(`${headlineKey}.message` as TranslationKey, {
              errors: errorCount,
              warnings: warningCount,
              pending: pendingCount,
              mods: data.mods.length,
            })}
          </p>
        </div>
        <span className="compatibility-summary-count">
          {t('compatibility.summary.counts', {
            checks: allRows.length,
            mods: data.mods.length,
            attention: errorCount + warningCount,
          })}
        </span>
      </section>
      <div className="settings-module-list compatibility-sections">
        {groups.map((group) => {
          const state = group.rows.some((row) => row.status === 'error')
            ? 'error'
            : group.rows.some((row) => row.status === 'warning')
              ? 'warning'
              : group.rows.some((row) => row.status === 'neutral')
                ? 'neutral'
                : 'ok'
          return (
            <section
              className="settings-module compatibility-section"
              key={group.id}
            >
              <header>
                <div>
                  <h3>{group.title}</h3>
                  <p className="hint">{group.description}</p>
                </div>
                <CompatibilityStatusBadge status={state} />
              </header>
              <div className="settings-options">
                {group.rows.map((row) => (
                  <CompatibilityRow key={row.id} row={row} />
                ))}
              </div>
            </section>
          )
        })}
      </div>
    </>
  )
}

export function CompatibilityStatusBadge({
  status,
}: {
  status: CompatibilityStatus
}) {
  const { t } = useI18n()
  return (
    <span className={`compatibility-status ${status}`}>
      <i />
      {t(`compatibility.status.${status}` as TranslationKey)}
    </span>
  )
}

export function CompatibilityRow({ row }: { row: CompatibilityRowData }) {
  const { t } = useI18n()
  return (
    <div className={`settings-option compatibility-row ${row.status}`}>
      <span className="compatibility-row-copy">
        <strong>
          {row.title}
          <EcosystemBadge ecosystem={row.ecosystem} />
        </strong>
        <small>{row.detail}</small>
        {row.technicalDetail && (
          <details>
            <summary>{t('compatibility.details')}</summary>
            <p>{row.technicalDetail}</p>
          </details>
        )}
      </span>
      <div className="compatibility-row-control">
        <CompatibilityStatusBadge status={row.status} />
        {row.action && (
          <button className="button ghost" onClick={row.action.run}>
            {row.action.label}
          </button>
        )}
      </div>
    </div>
  )
}
