import pkg from '~/../package.json'

export const version = pkg.version
const showFilter = ['dev', 'pre', 'test', 'alpha', 'beta', 'rc', 'canary']
export const needShowWatermark = showFilter.find(item => version.includes(item))
export const watermarkContent = `EraXvlanMesh ${import.meta.env.PROD ? '' : 'dev'} v:${version}`
