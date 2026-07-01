import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'group1/zenoh-rs' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring LivelinessToken on '${values.key}'...`)
  await using _token = await session.liveliness().declareToken(values.key)

  console.log('Press CTRL-C to undeclare LivelinessToken and quit...')
  // Keep the token alive until the process is killed.
  await new Promise<void>(() => {})
}

await main()
