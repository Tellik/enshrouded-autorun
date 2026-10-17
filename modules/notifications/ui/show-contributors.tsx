import { postRequest } from '../../transport/ui/send-host-command'
import { useEffect, useState } from 'react'
import { useI18n } from '../../ui-resources/i18n'
import type { GitHubContributor } from './notification-types'

import { Icon } from '../../ui-resources/show-icon'
import { getGitHubContributors } from './read-github-contributors'
export function ContributorList() {
  const { t } = useI18n()
  const [contributors, setContributors] = useState<GitHubContributor[] | null>(
    null,
  )
  useEffect(() => {
    let active = true
    getGitHubContributors().then((items) => {
      if (active) setContributors(items)
    })
    return () => {
      active = false
    }
  }, [])
  return (
    <section className="notice-contributors">
      <div>
        <strong>{t('news.contributors.title')}</strong>
        <p>{t('news.contributors.description')}</p>
      </div>
      {contributors === null ? (
        <small>{t('news.contributors.loading')}</small>
      ) : contributors.length ? (
        <ul>
          {contributors.map((person) => (
            <li key={person.login}>
              <button
                onClick={() =>
                  postRequest('open-url', { url: person.html_url })
                }
              >
                <Icon name="github" />
                <span>{person.login}</span>
                <small>
                  {t('news.contributors.commits', {
                    count: person.contributions,
                  })}
                </small>
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <small>{t('news.contributors.unavailable')}</small>
      )}
      <button
        className="contributors-link"
        onClick={() =>
          postRequest('open-url', {
            url: 'https://github.com/bonsaibauer/shroudforge/graphs/contributors',
          })
        }
      >
        {t('news.contributors.all')}
      </button>
    </section>
  )
}
