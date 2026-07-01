import { type ParseArgsOptionsConfig } from 'node:util'
import { type Bytes, Config } from '../index.js'

/**
 * CLI options shared by every example, mirroring Zenoh's `CommonArgs`
 * (`zenoh/examples/src/lib.rs`). Spread into each example's own `options`:
 *
 * ```ts
 * const { values } = parseArgs({
 *   options: { ...commonOptions, key: { type: 'string', default: 'demo/**' } },
 * })
 * const session = await Session.open(configFromArgs(values))
 * ```
 */
export const commonOptions = {
  config: { type: 'string', short: 'c' },
  mode: { type: 'string', short: 'm' },
  connect: { type: 'string', short: 'e', multiple: true },
  listen: { type: 'string', short: 'l', multiple: true },
  cfg: { type: 'string', multiple: true },
  'no-multicast-scouting': { type: 'boolean' },
} satisfies ParseArgsOptionsConfig

/** The subset of parsed args {@link configFromArgs} reads. */
export interface CommonValues {
  config?: string
  mode?: string
  connect?: string[]
  listen?: string[]
  cfg?: string[]
  'no-multicast-scouting'?: boolean
}

/**
 * Builds a {@link Config} from parsed {@link commonOptions}. Mirrors the
 * `CommonArgs -> Config` conversion in Zenoh's examples: start from a file or
 * the default, then layer on `mode`, endpoints, and raw `--cfg KEY:VALUE`
 * overrides. Zenoh validates every insert and throws on bad keys/values.
 */
export function configFromArgs(values: CommonValues): Config {
  const config = values.config ? Config.fromFile(values.config) : Config.default()

  if (values.mode) config.insertJson5('mode', JSON.stringify(values.mode))
  if (values.connect?.length) {
    config.insertJson5('connect/endpoints', JSON.stringify(values.connect))
  }
  if (values.listen?.length) {
    config.insertJson5('listen/endpoints', JSON.stringify(values.listen))
  }
  if (values['no-multicast-scouting']) {
    config.insertJson5('scouting/multicast/enabled', 'false')
  }

  for (const pair of values.cfg ?? []) {
    const sep = pair.indexOf(':')
    if (sep === -1) throw new Error(`--cfg expects KEY:VALUE pairs, got '${pair}'`)
    config.insertJson5(pair.slice(0, sep), pair.slice(sep + 1))
  }

  return config
}

/**
 * Decodes a `Bytes` payload as a UTF-8 string, mirroring the Zenoh examples'
 * `payload.try_to_string().unwrap_or_else(|e| e.to_string().into())`: a
 * non-UTF-8 payload yields the decode error's text instead of throwing, so a
 * bad sample never aborts a receive loop.
 */
export function bytesToString(bytes: Bytes): string {
  try {
    return bytes.tryToString()
  } catch (err) {
    return err instanceof Error ? err.message : String(err)
  }
}
