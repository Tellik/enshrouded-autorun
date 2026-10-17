export const post = (command: string, payload: Record<string, unknown> = {}) =>
  window.ipc?.postMessage(JSON.stringify({ command, ...payload }))

export let requestSequence = 0

export const postRequest = (
  command: string,
  payload: Record<string, unknown> = {},
) => {
  const requestId = `${Date.now()}-${++requestSequence}`
  post(command, { ...payload, requestId })
  return requestId
}
