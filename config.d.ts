import * as binding from './binding.js'

export type JsonObject = { [Key in string]: JsonValue }
export type JsonArray = JsonValue[] | readonly JsonValue[]
export type JsonPrimitive = string | number | boolean | null
export type JsonValue = JsonPrimitive | JsonObject | JsonArray

/**
 * A value that is either applied uniformly, or per-mode.
 *
 * The unique form applies whatever this node's {@link ZenohConfig.mode} is. The
 * mode-dependent form picks the entry matching this node's own mode; omitted modes are
 * left unset and fall back to Zenoh's default for that mode.
 *
 * e.g. `["tcp/10.0.0.1:7447"]` (unique) or
 * `{ router: ["tcp/10.0.0.1:7447"], peer: [] }` (mode-dependent).
 */
export type ModeDependent<T> = T | { router?: T; peer?: T; client?: T }

/**
 * A value that is either applied uniformly, or selected by the kind of the *remote*
 * node it applies to — as opposed to {@link ModeDependent}, which selects on this
 * node's own mode.
 *
 * e.g. `"always"` (unique) or `{ to_router: "always", to_peer: "greater-zid" }`
 * (target-dependent).
 */
export type TargetDependent<T> = T | { to_router?: T; to_peer?: T; to_client?: T }

/**
 * Strategy for autoconnection, mainly to avoid nodes connecting to each other
 * redundantly.
 *
 * - `always` — always attempt to connect to the other node. This may open a redundant
 *   connection, which is then closed.
 * - `greater-zid` — attempt to connect only if this node's own zid is greater than the
 *   other's, so that when both nodes use this strategy exactly one of them dials. Not
 *   suited when one node cannot reach the other, for example because of a private IP.
 *
 * Defaults to `always`.
 */
export type AutoConnectStrategy = 'always' | 'greater-zid'

/**
 * Backoff applied between connection attempts, used by
 * {@link ConnectConfig.retry} and {@link ListenConfig.retry}.
 *
 * The wait starts at `period_init_ms`, is multiplied by `period_increase_factor` after
 * every failed attempt, and is capped at `period_max_ms`.
 *
 * These values can also be set per endpoint in the locator string, overriding the
 * global ones — e.g.
 * `tcp/192.168.0.1:7447#retry_period_init_ms=20000;retry_period_max_ms=10000`.
 */
export interface ConnectionRetryConfig {
  /** Initial wait before the next attempt, in milliseconds. Defaults to `1000`. */
  period_init_ms?: ModeDependent<number>
  /** Maximum wait before the next attempt, in milliseconds. Defaults to `4000`. */
  period_max_ms?: ModeDependent<number>
  /** Factor the wait is multiplied by after each failed attempt. Defaults to `2`. */
  period_increase_factor?: ModeDependent<number>
}

/**
 * Which Zenoh nodes to connect to on session open.
 *
 * @see {@link ListenConfig} for the inbound side.
 */
export interface ConnectConfig {
  /**
   * Timeout for the whole connect cycle, in milliseconds: `0` retries nothing and
   * `-1` waits forever.
   *
   * Defaults to `-1` in router and peer mode, `0` in client mode.
   */
  timeout_ms?: ModeDependent<number>
  /**
   * The endpoints to connect to, as `<proto>/<address>` locator strings —
   * e.g. `tcp/localhost:7447`.
   *
   * A locator may carry per-link options appended with `#key=value;…`, such as
   * `#iface=eth0` (TCP/UDP on Linux), `#bind=192.168.0.1:0` (TCP, UDP, QUIC and TLS;
   * mutually exclusive with `iface`), `#so_sndbuf=65000;so_rcvbuf=65000` (TCP and TLS)
   * and `#dscp=0x08` (TCP/UDP). Priority and reliability are set in the query string
   * instead, e.g. `tcp/localhost?prio=6-7;rel=0`.
   *
   * Defaults to `[]`.
   */
  endpoints?: ModeDependent<string[]>
  /**
   * Exit the application if {@link ConnectConfig.timeout_ms} is exceeded.
   *
   * Defaults to `false` in router and peer mode, `true` in client mode.
   */
  exit_on_failure?: ModeDependent<boolean>
  /** Backoff between connection attempts. */
  retry?: ConnectionRetryConfig
}

/**
 * Which endpoints to listen on, i.e. the addresses other nodes can use to establish a
 * session with this one.
 *
 * @see {@link ConnectConfig} for the outbound side.
 */
export interface ListenConfig {
  /**
   * Timeout for the whole listen cycle, in milliseconds: `0` retries nothing and `-1`
   * waits forever.
   *
   * Defaults to `0`.
   */
  timeout_ms?: ModeDependent<number>
  /**
   * The endpoints to listen on, as `<proto>/<address>` locator strings —
   * e.g. `tcp/0.0.0.0:7447`. Port `0` binds an arbitrary free port.
   *
   * The same per-link options as {@link ConnectConfig.endpoints} apply.
   *
   * Defaults to `["tcp/[::]:7447"]` in router mode and `["tcp/[::]:0"]` in peer mode;
   * a client listens on nothing.
   */
  endpoints?: ModeDependent<string[]>
  /**
   * Exit the application if {@link ListenConfig.timeout_ms} is exceeded.
   *
   * Defaults to `true`.
   */
  exit_on_failure?: ModeDependent<boolean>
  /** Backoff between listen attempts. */
  retry?: ConnectionRetryConfig
}

