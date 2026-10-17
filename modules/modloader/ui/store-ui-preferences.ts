export const readUiStorage = (key: string, fallback: string) => {
  try {
    return window.localStorage.getItem(key) ?? fallback
  } catch {
    return fallback
  }
}

export const writeUiStorage = (key: string, value: string) => {
  try {
    window.localStorage.setItem(key, value)
  } catch {
    /* about:blank WebView origins may deny storage */
  }
}
