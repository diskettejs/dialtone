import { setTimeout as sleep } from 'node:timers/promises'
import { parseArgs } from 'node:util'
import { Parameters, Session, type QuerierGetOptions, type QueryTarget } from '../index.js'
import { bytesToString, commonOptions, configFromArgs } from './common.ts'

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
      'add-matching-listener': { type: 'boolean' },
    },
  })

  const target = QUERY_TARGETS[values.target]
  if (!target) {
    throw new Error(`invalid --target '${values.target}' (BEST_MATCHING|ALL|ALL_COMPLETE)`)
  }

  // A querier is declared on a key expression, so split off any "?parameters".
  const sep = values.selector.indexOf('?')
  const keyExpr = sep === -1 ? values.selector : values.selector.slice(0, sep)
  const params = sep === -1 ? '' : values.selector.slice(sep + 1)

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  console.log(`Declaring Querier on '${keyExpr}'...`)
  await using querier = await session.declareQuerier(keyExpr, {
    target,
    timeout: Number(values.timeout),
  })

  if (values['add-matching-listener']) {
    const listener = await querier.matchingListener()
    // No callbacks in Dialtone: consume the listener as an async iterator.
    void (async () => {
      // @ts-expect-error
      for await (const status of listener.stream()) {
        console.log(
          status.matching
            ? 'Querier has matching queryables.'
            : 'Querier has NO MORE matching queryables.',
        )
      }
    })()
  }

  console.log('Press CTRL-C to quit...')
  for (let idx = 0; ; idx++) {
    await sleep(1000)
    const buf = `[${idx.toString().padStart(4)}] ${values.payload ?? ''}`
    console.log(`Querying '${values.selector}' with payload: '${buf}'...`)

    // Refer to z_bytes.ts to see how to serialize different types of message.
    const options: QuerierGetOptions = { payload: buf, parameters: new Parameters(params) }
    const replies = await querier.get(options)
    // @ts-expect-error
    for await (const reply of replies.stream()) {
      if (reply.error) {
        console.log(`>> Received (ERROR: '${bytesToString(reply.error.payload)}')`)
      } else {
        console.log(
          `>> Received ('${reply.sample.keyExpr}': '${bytesToString(reply.sample.payload)}')`,
        )
      }
    }
  }
}

await main()
