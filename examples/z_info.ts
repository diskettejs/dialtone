import { parseArgs } from 'node:util'
import { Session } from '../index.js'
import { commonOptions, configFromArgs } from './common.ts'

async function main() {
  const { values } = parseArgs({ options: { ...commonOptions } })

  console.log('Opening session...')
  await using session = await Session.open(configFromArgs(values))

  const info = session.info()
  console.log(`zid: ${await info.zid()}`)
  console.log(`routers zid: ${(await info.routersZid()).join(', ')}`)
  console.log(`peers zid: ${(await info.peersZid()).join(', ')}`)
}

await main()
