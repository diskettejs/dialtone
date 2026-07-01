import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'group1/**' },
      history: { type: 'boolean' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Liveliness Subscriber on '${values.key}'...`)
  await using subscriber = await session
    .liveliness()
    .declareSubscriber(values.key, { history: values.history })

  console.log('Press CTRL-C to quit...')
  for await (const sample of subscriber.stream()) {
    if (sample.kind === 'Put') {
      console.log(`>> [LivelinessSubscriber] New alive token ('${sample.keyExpr}')`)
    } else {
      console.log(`>> [LivelinessSubscriber] Dropped token ('${sample.keyExpr}')`)
    }
  }
}

await main()
