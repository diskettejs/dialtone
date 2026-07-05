import { parseArgs } from 'node:util'
import { type LogLevel, type LogRecord, initLog, Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

const LEVELS = ['Trace', 'Debug', 'Info', 'Warn', 'Error'] as const

/** Accepts Zenoh-style lowercase (`info`) and returns a typed {@link LogLevel}. */
function toLogLevel(input: string): LogLevel {
  const normalized = (input[0]?.toUpperCase() ?? '') + input.slice(1).toLowerCase()
  const level = LEVELS.find((l) => l === normalized)
  if (!level) {
    throw new Error(
      `invalid --level '${input}', expected one of: ${LEVELS.join(', ').toLowerCase()}`,
    )
  }
  return level
}

/** Renders a {@link LogRecord} as a single line, e.g. `INFO  zenoh::net: opened (src/lib.rs:42)`. */
function format(record: LogRecord): string {
  let line = `${record.level.toUpperCase().padEnd(5)} ${record.target}`
  if (record.message) line += `: ${record.message}`
  if (record.attributes.length) {
    line += ` {${record.attributes.map((a) => `${a.key}=${a.value}`).join(', ')}}`
  }
  if (record.file) line += ` (${record.file}${record.line ? `:${record.line}` : ''})`
  return line
}

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      level: { type: 'string', default: 'info' },
      key: { type: 'string', short: 'k', default: 'demo/example/logging' },
    },
  })

  const level = toLogLevel(values.level)

  // Route Zenoh's internal `tracing` logs to our callback. Call this once,
  // before opening a session, so session setup is captured. It installs a
  // process-global subscriber; the returned boolean is `false` if one was
  // already installed (e.g. a second call), in which case this call is a no-op.
  const initialized = initLog((record) => console.log(format(record)), level)
  console.log(initialized ? `Logging initialized at '${level}'` : 'Logging was already initialized')

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))
  console.log(`Session zid: ${await session.info().zid()}`)

  // Publish periodically so Zenoh keeps emitting logs. Press CTRL-C to quit.
  console.log('Press CTRL-C to quit...')
  for (let i = 0; ; i++) {
    await session.put(values.key, `[${i}] hello`)
    await new Promise((resolve) => setTimeout(resolve, 1000))
  }
}

await main()
