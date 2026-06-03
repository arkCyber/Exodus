/**
 * Exodus Browser — P2P gossip topic helpers (presence, announcements).
 */
import { invoke } from '@tauri-apps/api/core';

export type GossipMessage = {
  topic: string;
  payload: Record<string, unknown>;
  from_node: string;
  timestamp: number;
  id: string;
};

export async function p2pGossipSubscribe(topic: string, subscriberId: string): Promise<void> {
  await invoke('p2p_gossip_subscribe', { topic, subscriberId });
}

export async function p2pGossipPublish(
  topic: string,
  payload: Record<string, unknown>
): Promise<string> {
  return invoke<string>('p2p_gossip_publish', { topic, payload });
}

export async function p2pGossipGetMessages(
  topic: string,
  limit = 200
): Promise<GossipMessage[]> {
  return invoke<GossipMessage[]>('p2p_gossip_get_messages', { topic, limit });
}
