import type { NetworkConfig } from '~/types/network'
import { invoke } from '@tauri-apps/api/core'

export async function parseNetworkConfig(cfg: NetworkConfig) {
  return invoke('parse_network_config', { cfg })
}

export async function startNetworkInstance(cfg: NetworkConfig) {
  return invoke('start_network_instance', { cfg })
}

export async function stopNetworkInstance(id: string) {
  return invoke('stop_network_instance', { id })
}

export async function collectNetworkInfos() {
  return await invoke('collect_network_infos')
}

export async function setLoggingLevel(level: string) {
  return await invoke('set_logging_level', { level })
}

export async function setTunFd(instanceId: string, fd: number) {
  return await invoke('set_tun_fd', { instanceId, fd })
}

export async function test_config(cfg: NetworkConfig) {
  return await invoke('test_config', { config: cfg })
}

export async function isAutostart() {
  return await invoke<boolean>('is_autostart')
}
