export const useAppStore = defineStore('appStore', () => {
  const isDark = useStorage('dark', true)
  const showMultipleNetwork = useStorage('multiple', false)
  const configModel = ref(false)
  const config = useStorage('config', DEFAULT_APP_CONFIG())

  const toggleDark = () => {
    isDark.value = !isDark.value
  }

  function cleanAutostartNetwork(id: string) {
    config.value.autostart.network = config.value.autostart.network.filter(item => item !== id)
  }

  async function toggleAutostart(enable: boolean) {
    const ret = await setAutostart(enable)
    config.value.autostart.enable = ret
    return ret
  }
  async function setLevel(level: 'off' | 'error' | 'warn' | 'info' | 'debug' | 'trace' = 'debug') {
    try {
      setLoggingLevel(level)
    }
    catch {
    }
  }
  return {
    isDark,
    showMultipleNetwork,
    configModel,
    config,
    toggleDark,
    cleanAutostartNetwork,
    toggleAutostart,
    setLoggingLevel: setLevel,
  }
})

if (import.meta.hot)
  import.meta.hot.accept(acceptHMRUpdate(useAppStore as any, import.meta.hot))
