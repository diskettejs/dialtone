import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { bytesToString, commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      key: { type: 'string', short: 'k', default: 'demo/example/zenoh-rs-queryable' },
      payload: { type: 'string', short: 'p', default: 'Queryable from Rust!' },
      complete: { type: 'boolean' },
    },
  })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Queryable on '${values.key}'...`)
  await using queryable = await session.declareQueryable(values.key, { complete: values.complete })

  console.log('Press CTRL-C to quit...')
  // @ts-expect-error
  for await (const query of queryable.stream()) {
    const params = query.parameters
    const selector = params.isEmpty ? `${query.keyExpr}` : `${query.keyExpr}?${params}`

    const payload = query.payload
    if (payload === null) {
      console.log(`>> [Queryable] Received Query '${selector}'`)
    } else {
      // Refer to z_bytes.ts to see how to deserialize different types of message.
      console.log(
        `>> [Queryable] Received Query '${selector}' with payload '${bytesToString(payload)}'`,
      )
    }

    console.log(`>> [Queryable] Responding ('${values.key}': '${values.payload}')`)
    try {
      // Refer to z_bytes.ts to see how to serialize different types of message.
      await query.reply(values.key, values.payload)
    } catch (err) {
      console.log(`>> [Queryable] Error sending reply: ${err}`)
    }
  }
}

await main()