/** Configures what `Session.open` waits for before it resolves. */
export interface OpenConfig {
  /** The conditions to be met before `Session.open` returns. */
  return_conditions?: {
    /**
     * Wait to connect to scouted peers and routers before returning. When `false`, the
     * first publications and queries issued after open may be lost.
     *
     * Defaults to `true`.
     */
    connect_scouted?: boolean
    /**
     * Wait to receive the initial declares from connected peers before returning.
     * Setting this to `false` may cause extra traffic at startup from peers.
     *
     * Defaults to `true`.
     */
    declares?: boolean
  }
}

/** Discovery of other nodes over UDP multicast. */
export interface ScoutingMulticastConfig {
  /** Whether multicast scouting is enabled. Defaults to `true`. */
  enabled?: boolean
  /**
   * The `address:port` socket used for multicast scouting.
   *
   * Defaults to `"224.0.0.224:7446"`.
   */
  address?: string
  /**
   * The network interface used for multicast scouting. `"auto"` picks one
   * automatically.
   *
   * Defaults to `"auto"`.
   */
  interface?: string
  /** Time-to-live set on multicast scouting packets. Defaults to `1`. */
  ttl?: number
  /**
   * Which kinds of node to automatically establish a session with upon discovery over
   * UDP multicast. Each value is a list, and an empty list disables autoconnection.
   *
   * Defaults to `[]` in router mode and `["router", "peer", "client"]` in peer and
   * client mode.
   */
  autoconnect?: ModeDependent<binding.WhatAmI[]>
  /**
   * Strategy used when {@link ScoutingMulticastConfig.autoconnect} fires, selectable
   * both by this node's mode and by the kind of node being connected to —
   * e.g. `{ peer: { to_router: "always", to_peer: "greater-zid" } }`.
   *
   * Defaults to `"always"`.
   */
  autoconnect_strategy?: ModeDependent<TargetDependent<AutoConnectStrategy>>
  /**
   * Whether to listen for scout messages on UDP multicast and reply to them.
   *
   * Defaults to `true`.
   */
  listen?: ModeDependent<boolean>
}

/**
 * Discovery of other nodes by gossip, i.e. by having already-connected nodes forward
 * what they know. Nodes in client mode do not participate in gossip.
 */
export interface ScoutingGossipConfig {
  /** Whether gossip scouting is enabled. Defaults to `true`. */
  enabled?: boolean
  /**
   * When `true`, gossip information is propagated over multiple hops to every node in
   * the local network; when `false`, only to the next hop. Multihop gossip implies more
   * scouting traffic and lower scalability, and mostly makes sense when nodes lack
   * direct connectivity with each other.
   *
   * Defaults to `false`.
   */
  multihop?: boolean
  /**
   * Which kinds of node to send gossip messages to.
   *
   * Defaults to `["router", "peer"]` in router and peer mode, `[]` in client mode.
   */
  target?: ModeDependent<binding.WhatAmI[]>
  /**
   * Which kinds of node to automatically establish a session with upon discovery
   * through gossip. Each value is a list, and an empty list disables autoconnection.
   *
   * Defaults to `[]` in router mode and `["router", "peer", "client"]` in peer and
   * client mode.
   */
  autoconnect?: ModeDependent<binding.WhatAmI[]>
  /**
   * Strategy used when {@link ScoutingGossipConfig.autoconnect} fires, selectable both
   * by this node's mode and by the kind of node being connected to.
   *
   * Defaults to `"always"`.
   */
  autoconnect_strategy?: ModeDependent<TargetDependent<AutoConnectStrategy>>
}

/** How this node discovers the rest of the network. */
export interface ScoutingConfig {
  /**
   * In client mode, how long to scout for a router before failing, in milliseconds.
   *
   * Defaults to `3000`.
   */
  timeout?: number
  /**
   * In peer mode, the maximum time spent scouting remote peers before moving on to
   * other operations, in milliseconds.
   *
   * Defaults to `500`.
   */
  delay?: number
  /** Discovery over UDP multicast. */
  multicast?: ScoutingMulticastConfig
  /** Discovery by gossip between connected nodes. */
  gossip?: ScoutingGossipConfig
}

/** How this node timestamps the data messages passing through it. */
export interface TimestampingConfig {
  /**
   * Whether data messages should be timestamped if they are not already.
   *
   * Defaults to `true` in router mode and `false` in peer and client mode.
   */
  enabled?: ModeDependent<boolean>
  /**
   * Whether data messages carrying a timestamp in the future should be dropped rather
   * than re-timestamped. Ignored when timestamping is disabled.
   *
   * Defaults to `false`.
   */
  drop_future_timestamp?: boolean
}

/**
 * Aggregation of declarations, which collapses the individual declarations covered by a
 * key expression into a single one announced to the rest of the network.
 */
