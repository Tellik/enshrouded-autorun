import type React from 'react'
import type { UpdaterCommand } from './update-types'

export function renderReleaseInline(
  text: string,
  keyPrefix: string,
  onCommand: UpdaterCommand,
): React.ReactNode[] {
  const pattern =
    /\[([^\]]+)\]\((https:\/\/[^\s)]+)\)|(\*\*|__)(.+?)\3|(`)(.+?)\5|(\*|_)(.+?)\7/g
  const nodes: React.ReactNode[] = []
  let last = 0
  for (const match of text.matchAll(pattern)) {
    const index = match.index ?? 0
    if (index > last) nodes.push(text.slice(last, index))
    if (match[1] && match[2])
      nodes.push(
        <button
          className="release-note-link"
          key={`${keyPrefix}-${index}`}
          onClick={() => onCommand('open-url', { url: match[2] })}
        >
          {match[1]}
        </button>,
      )
    else if (match[4])
      nodes.push(<strong key={`${keyPrefix}-${index}`}>{match[4]}</strong>)
    else if (match[6])
      nodes.push(<code key={`${keyPrefix}-${index}`}>{match[6]}</code>)
    else if (match[8])
      nodes.push(<em key={`${keyPrefix}-${index}`}>{match[8]}</em>)
    last = index + match[0].length
  }
  if (last < text.length) nodes.push(text.slice(last))
  return nodes
}

export function ReleaseNotes({
  text,
  onCommand,
}: {
  text: string
  onCommand: UpdaterCommand
}) {
  const lines = text.replace(/\r/g, '').split('\n')
  const blocks: React.ReactNode[] = []
  for (let index = 0; index < lines.length; ) {
    const line = lines[index].trim()
    if (!line) {
      index++
      continue
    }
    const heading = line.match(/^(#{1,6})\s+(.+?)\s*#*$/)
    if (heading) {
      blocks.push(
        <h4 key={`heading-${index}`}>
          {renderReleaseInline(heading[2], `heading-${index}`, onCommand)}
        </h4>,
      )
      index++
      continue
    }
    const firstItem = line.match(/^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/)
    if (firstItem) {
      const items: React.ReactNode[] = []
      while (index < lines.length) {
        const item = lines[index].trim().match(/^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/)
        if (!item) break
        items.push(
          <li key={`item-${index}`}>
            {renderReleaseInline(item[1], `item-${index}`, onCommand)}
          </li>,
        )
        index++
      }
      blocks.push(<ul key={`list-${index}`}>{items}</ul>)
      continue
    }
    const paragraph = [line]
    index++
    while (
      index < lines.length &&
      lines[index].trim() &&
      !/^(?:#{1,6}\s+|[-*+]\s+|\d+[.)]\s+)/.test(lines[index].trim())
    ) {
      paragraph.push(lines[index].trim())
      index++
    }
    blocks.push(
      <p key={`paragraph-${index}`}>
        {renderReleaseInline(paragraph.join(' '), `paragraph-${index}`, onCommand)}
      </p>,
    )
  }
  return <div className="release-notes">{blocks}</div>
}
