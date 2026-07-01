import { parseArgs } from 'node:util'
import { Session, type GetOptions, type QueryTarget } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

const QUERY_TARGETS: Record<string, QueryTarget> = {
  BEST_MATCHING: 'BestMatching',
  ALL: 'All',
  ALL_COMPLETE: 'AllComplete',
}

async function main() {
  const { values } = parseArgs({
    options: {
      ...commonOptions,
      selector: { type: 'string', short: 's', default: 'demo/example/**' },
      payload: { type: 'string', short: 'p' },
      target: { type: 'string', short: 't', default: 'BEST_MATCHING' },
      timeout: { type: 'string', short: 'o', default: '10000' },
    },
  })

  const target = QUERY_TARGETS[values.target]
  if (!target) {
    throw new Error(`invalid --target '${values.target}' (BEST_MATCHING|ALL|ALL_COMPLETE)`)
  }

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Sending Query '${values.selector}'...`)
  // By default get receives replies from a FIFO; pass a RingChannel via
  // `capacity`/a channel option to use a ring instead (see z_pull.ts).
  const options: GetOptions = { target, timeout: Number(values.timeout) }
  if (values.payload !== undefined) options.payload = values.payload
  const replies = await session.get(values.selector, options)

  for await (const reply of replies.stream()) {
    if (reply.error) {
      // Refer to z_bytes.ts to see how to deserialize different types of message.
      console.log(`>> Received (ERROR: '${reply.error.payload.toString()}')`)
    } else {
      const { sample } = reply
      console.log(`>> Received ('${sample.keyExpr}': '${sample.payload.toString()}')`)
    }
  }
}

await main()