export interface AggregationConfig {
  /** Key expressions all included subscribers are aggregated into. */
  subscribers?: string[]
  /** Key expressions all included publishers are aggregated into. */
  publishers?: string[]
}

/**
 * An array Zenoh rejects when empty. Leaving the key out entirely is the way to mean
 * "no restriction"; an empty list matches nothing and is a configuration error.
 */
export type NonEmptyArray<T> = [T, ...T[]]

/** The direction a network interceptor applies to. */
export type InterceptorFlow = 'egress' | 'ingress'

/**
 * A link protocol, used both to whitelist the protocols a session may use and to narrow
 * an interceptor to the transports carrying at least one link of that kind.
 *
 * Which of these are actually available depends on the features Zenoh was compiled
 * with.
 */
export type LinkProtocol =
  | 'tcp'
  | 'udp'
  | 'tls'
  | 'quic'
  | 'ws'
  | 'serial'
  | 'unixpipe'
  | 'unixsock-stream'
  | 'vsock'

/** Weight of one outgoing link, used by a router in linkstate mode. */
export interface TransportWeight {
  /** The Zenoh ID of the destination node. */
  dst_zid: string
  /** Weight of the link from this node to that destination. Must not be `0`. */
  weight: number
}

/** The routing strategy and its configuration. */
export interface RoutingConfig {
  /** The routing strategy used in router mode. */
  router?: {
    /** Linkstate mode configuration. */
    linkstate?: {
      /**
       * Weights of this node's outgoing links. A link whose two endpoints both leave
       * their weight unset gets a weight of `100`; if only one endpoint specifies a
       * weight it is applied; if both do, the greater one wins.
       */
      transport_weights?: TransportWeight[]
    }
  }
  /** Interests-based routing, which applies in every mode. */
  interests?: {
    /**
     * How long to wait for incoming interest declarations, in milliseconds. Letting
     * this expire means the discovery protocol may be incomplete, which can cost
     * messages, queries or liveliness tokens.
     *
     * Defaults to `10000`.
     */
    timeout?: number
  }
}

/** QoS applied to the publications matching a key expression. */
export interface PublicationQoSOverwrite {
  /** What to do when the transmission queue is full. */
  congestion_control?: binding.CongestionControl
  /** The priority the messages are sent at. */
  priority?: binding.Priority
  /** Whether to send the messages immediately rather than batching them. */
  express?: boolean
  /** Whether the messages may be lost in transit. */
  reliability?: binding.Reliability
  /** Which entities the messages may reach. */
  allowed_destination?: binding.Locality
}

/** A publication QoS rule, matching by key expression. */
export interface PublicationQoSConfig {
  /**
   * PUT and DELETE messages on key expressions included by these have their QoS
   * overwritten by {@link PublicationQoSConfig.config}.
   */
  key_exprs: string[]
  /** The QoS values applied on the publisher, overwriting what the API asked for. */
  config: PublicationQoSOverwrite
}

/** The kinds of message a network QoS overwrite rule can apply to. Replies cannot. */
export type QosOverwriteMessage = 'put' | 'delete' | 'query'

/**
 * An inclusive payload size range, written as `"<start>..<end>"` in bytes. Either bound
 * may be omitted — `"1000000.."` matches everything from 1 MB up, `"..99"` everything
 * up to and including 99 bytes.
 *
 * Each bound must be a non-negative integer. A negative bound satisfies this type but is
 * still rejected by Zenoh.
 */
export type PayloadSizeRange = `${bigint}..${bigint}` | `${bigint}..` | `..${bigint}` | '..'

/** Narrows a network QoS overwrite rule to the messages already carrying this QoS. */
export interface QosFilter {
  congestion_control?: binding.CongestionControl
  priority?: binding.Priority
  express?: boolean
  reliability?: binding.Reliability
}

/**
 * A network QoS overwrite rule. It applies to every message matching all of its filter
 * fields; each field left out matches everything.
 *
 * This is more finely grained than {@link PublicationQoSConfig} — it can select on
 * interface, link protocol, peer and payload size — but also less performant.
 */
export interface NetworkQosOverwriteConfig {
  /** Optional identifier, which must be unique across the list. */
  id?: string
  /** Only overwrite QoS on messages exchanged with these Zenoh IDs. */
  zids?: NonEmptyArray<string>
  /** Only overwrite QoS on these network interfaces. */
  interfaces?: NonEmptyArray<string>
  /** Only overwrite QoS on transports carrying a link of one of these kinds. */
  link_protocols?: NonEmptyArray<LinkProtocol>
  /** The kinds of message this rule applies to. */
  messages: NonEmptyArray<QosOverwriteMessage>
  /** Only overwrite QoS on messages matching these key expressions. */
  key_exprs?: NonEmptyArray<string>
  /** The QoS values to overwrite. Fields left out keep the message's current value. */
  overwrite: {
    /**
     * The new priority: either an absolute priority, or an integer in `-7..=7` shifting
     * the message's current priority by that many steps.
     */
    priority?: binding.Priority | number
    congestion_control?: binding.CongestionControl
    express?: boolean
  }
  /** The flows this rule applies to. Both when left out. */
  flows?: NonEmptyArray<InterceptorFlow>
  /** Only overwrite QoS on messages already carrying this QoS. */
  qos?: QosFilter
  /** Only overwrite QoS on messages whose payload size falls in this range. */
  payload_size?: PayloadSizeRange
}

