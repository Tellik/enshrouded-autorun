import type { ModInfo } from './mod-types'

export function ModTypeMark({ ecosystem }: { ecosystem: 'EML' | 'SF' }) {
  return (
    <span
      className={`mod-type-mark ${ecosystem.toLowerCase()}`}
      aria-hidden="true"
    >
      {ecosystem}
    </span>
  )
}

export function ModIcon({
  mod,
  variant,
}: {
  mod: ModInfo
  variant: 'nav' | 'list' | 'page'
}) {
  const image = mod.icon ? mod.assets[mod.icon] : undefined
  const className =
    variant === 'nav'
      ? 'mod-nav-avatar'
      : variant === 'page'
        ? 'mod-page-avatar'
        : 'mod-avatar'
  return (
    <span className={`${className} ${mod.ecosystem.toLowerCase()}`}>
      {image ? (
        <img src={image} alt="" />
      ) : (
        <ModTypeMark ecosystem={mod.ecosystem} />
      )}
    </span>
  )
}
