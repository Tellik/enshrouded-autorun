import type { ModInfo } from './mod-types'
import { postRequest } from '../../transport/ui/send-host-command'
import { useEffect, useRef, useState } from 'react'

export function useModSettings(mod: ModInfo) {
  const [enabled, setLocalEnabled] = useState(mod.enabled)
  const initial = () =>
    Object.fromEntries(
      mod.settings
        .filter((item) => item.key)
        .map((item) => [
          item.key,
          mod.settingValues[item.key] ??
            item.default ??
            (item.type === 'boolean' ? false : ''),
        ]),
    )
  const [values, setValues] = useState<Record<string, unknown>>(initial)
  const valuesRef = useRef(values)
  const revisionRef = useRef(mod.revision)
  const modIdRef = useRef(mod.id)
  const autoSaveTimer = useRef<number | undefined>(undefined)
  const pendingAutoSave = useRef(false)
  useEffect(() => {
    const changedMod = modIdRef.current !== mod.id
    if (changedMod) {
      if (autoSaveTimer.current !== undefined) {
        window.clearTimeout(autoSaveTimer.current)
        autoSaveTimer.current = undefined
      }
      if (pendingAutoSave.current) {
        pendingAutoSave.current = false
        postRequest('save-mod-settings', {
          modId: modIdRef.current,
          values: valuesRef.current,
          revision: revisionRef.current,
        })
      }
      modIdRef.current = mod.id
      valuesRef.current = initial()
      setValues(valuesRef.current)
    } else if (!pendingAutoSave.current) {
      valuesRef.current = initial()
      setValues(valuesRef.current)
    }
    revisionRef.current = mod.revision
    setLocalEnabled(mod.enabled)
  }, [mod.id, mod.revision])
  useEffect(
    () => () => {
      if (autoSaveTimer.current !== undefined)
        window.clearTimeout(autoSaveTimer.current)
      if (pendingAutoSave.current)
        postRequest('save-mod-settings', {
          modId: modIdRef.current,
          values: valuesRef.current,
          revision: revisionRef.current,
        })
    },
    [],
  )
  const save = () => {
    if (autoSaveTimer.current !== undefined) {
      window.clearTimeout(autoSaveTimer.current)
      autoSaveTimer.current = undefined
    }
    pendingAutoSave.current = false
    postRequest('save-mod-settings', {
      modId: mod.id,
      values: valuesRef.current,
      revision: revisionRef.current,
    })
  }
  const updateSetting = (key: string, value: unknown) => {
    const next = { ...valuesRef.current, [key]: value }
    valuesRef.current = next
    setValues(next)
    pendingAutoSave.current = true
    if (autoSaveTimer.current !== undefined)
      window.clearTimeout(autoSaveTimer.current)
    autoSaveTimer.current = window.setTimeout(() => {
      autoSaveTimer.current = undefined
      if (!pendingAutoSave.current) return
      pendingAutoSave.current = false
      postRequest('save-mod-settings', {
        modId: modIdRef.current,
        values: valuesRef.current,
        revision: revisionRef.current,
      })
    }, 350)
  }
  const setEnabled = (next: boolean) => {
    setLocalEnabled(next)
    postRequest('set-mod-enabled', {
      modId: mod.id,
      enabled: next,
      revision: revisionRef.current,
    })
  }
  return { enabled, values, save, updateSetting, setEnabled }
}