/**
 * QoS overwrites applied by key expression, which take precedence over the QoS
 * requested through the API.
 */
export interface QoSConfig {
  /** QoS overwrites for outgoing PUT and DELETE messages. */
  publication?: PublicationQoSConfig[]
  /** QoS overwrites for messages sent to and received from the network. */
  network?: NetworkQosOverwriteConfig[]
}

/** Unicast transport limits and features. */
export interface TransportUnicastConfig {
  /** Timeout when opening a link, in milliseconds. Defaults to `10000`. */
  open_timeout?: number
  /** Timeout when accepting a link, in milliseconds. Defaults to `10000`. */
  accept_timeout?: number
  /**
   * How many links may sit in the pending state during the accept handshake.
   *
   * Defaults to `100`.
   */
  accept_pending?: number
  /** Maximum number of simultaneously alive unicast transports. Defaults to `1000`. */
  max_sessions?: number
  /**
   * Maximum number of incoming links admitted per transport. Setting this above `1`
   * also allows multiple outgoing links; otherwise only one outgoing link is allowed.
   *
   * Defaults to `1`.
   */
  max_links?: number
  /**
   * Enable the LowLatency transport. This does not make it mandatory — which transport
   * is actually used depends on the establish procedure and the other party's settings.
   *
   * LowLatency does not preserve QoS prioritization, so it is incompatible with
   * {@link TransportUnicastConfig.qos} and requires disabling it explicitly. It also
   * does not support fragmentation, so messages must stay under the TX `batch_size`.
   *
   * Defaults to `false`.
   */
  lowlatency?: boolean
  /** QoS on unicast communications. */
  qos?: {
    /** Defaults to `true`. */
    enabled?: boolean
  }
  /**
   * Compression on unicast communications, negotiated during session establishment and
   * activated only if both nodes support it. Requires Zenoh's `transport_compression`
   * feature.
   */
  compression?: {
    /** Defaults to `false`. */
    enabled?: boolean
  }
}

/**
 * Multicast transport limits and features.
 *
 * Multicast performs no negotiation on group joining, so every node in the system must
 * agree on these parameters — in particular `transport.link.tx.batch_size`, whose
 * default is platform-dependent.
 */
export interface TransportMulticastConfig {
  /** JOIN message transmission interval, in milliseconds. Defaults to `2500`. */
  join_interval?: number
  /** Maximum number of multicast sessions. Defaults to `1000`. */
  max_sessions?: number
  /** QoS on multicast communications. */
  qos?: {
    /** Defaults to `false`, for out-of-the-box Zenoh-Pico compatibility. */
    enabled?: boolean
  }
  /** Compression on multicast communications. */
  compression?: {
    /** Defaults to `false`, for out-of-the-box Zenoh-Pico compatibility. */
    enabled?: boolean
  }
}

/** The resolution in bits used for message sequence numbers. */
export type SequenceNumberResolution = '8bit' | '16bit' | '32bit' | '64bit'

/** How the batches of a priority queue are allocated. */
export type QueueAllocMode =
  /** Allocate every batch when the queue is initialized. */
  | 'init'
  /** Allocate batches as needed, up to the configured queue size. */
  | 'lazy'

/**
 * The number of batches each priority queue can hold. Each value must be between `1`
 * and `16`; anything else is a configuration error.
 *
 * The memory allocated per queue is its size times the batch size (or times the link
 * MTU, when that is smaller). When QoS is disabled only the `data` priority is
 * allocated.
 */
export interface QueueSizeConfig {
  /** Defaults to `2`. */
  control?: number
  /** Defaults to `2`. */
  real_time?: number
  /** Defaults to `2`. */
  interactive_high?: number
  /** Defaults to `2`. */
  interactive_low?: number
  /** Defaults to `2`. */
  data_high?: number
  /** Defaults to `2`. */
  data?: number
  /** Defaults to `2`. */
  data_low?: number
  /** Defaults to `2`. */
  background?: number
}

/** The transmission queue of a link. */
export interface LinkQueueConfig {
  /** How many batches each priority queue can hold. */
  size?: QueueSizeConfig
  /**
   * What to do when the queue is congested, i.e. has no batch available.
   */
  congestion_control?: {
    /** Behaviour for messages sent with `CongestionControl.drop`. */
    drop?: {
      /**
       * Maximum time to wait for an available batch before dropping a droppable
       * message, in microseconds.
       *
       * Defaults to `1000`.
       */
      wait_before_drop?: number
      /**
       * Maximum deadline for multi-fragment messages, in microseconds.
       *
       * Defaults to `50000`.
       */
      max_wait_before_drop_fragments?: number
    }
    /** Behaviour for messages sent with `CongestionControl.block`. */
    block?: {
      /**
       * Maximum time to wait for an available batch before closing the transport
       * session, in microseconds.
       *
       * Defaults to `5000000`.
       */
      wait_before_close?: number
    }
  }
  /** Adaptive batching of messages smaller than the batch size. */
  batching?: {
    /**
     * Batch small messages together when the network is not fast enough to transmit
     * them individually, cutting the overall network overhead. Batching is driven by
     * network back-pressure, so it mostly kicks in on high-throughput streams of small
     * messages.
     *
     * Defaults to `true`.
     */
    enabled?: boolean
    /**
     * How long a message may be retained for batching under back-pressure, in
     * milliseconds.
     *
     * Defaults to `1`.
     */
    time_limit?: number
  }
  /** Memory allocation of the batches in the priority queues. */
  allocation?: {
    /** Defaults to `"lazy"`. */
    mode?: QueueAllocMode
  }
}

