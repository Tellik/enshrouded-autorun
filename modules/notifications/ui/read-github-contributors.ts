import type { GitHubContributor } from './notification-types'
export let githubContributorRequest: Promise<GitHubContributor[]> | undefined

export function getGitHubContributors() {
  if (!githubContributorRequest) {
    let cached: GitHubContributor[] = []
    let cachedAt = 0
    try {
      const value = JSON.parse(
        localStorage.getItem('sf-github-contributors') || 'null',
      )
      if (value && Array.isArray(value.items)) {
        cachedAt = Number(value.savedAt) || 0
        cached = value.items
          .filter(
            (item: unknown): item is GitHubContributor =>
              !!item &&
              typeof item === 'object' &&
              typeof (item as GitHubContributor).login === 'string' &&
              typeof (item as GitHubContributor).html_url === 'string' &&
              /^https:\/\/github\.com\/[A-Za-z0-9-]+\/?$/.test(
                (item as GitHubContributor).html_url,
              ) &&
              typeof (item as GitHubContributor).contributions === 'number',
          )
          .slice(0, 8)
      }
    } catch {}
    if (cached.length && Date.now() - cachedAt < 21600000)
      githubContributorRequest = Promise.resolve(cached)
    else
      githubContributorRequest = fetch(
        'https://api.github.com/repos/bonsaibauer/shroudforge/contributors?per_page=8',
        { headers: { Accept: 'application/vnd.github+json' } },
      )
        .then((response) => (response.ok ? response.json() : []))
        .then((items: unknown) => {
          const list = Array.isArray(items)
            ? items
                .filter(
                  (item): item is GitHubContributor =>
                    !!item &&
                    typeof item === 'object' &&
                    typeof (item as GitHubContributor).login === 'string' &&
                    typeof (item as GitHubContributor).html_url === 'string' &&
                    /^https:\/\/github\.com\/[A-Za-z0-9-]+\/?$/.test(
                      (item as GitHubContributor).html_url,
                    ) &&
                    typeof (item as GitHubContributor).contributions ===
                      'number',
                )
                .slice(0, 8)
            : []
          if (list.length) {
            try {
              localStorage.setItem(
                'sf-github-contributors',
                JSON.stringify({ savedAt: Date.now(), items: list }),
              )
            } catch {}
          }
          return list.length ? list : cached
        })
        .catch(() => cached)
  }
  return githubContributorRequest
}
