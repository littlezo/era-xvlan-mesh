import type { DataInfo, Network, NetworkConfig, NetworkInfoStack, NetworkInstanceInfo } from '~/types/network'

const MAX_STACK = 120
export const useNetworkStore = defineStore('networkStore', () => {
  const networkList = useStorage<Network[]>('networkList', [])
  const networkInfo = ref<NetworkInstanceInfo[]>([])

  const networkFilter = ref<string>('')
  const networkCurrentId = useStorage<string>('networkCurrentId', networkList.value.length ? networkList.value[0].config.instance_id : '')

  const currentNetwork = computed<Network | undefined>(() => {
    return networkList.value.find(item => item.config.instance_id === networkCurrentId.value)
  })

  const currentNetworkInfo = computed<NetworkInstanceInfo | undefined>(() => {
    return networkInfo.value.find(item => item.instance_id.toLowerCase() === networkCurrentId.value.toLowerCase())
  })

  const currentNetworkInfoData = computed<DataInfo[]>(() => {
    return peerRoutePairToStatusData(currentNetworkInfo.value?.peer_route_pairs || [])
  })

  const isCurrentNetworkRunning = computed<boolean>(() => {
    return !!networkInfo.value.find(i => i.instance_id.toLowerCase() === networkCurrentId.value.toLowerCase())
  })

  const networkInfoDataStack = reactive<Record<string, NetworkInfoStack[]>>({})

  const currentNetworkInfoDataStack = computed<NetworkInfoStack[]>(() => {
    return networkInfoDataStack[networkCurrentId.value] || []
  })

  function pushInfoStack(id: string, info: NetworkInstanceInfo) {
    const now = new Date()
    const nowTime = `${now.getHours()}:${now.getMinutes()}:${now.getSeconds()}`
    if (!networkInfoDataStack[id])
      networkInfoDataStack[id] = []

    networkInfoDataStack[id].push({
      time: nowTime,
      id: info.instance_id,
      node: info.node,
      peerRoutePair: info.peer_route_pairs,
    })

    if (networkInfoDataStack[id].length > MAX_STACK)
      networkInfoDataStack[id].shift()
  }

  function addNetwork() {
    const newNetwork: Network = {
      name: `EraXvlanMesh-${uuid(3)}`,
      config: DEFAULT_NETWORK_CONFIG(),
      status: NetworkStatus.OFF,
      otherConfig: DEFAULT_NETWORK_OTHER_CONFIG(),
    }
    networkList.value.push(newNetwork)
  }

  function removeNetwork(id: string) {
    networkList.value = networkList.value.filter(network => network.config.instance_id !== id)
  }

  async function startNetwork(callback: (e: any) => void, id: string | undefined = void 0) {
    id ||= networkCurrentId.value
    if (id === void 0 || id === '') {
      callback('Network id is required')
      return
    }
    const network = networkList.value.find(network => network.config.instance_id === id)
    if (network) {
      // eslint-disable-next-line no-console
      console.info('startNetwork', network)
      const cfg: NetworkConfig = { ...network.config }
      // eslint-disable-next-line no-console
      console.log('parseNetworkConfig', cfg)
      if (network.otherConfig.token) {
        // @ts-expect-error ts-migrate(2345)
        delete cfg.network_name
        // @ts-expect-error ts-migrate(2345)
        delete cfg.network_secret
      }
      else {
        delete cfg.token
      }
      try {
        const res = await parseNetworkConfig(cfg)
        // eslint-disable-next-line no-console
        console.log('parseNetworkConfig', res)
        await startNetworkInstance(cfg)
      }
      catch (e: any) {
        console.error(e)
        callback(e)
      }
    }
  }

  async function stopNetwork(callback: (e: any) => void = () => { }, id: string | undefined = undefined) {
    const network = id ? networkList.value.find(network => network.config.instance_id === id) : currentNetwork.value
    if (network) {
      try {
        await stopNetworkInstance(network.config.instance_id)
      }
      catch (e: any) {
        callback(e)
      }
    }
  }
  async function setLevel(level: 'off' | 'error' | 'warn' | 'info' | 'debug' | 'trace' = 'debug') {
    try {
      setLoggingLevel(level)
    }
    catch {
    }
  }
  return {
    networkList,
    networkInfo,
    networkFilter,
    networkCurrentId,
    networkInfoDataStack,
    currentNetwork,
    currentNetworkInfo,
    currentNetworkInfoData,
    isCurrentNetworkRunning,
    currentNetworkInfoDataStack,
    addNetwork,
    removeNetwork,
    startNetwork,
    stopNetwork,
    pushInfoStack,
    setLoggingLevel: setLevel,
  }
})

if (import.meta.hot)
  import.meta.hot.accept(acceptHMRUpdate(useNetworkStore as any, import.meta.hot))