/** TX parameters of a link. */
export interface LinkTxConfig {
  /**
   * The resolution used for message sequence numbers. When establishing a session, the
   * lower of the two nodes' values is used.
   *
   * Defaults to `"32bit"`.
   */
  sequence_number_resolution?: SequenceNumberResolution
  /**
   * Link lease duration announced to other nodes, in milliseconds.
   *
   * Defaults to `10000`.
   */
  lease?: number
  /**
   * How many keep-alive messages are sent within one lease duration, when no data is
   * flowing. Four keep-alives per lease follows ITU-T G.8013/Y.1731, which considers a
   * link failed after 3.5 missed intervals.
   *
   * Defaults to `4`.
   */
  keep_alive?: number
  /**
   * Zenoh's MTU equivalent, in bytes. Expressed as a 16-bit unsigned integer, so the
   * maximum is `65535`.
   *
   * Defaults to `65535`.
   */
  batch_size?: number
  /** The transmission queue of the link. */
  queue?: LinkQueueConfig
  /** Number of threads used for transmission. Defaults to `1 + (cpus - 1) / 4`. */
  threads?: number
}

/** RX parameters of a link. */
export interface LinkRxConfig {
  /**
   * Receive buffer size per link, in bytes. Raising it accommodates more in-flight
   * data, which matters for very high throughput or large messages — e.g. `16777216`
   * for 16 MiB.
   *
   * Defaults to `65535`, the default batch size.
   */
  buffer_size?: number
  /**
   * Maximum size of the defragmentation buffer at the receiving end, in bytes.
   * Fragmented messages larger than this are dropped. Lower it on memory-constrained
   * devices.
   *
   * Defaults to `1073741824` (1 GiB).
   */
  max_message_size?: number
}

/** TLS parameters, which also cover QUIC links. */
export interface TlsConfig {
  /**
   * Path to the certificate of the certificate authority validating the other side's
   * keys and certificates. In router mode, the default WebPKI certificates are used
   * when this is unset.
   */
  root_ca_certificate?: string
  /** Path to the private key of the listening side. */
  listen_private_key?: string
  /** Path to the public certificate of the listening side. */
  listen_certificate?: string
  /** Enable mTLS, i.e. client authentication. Defaults to `false`. */
  enable_mtls?: boolean
  /** Path to the private key of the connecting side. */
  connect_private_key?: string
  /** Path to the certificate of the connecting side. */
  connect_certificate?: string
  /**
   * Verify that the server's certificate matches the hostname being connected to.
   * Setting this to `false` makes Zenoh disregard the common names of server
   * certificates, so a certificate signed for one host is accepted at another.
   *
   * Defaults to `true`.
   */
  verify_name_on_connect?: boolean
  /**
   * Close TLS and QUIC links when the remote certificate chain expires. A listener can
   * only disconnect a client this way if mTLS is enabled.
   *
   * Defaults to `false`.
   */
  close_link_on_expiration?: boolean
  /** TCP write buffer size, in bytes. */
  so_sndbuf?: number
  /** TCP read buffer size, in bytes. */
  so_rcvbuf?: number
  /** The CA certificate inline, base64-encoded, instead of a path. */
  root_ca_certificate_base64?: string
  /** The listening side's private key inline, base64-encoded, instead of a path. */
  listen_private_key_base64?: string
  /** The listening side's certificate inline, base64-encoded, instead of a path. */
  listen_certificate_base64?: string
  /** The connecting side's private key inline, base64-encoded, instead of a path. */
  connect_private_key_base64?: string
  /** The connecting side's certificate inline, base64-encoded, instead of a path. */
  connect_certificate_base64?: string
}

/** Per-link-kind transport parameters. */
export interface TransportLinkConfig {
  /**
   * Whitelist of protocols usable for accepting and opening sessions. Every supported
   * protocol is whitelisted when this is unset.
   *
   * Zenoh takes plain strings here and only rejects an unknown protocol when the link
   * is established, so this union is narrower than what the configuration parser
   * accepts.
   */
  protocols?: LinkProtocol[]
  /** TX parameters. */
  tx?: LinkTxConfig
  /** RX parameters. */
  rx?: LinkRxConfig
  /** TLS and QUIC parameters. */
  tls?: TlsConfig
  /** TCP parameters. */
  tcp?: {
    /** TCP write buffer size, in bytes. */
    so_sndbuf?: number
    /** TCP read buffer size, in bytes. */
    so_rcvbuf?: number
  }
  /** Unix pipe parameters. */
  unixpipe?: {
    /** Access mask applied to the pipe's file. */
    file_access_mask?: number
  }
}

