import { postRequest } from '../../transport/ui/send-host-command'
import type React from 'react'
import { useState } from 'react'
import { useI18n } from '../../ui-resources/i18n'
import type { Snapshot } from '../../modloader/ui/ui-snapshot'
import { empty } from '../../modloader/ui/ui-snapshot'
import { Empty, Loading } from '../../ui-resources/show-empty-state'
import { Icon } from '../../ui-resources/show-icon'
import { PageHeader } from '../../ui-resources/show-page-header'

export function Discover({ data }: { data: Snapshot }) {
  const { t } = useI18n()
  const [query, setQuery] = useState(data.catalog?.query || '')
  const catalog = data.catalog || empty.catalog!
  const search = (event: React.FormEvent) => {
    event.preventDefault()
    postRequest('search-catalog', { query })
  }
  return (
    <>
      <PageHeader
        eyebrow={t('nav.mods')}
        title={t('discover.title')}
        subtitle={t('discover.subtitle')}
      />
      <form className="searchbar" onSubmit={search}>
        <Icon name="search" />
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t('discover.placeholder')}
        />
        <button className="button" type="submit">
          {t('discover.search')}
        </button>
      </form>
      {catalog.state === 'loading' ? (
        <Loading text={t('discover.catalog.loading')} />
      ) : catalog.items.length ? (
        <div className="catalog-grid">
          {catalog.items.map((item) => (
            <article className="catalog-card" key={item.id}>
              <div className="catalog-icon">
                {item.iconUrl ? (
                  <img src={item.iconUrl} />
                ) : (
                  item.name.slice(0, 2).toUpperCase()
                )}
              </div>
              <div>
                <h3>{item.name}</h3>
                <p>{item.summary}</p>
                <div className="meta">
                  <span>{item.author || t('discover.community')}</span>
                  {item.version && <span>{item.version}</span>}
                </div>
              </div>
              <button
                className="button"
                onClick={() =>
                  postRequest('install-mod', {
                    projectId: item.id,
                    title: item.name,
                  })
                }
              >
                {t('updates.queue.addMod')}
              </button>
            </article>
          ))}
        </div>
      ) : (
        <Empty
          title={
            catalog.state === 'error'
              ? t('discover.catalog.unavailable')
              : t('discover.empty.title')
          }
          text={
            catalog.state === 'ready'
              ? t('discover.empty.noResults')
              : catalog.message || t('discover.empty.text')
          }
          icon="search"
        />
      )}
    </>
  )
}
