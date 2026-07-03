import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { bytesToString, commonOptions, configFromArgs } from './common.ts'

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
    const { result } = reply
    if (result.error) {
      console.log(`>> Received (ERROR: '${bytesToString(result.error.payload)}')`)
    } else {
      console.log(`>> Alive token ('${result.sample.keyExpr}')`)
    }
  }
}

await main()
