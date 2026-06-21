// Shared test fixtures.
//
// These exist to keep the test bodies focused on the *binding seam* — the JS↔Rust
// marshalling, lifecycle, and channel semantics this library owns — rather than on
// Zenoh's transport timing. Two footguns in particular are handled here so they
// cannot recur per-test:
//
//   1. Cross-talk: vitest runs test files in parallel, each opening its own peer
//      session on the local network. Sessions discover each other, so two tests
//      using the same key expression would see each other's samples. `ke()` hands
//      out a process-and-load unique key expression so that can't happen.
//
//   2. The 10s stall: Zenoh's default get consolidation (`Auto`→`Latest`) holds
//      replies until the query finalizes, which — absent a router that tracks
//      queryable completeness — only happens at the default 10s timeout. `bounded`
//      pins consolidation to `None` (replies flow immediately) and adds a short
//      `timeout` (the channel closes promptly).
import type { Replies, ReplyError, ReplySample } from '../index.js'

// Unique per module load *and* per call: `Math.random` distinguishes parallel
// workers that share a pid; the counter distinguishes calls within one file.
const base = `${process.pid}-${Math.random().toString(36).slice(2, 8)}`
let seq = 0

/** A collision-free key expression rooted at a per-run-unique prefix. */
export function ke(name: string): string {
  return `dialtone-test/${base}/${seq++}/${name}`
}

/**
 * Bound a `recv()` so a delivery that never arrives fails fast instead of hanging
 * to the suite-wide timeout. Rejects on timeout (it does not masquerade as a
 * `null`/empty result, which would hide the missing delivery).
 */
export function recvWithin<T>(recv: () => Promise<T | null>, ms = 2_000): Promise<T | null> {
  let timer: ReturnType<typeof setTimeout>
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(`recv did not resolve within ${ms}ms`)), ms)
  })
  return Promise.race([recv().finally(() => clearTimeout(timer)), timeout])
}

/**
 * Get options that bound the query. Spread into a `session.get`/`querier.get`
 * options object for any test that is *not* specifically exercising consolidation
 * or timeout marshalling.
 */
export const bounded = { consolidation: 'None', timeout: 500 } as const

/**
 * Collect exactly `count` replies from a (bounded) get, failing fast if they do
 * not arrive. Stops early — and returns fewer than `count` — if the reply channel
 * closes first, so a caller asserting on `length` gets a clear failure.
 */
export async function collectReplies(
  replies: Replies,
  count: number,
  ms = 2_000,
): Promise<Array<ReplySample | ReplyError>> {
  const out: Array<ReplySample | ReplyError> = []
  while (out.length < count) {
    const reply = await recvWithin(() => replies.recv(), ms)
    if (reply === null) break
    out.push(reply)
  }
  return out
}