/** How shared-memory internals are brought up. */
export type ShmInitMode =
  /** Initialize on session open: slower startup, no latency on the first SHM buffer. */
  | 'init'
  /** Initialize on the first SHM buffer: faster startup, latency on first use. */
  | 'lazy'

/**
 * Shared memory, which only has an effect when Zenoh is compiled with the
 * `shared-memory` feature.
 */
export interface SharedMemoryConfig {
  /**
   * Announce SHM buffer optimization support to other parties. This does not make SHM
   * mandatory; a probing procedure runs on session open, and SHM only takes over when
   * the other side has it enabled too — otherwise Zenoh falls back to network mode.
   *
   * Defaults to `true`.
   */
  enabled?: boolean
  /** Defaults to `"lazy"`. */
  mode?: ShmInitMode
  /** Automatic use of shared memory for large messages. */
  transport_optimization?: {
    /**
     * Put large messages into shared memory on SHM-compatible transports.
     *
     * Defaults to `true`.
     */
    enabled?: boolean
    /**
     * Size of the SHM arena used for this, in bytes. Must not be `0`.
     *
     * Defaults to `16777216` (16 MiB).
     */
    pool_size?: number
    /**
     * Only optimize messages at least this large, in bytes.
     *
     * Defaults to `3072`.
     */
    message_size_threshold?: number
  }
}

/** Authentication credentials presented and accepted on session establishment. */
export interface AuthConfig {
  /** Username and password authentication. Supplying a password requires a user. */
  usrpwd?: {
    user?: string
    password?: string
    /** Path to a file of `<user>:<password>` lines. */
    dictionary_file?: string
  }
  /** Public key authentication. */
  pubkey?: {
    public_key_pem?: string
    private_key_pem?: string
    public_key_file?: string
    private_key_file?: string
    key_size?: number
    known_keys_file?: string
  }
}

/** Internal transport parameters. */
export interface TransportConfig {
  /** Unicast transport limits and features. */
  unicast?: TransportUnicastConfig
  /** Multicast transport limits and features. */
  multicast?: TransportMulticastConfig
  /** Per-link-kind parameters. */
  link?: TransportLinkConfig
  /** Shared memory. */
  shared_memory?: SharedMemoryConfig
  /** Authentication credentials. */
  auth?: AuthConfig
}

/**
 * The admin space, which exposes this node's state and configuration as queryable key
 * expressions under `@/<zid>/<mode>`.
 *
 * This part of the configuration is unstable: it works as advertised, but may change in
 * a future Zenoh release.
 */
export interface AdminSpaceConfig {
  /** Defaults to `false`. */
  enabled?: boolean
  /** What the admin space allows. */
  permissions?: {
    /** Whether the admin space replies to queries. Defaults to `true`. */
    read?: boolean
    /**
     * Whether the admin space accepts configuration changes at runtime.
     *
     * Defaults to `false`.
     */
    write?: boolean
  }
}

/** Whether a matching access-control rule grants or refuses access. */
export type Permission = 'allow' | 'deny'

/** The kinds of message an access-control rule can apply to. */
export type AclMessage =
  | 'put'
  | 'delete'
  | 'declare_subscriber'
  | 'query'
  | 'declare_queryable'
  | 'reply'
  | 'liveliness_token'
  | 'declare_liveliness_subscriber'
  | 'liveliness_query'

/** An access-control rule: what is permitted, on which key expressions. */
export interface AclRuleConfig {
  /** Identifier, which must be unique across the rule set. */
  id: string
  /** The key expressions this rule covers. */
  key_exprs: NonEmptyArray<string>
  /** The kinds of message this rule covers. */
  messages: NonEmptyArray<AclMessage>
  /** The flows this rule covers. Both when left out. */
  flows?: NonEmptyArray<InterceptorFlow>
  /** Whether the covered messages are allowed or denied. */
  permission: Permission
}

/**
 * A combination of subject properties. A message matches it when it matches every
 * property listed; a property left out is a wildcard, and a property present but empty
 * is a configuration error.
 *
 * Listing several values for a property expands to the combinations of them, so
 * `{ interfaces: ["lo0", "en0"], usernames: ["zenoh"] }` matches
 * `(lo0 && zenoh) || (en0 && zenoh)`.
 */
export interface AclSubjectConfig {
  /** Identifier, which must be unique across the subjects list. */
  id: string
  /** Network interfaces. */
  interfaces?: NonEmptyArray<string>
  /** Certificate common names, when using TLS or QUIC. */
  cert_common_names?: NonEmptyArray<string>
  /** Usernames, when using user/password authentication. */
  usernames?: NonEmptyArray<string>
  /** Link protocols carried by the transport. */
  link_protocols?: NonEmptyArray<LinkProtocol>
  /**
   * Zenoh IDs.
   *
   * A ZID is not backed by an authentication mechanism, so it can only be trusted for
   * access control when Zenoh adds and removes it itself as transports open and close.
   * Managing these by hand is useful for prototyping but should not be relied on in
   * production.
   */
  zids?: NonEmptyArray<string>
}

