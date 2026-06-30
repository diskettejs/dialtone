import { Config } from './binding.js'

export function defineConfig(config = {}) {
  return Config.fromJson5(JSON.stringify(config))
}
