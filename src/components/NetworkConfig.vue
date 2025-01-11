<script setup lang="ts">
import { hostname } from '@tauri-apps/plugin-os'
import { h } from 'vue'

const appStore = useAppStore()
const networkStore = useNetworkStore()
const { t } = useI18n()
const { cleanAutostartNetwork } = appStore
const { showMultipleNetwork } = storeToRefs(appStore)
const { removeNetwork, addNetwork } = networkStore
const { networkList, currentNetwork, networkCurrentId, isCurrentNetworkRunning } = storeToRefs(networkStore)
const device_name = ref('')

function onlyAllowHostname(value: string) {
  return !value || /^[\u4E00-\u9FA5a-z0-9\-]*$/i.test(value)
}

function resetConfig() {
  if (currentNetwork.value) {
    addNetwork()
    cleanAutostartNetwork(currentNetwork.value.config.instance_id)
    removeNetwork(currentNetwork.value.config.instance_id)
    nextTick(() => {
      networkCurrentId.value = networkList.value[networkList.value.length - 1]?.config.instance_id || ''
    })
  }
}

onMounted(async () => {
  device_name.value = await hostname() || ''
})
</script>

<template>
  <n-tabs v-if="currentNetwork" type="line" animated h-full>
    <template #suffix>
      <n-popconfirm
        :negative-text="t('component.networkConfig.reset')"
        :positive-text="t('component.networkConfig.cancel')" @negative-click="resetConfig"
      >
        <template #trigger>
          <n-button dashed type="error" size="small" :disabled="isCurrentNetworkRunning">
            {{ t('component.networkConfig.reset') }}
          </n-button>
        </template>
        {{ t('component.networkConfig.confirmResetMessageTwice') }}
      </n-popconfirm>
    </template>
    <n-tab-pane name="common" :tab="t('component.networkConfig.commonConfig')">
      <n-scrollbar :style="{ 'max-height': 'calc(100vh - 58px)' }">
        <n-form label-width="auto" :disabled="isCurrentNetworkRunning">
          <n-form-item v-show="showMultipleNetwork" :label="t('component.networkConfig.configName')">
            <n-input
              v-model:value="currentNetwork!.name"
              :placeholder="t('component.networkConfig.configNamePlaceholder')"
            />
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.deviceName')">
            <n-input
              v-model:value="currentNetwork.config.device_name"
              :placeholder="t('component.networkConfig.deviceNamePlaceholder', [device_name])"
              :allow-input="onlyAllowHostname"
            />
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.virtualIP')">
            <template #label>
              <n-flex align="center">
                <n-text>{{ t('component.networkConfig.virtualIP') }}</n-text>
                <n-checkbox v-model:checked="currentNetwork.config.dhcp" :checked-value="true" :unchecked-value="false">
                  {{ t('component.networkConfig.dhcp') }}
                </n-checkbox>
              </n-flex>
            </template>
            <n-input-group>
              <n-input
                v-model:value="currentNetwork.config.virtual_ipv4"
                :disabled="currentNetwork.config.dhcp || isCurrentNetworkRunning"
                :placeholder="t('component.networkConfig.virtualIPPlaceholder')" type="text"
                :style="{ width: '80%' }"
              />
              <n-input-number
                v-model:value="currentNetwork.config.network_length" :min="16" :max="32"
                :style="{ width: '20%' }" :show-button="false" :step="1"
              >
                <template #prefix>
                  /
                </template>
              </n-input-number>
            </n-input-group>
            <!-- <n-input v-model:value="currentNetwork.config.ipv4"
              :disabled="currentNetwork.config.dhcp || isCurrentNetworkRunning"
              :placeholder="t('component.networkConfig.virtualIPPlaceholder')" /> -->
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.peer')">
            <n-select
              v-model:value="currentNetwork.config.peer_urls" filterable tag multiple
              :options="currentNetwork.config.preset_peer_urls"
              :placeholder="t('component.networkConfig.peerPlaceholder')"
            />
          </n-form-item>
        </n-form>
      </n-scrollbar>
    </n-tab-pane>
    <n-tab-pane name="advance" :tab="t('component.networkConfig.advanceConfig')">
      <n-scrollbar :style="{ 'max-height': 'calc(100vh - 58px)' }">
        <n-form label-width="auto" :disabled="isCurrentNetworkRunning">
          <n-form-item :label="t('component.networkConfig.subnetProxy')">
            <n-dynamic-tags />
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.rpcPortal')">
            <n-input-number v-model:value="currentNetwork.config.rpc_port" :min="0" :max="65535" />
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.presetListenerUrl')">
            <n-select
              v-model:value="currentNetwork.config.listener_urls" tag multiple
              :options="currentNetwork.config.preset_listener_urls" max-tag-count="responsive"
              :placeholder="t('component.networkConfig.presetListenerUrl')" :style="{ width: '100%' }"
            />
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.listenerUrl')">
            <n-dynamic-input
              v-model:value="currentNetwork.config.listener_urls"
              :options="currentNetwork.config.preset_listener_urls" :disabled="isCurrentNetworkRunning"
            />
          </n-form-item>
          <n-form-item :label="t('component.networkConfig.vpnPortal')">
            <n-input-group>
              <n-input
                v-model:value="currentNetwork.config.vpn_portal_client_network_addr" type="text"
                :style="{ width: '50%' }"
              />
              <n-input-number
                v-model:value="currentNetwork.config.vpn_portal_client_network_len" :min="16" :max="32"
                :style="{ width: '20%' }" :show-button="false" :step="1"
              >
                <template #prefix>
                  /
                </template>
              </n-input-number>
              <n-input
                v-model:value="currentNetwork.config.vpn_portal_listen_port" type="text"
                :style="{ width: '30%' }"
              >
                <template #prefix>
                  :
                </template>
              </n-input>
            </n-input-group>
          </n-form-item>
        </n-form>
      </n-scrollbar>
    </n-tab-pane>
  </n-tabs>
  <n-empty v-else :description="t('component.networkConfig.empty')" h-full justify-center>
    <template #extra>
      {{ t('component.networkConfig.chooseOther') }}
    </template>
  </n-empty>
</template>

<style scoped lang="postcss">
.n-dynamic-input {
  & :deep(.n-button) {
    @apply rounded-none;
  }
}
</style>
