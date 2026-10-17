export async function writeClipboardText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    const temporary = document.createElement('textarea')
    temporary.value = text
    temporary.style.position = 'fixed'
    temporary.style.opacity = '0'
    document.body.append(temporary)
    temporary.select()
    const copied = document.execCommand('copy')
    temporary.remove()
    if (!copied) throw new Error('Clipboard write failed')
  }
}