/** Binds rules to subjects, both referenced by their identifiers. */
export interface AclPolicyConfig {
  /** Optional identifier, which must be unique across the policies list. */
  id?: string
  /** The {@link AclRuleConfig.id} values this policy applies. */
  rules: string[]
  /** The {@link AclSubjectConfig.id} values this policy applies them to. */
  subjects: string[]
}

/** Access control (ACL). */
export interface AccessControlConfig {
  /** Access control only takes effect when this is `true`. Defaults to `false`. */
  enabled?: boolean
  /** What to do with a message no rule matches. Defaults to `"deny"`. */
  default_permission?: Permission
  /** The rules granting or denying access to key expressions. */
  rules?: AclRuleConfig[]
  /** The subject combinations the rules can be bound to. */
  subjects?: AclSubjectConfig[]
  /** The bindings between rules and subjects. */
  policies?: AclPolicyConfig[]
}

/** The kinds of message a low-pass filter can apply to. */
export type LowPassFilterMessage = 'put' | 'delete' | 'query' | 'reply'

/**
 * A low-pass filter item, which drops messages whose payload plus attachment exceeds a
 * size limit. Each filter field left out matches everything.
 */
export interface LowPassFilterConfig {
  /** Optional identifier, which must be unique across the list. */
  id?: string
  /** Only filter on these network interfaces. */
  interfaces?: NonEmptyArray<string>
  /** Only filter on transports carrying a link of one of these kinds. */
  link_protocols?: NonEmptyArray<LinkProtocol>
  /** The flows to filter. Both when left out. */
  flows?: NonEmptyArray<InterceptorFlow>
  /** The kinds of message to filter. */
  messages: NonEmptyArray<LowPassFilterMessage>
  /** The key expressions whose matching messages are filtered. */
  key_exprs: NonEmptyArray<string>
  /** Inclusive maximum size of the serialized payload plus attachment, in bytes. */
  size_limit: number
}

/** The kinds of message downsampling can apply to. */
export type DownsamplingMessage = 'put' | 'delete' | 'query' | 'reply'

/** A downsampling rule: a key expression and the rate it is capped at. */
export interface DownsamplingRuleConfig {
  /** The key expression the rule applies to. */
  key_expr: string
  /** The maximum frequency, in Hertz. */
  freq: number
}

/**
 * A downsampling item, which caps how often matching messages pass through. Messages
 * that do not match are passed on untouched, and each filter field left out matches
 * everything.
 */
export interface DownsamplingConfig {
  /** Optional identifier, which must be unique across the list. */
  id?: string
  /** Only downsample on these network interfaces. */
  interfaces?: NonEmptyArray<string>
  /** Only downsample on transports carrying a link of one of these kinds. */
  link_protocols?: NonEmptyArray<LinkProtocol>
  /** The flows to downsample. Both when left out. */
  flows?: NonEmptyArray<InterceptorFlow>
  /** The kinds of message to downsample. */
  messages: NonEmptyArray<DownsamplingMessage>
  /** The rate cap per key expression. */
  rules: NonEmptyArray<DownsamplingRuleConfig>
}

/** Per-key-expression statistics. */
export interface StatsConfig {
  /** The key expressions statistics are collected for. */
  filters?: Array<{ key: string }>
}

/**
 * A filter assigning remotes to a gateway subregion. A remote matches it when it
 * matches every field listed; a field left out matches everything.
 */
export interface GatewayFilterConfig {
  /** The kinds of node to match. */
  modes?: binding.WhatAmI[]
  /** The network interfaces to match. */
  interfaces?: NonEmptyArray<string>
  /** The Zenoh IDs to match. */
  zids?: NonEmptyArray<string>
  /** The {@link ZenohConfig.region_name} values to match. */
  region_names?: NonEmptyArray<string>
  /** Invert the filter, matching every remote it would otherwise reject. */
  negated?: boolean
}

/** One south-bound subregion. Its index in the list is its subregion id. */
export interface GatewaySubregionConfig {
  /**
   * A remote is assigned to this subregion when it matches *any* of these filters, so
   * an empty list matches nothing. Omitting `filters` matches every remote.
   */
  filters?: GatewayFilterConfig[]
}

/** Gateway configuration. */
export interface GatewayConfig {
  /**
   * The south-bound topology, either a preset or an explicit list of subregions. A
   * remote is assigned the id of the first subregion whose filters match it.
   *
   * `"auto"`, the only preset, puts peers and clients south of routers and clients
   * south of peers.
   */
  south?: 'auto' | GatewaySubregionConfig[]
}

/**
 * A directory to search for plugin libraries. A plain string is a filesystem path,
 * shell-expanded — `"."` is the working directory and `"~/.zenoh/lib"` expands as
 * expected. The object form additionally allows `{ kind: "current_exe_parent" }`, the
 * directory holding the current executable.
 */
export type LibSearchDir =
  | string
  | { kind: 'path'; value: string }
  | { kind: 'current_exe_parent'; value?: null }

