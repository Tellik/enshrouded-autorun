import { postRequest } from '../../transport/ui/send-host-command'
import { useI18n } from '../../ui-resources/i18n'
import { useState } from 'react'
import { Snapshot } from '../../modloader/ui/ui-snapshot'
import { Empty } from '../../ui-resources/show-empty-state'
import { Icon } from '../../ui-resources/show-icon'
import { PageHeader } from '../../ui-resources/show-page-header'
import { Segmented } from '../../ui-resources/choose-segment'
import { SystemUpdate } from './show-system-update'
import { UpdateQueuePanel } from './show-update-queue'

export function Updates({
  data,
}: { data: Snapshot }) {
  const { t } = useI18n()
  const [tab, setTab] = useState('all')
  const queue = data.updateQueue || { items: [] }
  const queuedSystem = queue.items.some(
    (item) =>
      item.kind === 'system' &&
      item.version === data.release.latestVersion &&
      !['complete', 'cancelled'].includes(item.state),
  )
  return (
    <>
      <PageHeader
        eyebrow={t('nav.mods')}
        title={t('updates.title')}
        subtitle={t('updates.subtitle')}
        actions={
          <button
            className="button"
            onClick={() => postRequest('check-updates')}
            disabled={data.release.state === 'checking'}
          >
            <Icon name="refresh" />
            {data.release.state === 'checking'
              ? t('updates.checking')
              : t('updates.check')}
          </button>
        }
      />
      <Segmented
        value={tab}
        onChange={setTab}
        items={[
          ['all', t('common.all')],
          ['system', t('updates.system')],
          ['mods', t('updates.mods')],
          ['history', t('updates.history')],
        ]}
      />
      {tab !== 'history' && (
        <UpdateQueuePanel
          queue={queue}
          mode={data.mode}
          release={data.release}
        />
      )}
      {tab === 'mods' ? (
        <Empty
          title={t('updates.mods.empty.title')}
          text={t('updates.mods.empty.text')}
          icon="updates"
        />
      ) : tab === 'history' ? (
        <Empty
          title={t('updates.history.empty.title')}
          text={t('updates.history.empty.text')}
          icon="activity"
        />
      ) : (
        <SystemUpdate release={data.release} queued={queuedSystem} />
      )}
    </>
  )
}
