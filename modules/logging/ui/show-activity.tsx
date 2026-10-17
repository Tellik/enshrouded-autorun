import { writeClipboardText } from '../../ui-resources/copy-to-clipboard'
import type { Activity } from './log-entry'
import { postRequest } from '../../transport/ui/send-host-command'
import { useEffect, useRef, useState } from 'react'
import { useI18n } from '../../ui-resources/i18n'

import { Card } from '../../ui-resources/show-card'
import { EcosystemBadge } from '../../ui-resources/show-ecosystem-badge'
import { Empty } from '../../ui-resources/show-empty-state'
import { Icon } from '../../ui-resources/show-icon'
import { PageHeader } from '../../ui-resources/show-page-header'
import { Segmented } from '../../ui-resources/choose-segment'

export function ActivityPage({
  activity,
  mods,
}: {
  activity: Activity[]
  mods: Array<{ id: string; ecosystem: 'EML' | 'SF' }>
}) {
  const { t } = useI18n()
  const [filter, setFilter] = useState('all')
  const [copyState, setCopyState] = useState<
    'idle' | 'copying' | 'copied' | 'error'
  >('idle')
  const copyRequest = useRef('')
  useEffect(() => {
    window.__shroudforgeLogCopyResult = (result) => {
      if (result.requestId !== copyRequest.current) return
      if (!result.success || result.text === undefined) {
        setCopyState('error')
        return
      }
      void writeClipboardText(result.text)
        .then(() => {
          setCopyState('copied')
          window.setTimeout(
            () =>
              setCopyState((current) =>
                current === 'copied' ? 'idle' : current,
              ),
            1800,
          )
        })
        .catch(() => setCopyState('error'))
    }
    return () => {
      window.__shroudforgeLogCopyResult = undefined
    }
  }, [])
  const items = activity.filter(
    (item) => filter === 'all' || item.level === filter,
  )
  const copyLabel =
    copyState === 'copying'
      ? t('activity.copying')
      : copyState === 'copied'
        ? t('activity.copied')
        : copyState === 'error'
          ? t('activity.copyFailed')
          : t('activity.copyLogs')
  return (
    <>
      <PageHeader
        eyebrow={t('nav.modloader')}
        title={t('activity.title')}
        subtitle={t('activity.subtitle')}
        actions={
          <>
            <button
              className="button ghost"
              disabled={copyState === 'copying'}
              onClick={() => {
                setCopyState('copying')
                copyRequest.current = postRequest('copy-logs')
              }}
            >
              <Icon name="copy" />
              {copyLabel}
            </button>
            <button
              className="button ghost"
              onClick={() => postRequest('refresh')}
            >
              <Icon name="refresh" />
              {t('common.refresh')}
            </button>
          </>
        }
      />
      <Segmented
        value={filter}
        onChange={setFilter}
        items={[
          ['all', t('activity.all')],
          ['info', t('activity.info')],
          ['warn', t('activity.warnings')],
          ['error', t('activity.errors')],
          ['debug', t('activity.debug')],
          ['trace', t('activity.trace')],
        ]}
      />
      <Card flush>
        {items.length ? (
          <div className="activity-table">
            <div className="table-head">
              <span>{t('activity.time')}</span>
              <span>{t('activity.source')}</span>
              <span>{t('activity.action')}</span>
              <span>{t('activity.result')}</span>
            </div>
            {items.map((item) => (
              <div className="table-row" key={item.id}>
                <time>{item.time}</time>
                <span>
                  {item.source}
                  <EcosystemBadge
                    ecosystem={
                      mods.find((mod) => mod.id === item.source)?.ecosystem
                    }
                  />
                </span>
                <span title={item.action}>{item.action}</span>
                <span className={`result ${item.level}`} title={item.result}>
                  <i />
                  {item.result}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <Empty
            title={t('activity.empty.title')}
            text={t('activity.empty.text')}
            icon="activity"
          />
        )}
      </Card>
    </>
  )
}
