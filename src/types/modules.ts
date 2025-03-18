import type { App } from 'vue'

export type UseModule = (app: App<Element>) => Promise<void> | void
