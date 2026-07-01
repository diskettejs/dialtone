import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      'key-expr': { type: 'string', short: 'k', default: 'group1/**' },
      timeout: { type: 'string', short: 'o', default: '10000' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  const keyExpr = values['key-expr']
  console.log(`Sending Liveliness Query '${keyExpr}'...`)
  const replies = await session.liveliness().get(keyExpr, { timeout: Number(values.timeout) })

  for await (const reply of replies.stream()) {
    if (reply.error) {
      console.log(`>> Received (ERROR: '${reply.error.payload.toString()}')`)
    } else {
      console.log(`>> Alive token ('${reply.sample.keyExpr}')`)
    }
  }
}

await main()
