import { useEffect, useRef, useState } from 'react'
import { logUiEvent } from '../../logging/ui/log-ui-event'
import { useI18n } from '../../ui-resources/i18n'
import { Icon } from '../../ui-resources/show-icon'
import { readWikiArticle } from './read-wiki-articles'
import { WikiArticle } from './show-wiki-article'

export function WikiHelp({ articleId, compact = false }: { articleId: string; compact?: boolean }) {
  const { t, locale } = useI18n()
  const [visible, setVisible] = useState(!compact)
  const [expanded, setExpanded] = useState(false)
  const article = readWikiArticle(articleId, locale)
  const missingArticle = useRef('')
  useEffect(() => {
    if (article || missingArticle.current === `${articleId}:${locale}`) return
    missingArticle.current = `${articleId}:${locale}`
    logUiEvent({ level: 'ERROR', source: 'wiki', operation: 'Load help', subject: articleId,
      message: t('wiki.unavailable'), notify: true })
  }, [articleId, locale, Boolean(article), t])
  function showHelp(nextVisible: boolean, nextExpanded: boolean) {
    logUiEvent({ level: 'DEBUG', source: 'wiki', operation: 'Change help view', subject: articleId,
      message: !nextVisible ? 'Help banner hidden.' : nextExpanded ? 'Full article opened.' : 'Article collapsed to its banner.' })
    setVisible(nextVisible)
    setExpanded(nextExpanded)
  }
  if (!article) return <p className="hint" role="alert">{t('wiki.unavailable')}</p>
  if (!visible) return <button type="button" className="button ghost wiki-help-trigger"
    aria-label={t('wiki.openGuide', { title: article.title })} title={t('wiki.openGuide', { title: article.title })}
    onClick={() => showHelp(true, true)}>
    <Icon name="wiki" />{!compact && t('wiki.showGuide')}
  </button>
  return <section className="wiki-help">
    <WikiArticle article={article} expanded={expanded}>
      <div className="feature-banner-actions">
        <button type="button" className="button ghost" aria-expanded={expanded}
          onClick={() => showHelp(true, !expanded)}><Icon name="wiki" />
          {t(expanded ? 'wiki.collapseGuide' : 'wiki.showGuide')}
        </button>
        <button type="button" className="button ghost" onClick={() => showHelp(false, expanded)}>{t('wiki.hideGuide')}</button>
      </div>
    </WikiArticle>
  </section>
}
