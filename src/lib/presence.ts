/**
 * Exodus Browser — lightweight online presence via gossip heartbeats.
 */
import { p2pGossipGetMessages, p2pGossipPublish, p2pGossipSubscribe } from '$lib/p2pGossip';

export const PRESENCE_TOPIC = 'exodus-presence';

/** Peers seen within this window are marked online. */
export const PRESENCE_TTL_MS = 90_000;

export type PresenceEntry = {
  nodeId: string;
  displayName: string;
  lastSeen: number;
};

let heartbeatTimer: ReturnType<typeof setInterval> | null = null;

/**
 * Publish periodic heartbeats and subscribe to the presence topic.
 */
export async function startPresenceHeartbeat(
  nodeId: string,
  displayName: string,
  intervalMs = 25_000
): Promise<void> {
  if (typeof window === 'undefined') return;
  stopPresenceHeartbeat();
  await p2pGossipSubscribe(PRESENCE_TOPIC, nodeId);
  const beat = async () => {
    try {
      await p2pGossipPublish(PRESENCE_TOPIC, {
        nodeId,
        displayName,
        ts: Date.now(),
      });
    } catch {
      /* gossip may be unavailable until mesh starts */
    }
  };
  await beat();
  heartbeatTimer = setInterval(() => void beat(), intervalMs);
}

export function stopPresenceHeartbeat(): void {
  if (heartbeatTimer) {
    clearInterval(heartbeatTimer);
    heartbeatTimer = null;
  }
}

/**
 * Merge recent gossip presence messages into a nodeId → entry map.
 */
export async function fetchOnlinePeers(excludeNodeId?: string): Promise<Map<string, PresenceEntry>> {
  const now = Date.now();
  const map = new Map<string, PresenceEntry>();
  try {
    const msgs = await p2pGossipGetMessages(PRESENCE_TOPIC, 300);
    for (const m of msgs) {
      const p = m.payload as { nodeId?: string; displayName?: string; ts?: number };
      const nodeId = typeof p.nodeId === 'string' ? p.nodeId : m.from_node;
      if (!nodeId || nodeId === excludeNodeId) continue;
      const ts = typeof p.ts === 'number' ? p.ts : m.timestamp * 1000;
      if (now - ts > PRESENCE_TTL_MS) continue;
      const prev = map.get(nodeId);
      if (!prev || ts > prev.lastSeen) {
        map.set(nodeId, {
          nodeId,
          displayName: typeof p.displayName === 'string' ? p.displayName : nodeId.slice(0, 12),
          lastSeen: ts,
        });
      }
    }
  } catch {
    /* ignore */
  }
  return map;
}

export function isNodeOnline(map: Map<string, PresenceEntry>, nodeId: string): boolean {
  return map.has(nodeId);
}