/** Where plugin libraries are looked up. */
export interface PluginsLoadingConfig {
  /** Plugins are only loaded when this is `true`. Defaults to `false`. */
  enabled?: boolean
  /**
   * Directories searched for plugins configured by name. Plugins configured with
   * `__path__` are not looked up.
   *
   * Defaults to
   * `[{ kind: "current_exe_parent" }, ".", "~/.zenoh/lib", "/opt/homebrew/lib", "/usr/local/lib", "/usr/lib"]`.
   */
  search_dirs?: LibSearchDir[]
}

/**
 * The configuration of a single plugin. Beyond the `__`-prefixed keys Zenoh itself
 * interprets, a plugin may take any JSON configuration of its own.
 *
 * Any value held under a `private` key is hidden from the admin space, which is where
 * passwords and other secrets belong.
 */
export interface PluginConfig {
  /**
   * Allow Zenoh to panic on problems with this plugin rather than logging them. Plugins
   * are expected to honour this for their own non-recoverable errors too.
   *
   * Defaults to `false`.
   */
  __required__?: boolean
  /**
   * Path(s) to the plugin's dynamic library. Supplying any path disables the search by
   * name, and the first path that loads is used.
   */
  __path__?: string | string[]
  /** The plugin's library name, when it differs from the key this entry is under. */
  __plugin__?: string
  /**
   * Path to a file whose contents are merged into this configuration, overriding the
   * properties given here. Nested `__config__` keys are resolved recursively.
   */
  __config__?: string
  [key: string]: JsonValue | undefined
}

/**
 * A typed view of Zenoh's session configuration. Keys mirror Zenoh's
 * configuration (`snake_case`); every field is optional and falls back to
 * Zenoh's defaults.
 *
 * @see [`DEFAULT_CONFIG.json5`](https://github.com/eclipse-zenoh/zenoh/blob/1.9.0/DEFAULT_CONFIG.json5)
 * for the annotated upstream reference of every key, including the open sections.
 */
export interface ZenohConfig {
  /**
   * This node's Zenoh ID, as an unsigned 128-bit integer in lowercase hexadecimal with
   * no leading zeros — at most 16 bytes, e.g. `"1234567890abcdef"`.
   *
   * Must be unique throughout the Zenoh infrastructure. A random one is generated when
   * left unset.
   */
  id?: string
  /**
   * Arbitrary JSON data (name, location, DNS name, …) that Zenoh does not interpret and
   * publishes in the admin space under `@/<zid>/<mode>`.
   */
  metadata?: JsonValue
  /**
   * The kind of node this session runs as.
   *
   * Defaults to `"peer"`.
   */
  mode?: binding.WhatAmI
  /**
   * North region name: a non-empty UTF-8 string of at most 32 bytes. Used during
   * establishment to assign remotes to a region through the gateway's
   * `south[].filters[].region_names` filter.
   */
  region_name?: string
  /**
   * A non-wildcard key expression prefixed onto every outgoing key expression of this
   * session, and stripped from every incoming one. With a namespace of `"1"`,
   * `session.put('my/keyexpr', …)` writes to `1/my/keyexpr`, and so on for every other
   * operation on the session.
   */
  namespace?: string
  /**
   * The default timeout applied to queries, in milliseconds.
   *
   * Defaults to `10000`.
   */
  queries_default_timeout?: number
  /** Which Zenoh nodes to connect to on session open. */
  connect?: ConnectConfig
  /** Which endpoints to listen on for incoming sessions. */
  listen?: ListenConfig
  /** What `Session.open` waits for before it resolves. */
  open?: OpenConfig
  /** How this node discovers the rest of the network. */
  scouting?: ScoutingConfig
  /** How this node timestamps the data messages passing through it. */
  timestamping?: TimestampingConfig
  /** Aggregation of subscriber and publisher declarations by key expression. */
  aggregation?: AggregationConfig
  /**
   * The routing strategy and its configuration.
   *
   * Zenoh still accepts the deprecated `router.peers_failover_brokering` and `peer`
   * keys here, but they have no effect and are omitted from this type.
   */
  routing?: RoutingConfig
  /** QoS overwrites by key expression, taking precedence over the QoS set in the API. */
  qos?: QoSConfig
  /** Internal transport parameters. */
  transport?: TransportConfig
  /** The admin space, which exposes this node's state and configuration. */
  adminspace?: AdminSpaceConfig
  /** Access control (ACL). */
  access_control?: AccessControlConfig
  /** Filters dropping the messages that exceed a size limit. */
  low_pass_filter?: LowPassFilterConfig[]
  /** Rules capping how often matching messages pass through. */
  downsampling?: DownsamplingConfig[]
  /** Per-key-expression statistics. */
  stats?: StatsConfig
  /** Gateway configuration. */
  gateway?: GatewayConfig
  /** Where plugin libraries are looked up. */
  plugins_loading?: PluginsLoadingConfig
  /**
   * Per-plugin configuration, keyed by the name the plugin is loaded under. Plugins are
   * only loaded when {@link PluginsLoadingConfig.enabled} is `true`; once loaded, they
   * may react to changes made through the admin space.
   */
  plugins?: Record<string, PluginConfig>
}
