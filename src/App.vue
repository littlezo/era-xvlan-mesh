<!-- eslint-disable no-console -->
<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { InstanceEvent, Network, NetworkInstanceInfo } from './types/network'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { exit } from '@tauri-apps/plugin-process'
import hljs from 'highlight.js'
import { darkTheme, dateZhCN, zhCN } from 'naive-ui'

const { locale, t } = useI18n()
const appStore = useAppStore()
const networkStore = useNetworkStore()
const { isDark, config, setLoggingLevel } = storeToRefs(appStore)
const { pushInfoStack, startNetwork } = networkStore
const { networkInfo, networkList, currentNetworkInfo } = storeToRefs(networkStore)

const theme = computed(() => (isDark.value ? darkTheme : null))

const eventListen = ref<UnlistenFn | null>(null)
const infoListen = ref<UnlistenFn | null>(null)
const requestListen = ref<UnlistenFn | null>(null)
const closeModel = ref(false)

async function hideCallback() {
  await getCurrentWindow().hide()
}

async function closeCallback() {
  await exit(0)
}

useTray(true)

onBeforeMount(async () => {
  setLoggingLevel('debug')
  if (await isAutostart()) {
    await getCurrentWindow().hide()
    config.value.autostart.network.forEach(async (id) => {
      await startNetwork(() => { }, id)
    })
  }
})

onMounted(async () => {
  console.log(currentNetworkInfo.value)
  eventListen.value = await listen<InstanceEvent>('era://xvlan/event', (event) => {
    console.log({ 'era://xvlan/event': event })
  })
  infoListen.value = await listen<NetworkInstanceInfo[]>('era://xvlan/mesh/info', (event) => {
    console.log({ 'era://xvlan/mesh/info': event })
    networkInfo.value = [...event.payload].map(x => ({ ...x, node: x?.my_node_info }))
    networkList.value.forEach((n: Network) => {
      // console.log(n.config.instance_id, event.payload[0].instance_id.toLowerCase())
      const p = event.payload.find(i => i.instance_id === n.config.instance_id.toLowerCase())
      const events = p?.events
      if (events && Array.isArray(events)) {
        p.events = events.map((e: string) => JSON.parse(e) as NetworkInstanceInfo['events'][0])
      }
      if (p) {
        n.detail = p
        pushInfoStack(n.config.instance_id, p)
      }
    })
  })
  requestListen.value = await listen<InstanceEvent>('era://xvlan/mesh/window/close', () => {
    console.log('era://xvlan/mesh/window/close')
    closeModel.value = true
  })
})

onBeforeUnmount(() => {
  // eventListen.value && eventListen.value()
  eventListen.value?.()
  infoListen.value?.()
  requestListen.value?.()
})
</script>

<template>
  <n-config-provider
    :theme :locale="locale === 'zh-CN' ? zhCN : undefined"
    :date-locale="locale === 'zh-CN' ? dateZhCN : undefined" :hljs="hljs"
  >
    <n-message-provider>
      <n-dialog-provider>
        <n-modal-provider>
          <RouterView />
          <n-modal
            v-model:show="closeModel" preset="dialog" :title="t('app.sureToExit')"
            :content="t('app.sureToExitContent')" :positive-text="t('app.minimize')" :negative-text="t('app.exit')"
            @positive-click="hideCallback" @negative-click="closeCallback"
          />
          <n-watermark
            v-if="needShowWatermark" :content="watermarkContent" cross fullscreen :font-size="16"
            :line-height="16" :width="240" :height="240" :x-offset="12" :y-offset="64" :rotate="-15"
          />
        </n-modal-provider>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
