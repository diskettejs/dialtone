import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'demo/example/zenoh-rs-put' },
      payload: { type: 'string', short: 'p', default: 'Put from Rust!' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Putting Data ('${values.key}': '${values.payload}')...`)
  // Refer to z_bytes.ts to see how to serialize different types of message.
  await session.put(values.key, values.payload)
}

await main()
