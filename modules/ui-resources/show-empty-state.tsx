import { Icon } from './show-icon'

export function Empty({
  title,
  text,
  icon,
}: {
  title: string
  text: string
  icon: string
}) {
  return (
    <div className="empty">
      <span>
        <Icon name={icon} />
      </span>
      <h3>{title}</h3>
      <p>{text}</p>
    </div>
  )
}

export function Loading({ text }: { text: string }) {
  return (
    <div className="empty">
      <span className="spinner" />
      <p>{text}</p>
    </div>
  )
}
