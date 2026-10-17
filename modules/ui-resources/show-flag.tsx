import flagSvgs from '@generated-flags'

export function Flag({ value }: { value: string }) {
  return (
    <span
      className="flag"
      aria-hidden
      dangerouslySetInnerHTML={{ __html: flagSvgs[value.toLowerCase()] || '' }}
    />
  )
}
