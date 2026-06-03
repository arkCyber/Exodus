/**
 * Exodus Browser — P2P CDN API and helper tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

import {
  cdnSourceLabel,
  listenP2pCdnProgress,
  p2pCdnAnnounceFromAi,
  p2pCdnAnnounceGroupHot,
  p2pCdnDownload,
  p2pCdnHashFile,
  p2pCdnJoinRoom,
  p2pCdnRoomFeed,
  p2pCdnStartMesh,
  p2pCdnSyncGossip,
  p2pCdnAnnounceUrlHot,
  p2pCdnGroupSendMessage,
} from './cdn';

describe('p2p cdn helpers', () => {
  it('cdnSourceLabel maps p2p peers', () => {
    expect(cdnSourceLabel('p2p_peers', 3)).toBe('P2P · 3 peer(s)');
  });

  it('cdnSourceLabel maps http seed', () => {
    expect(cdnSourceLabel('http_then_seed', 0)).toBe('HTTP → now seeding swarm');
  });

  it('cdnSourceLabel maps local cache', () => {
    expect(cdnSourceLabel('local_cache', 0)).toBe('Local cache');
  });
});

describe('p2p cdn api', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it('p2pCdnStartMesh invokes start_mesh', async () => {
    invokeMock.mockResolvedValue({
      nodeId: 'exodus-abc',
      joinedRooms: ['lobby'],
      meshHost: '127.0.0.1',
      meshPort: 7878,
    });
    const info = await p2pCdnStartMesh();
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_start_mesh');
    expect(info.meshPort).toBe(7878);
  });

  it('p2pCdnJoinRoom passes roomId', async () => {
    await p2pCdnJoinRoom('group-42');
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_join_room', { roomId: 'group-42' });
  });

  it('p2pCdnRoomFeed returns feed', async () => {
    invokeMock.mockResolvedValue({
      roomId: 'lobby',
      assets: [],
      peerMap: {},
    });
    const feed = await p2pCdnRoomFeed('lobby');
    expect(feed.roomId).toBe('lobby');
  });

  it('p2pCdnAnnounceFromAi sends payload', async () => {
    await p2pCdnAnnounceFromAi('lobby', {
      contentHash: 'deadbeef',
      title: 'Llama',
      kind: 'ai_model',
      sizeBytes: 1000,
    });
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_announce_asset', {
      roomId: 'lobby',
      payload: expect.objectContaining({ contentHash: 'deadbeef' }),
    });
  });

  it('p2pCdnAnnounceGroupHot passes group fields', async () => {
    await p2pCdnAnnounceGroupHot({
      groupId: 'g1',
      title: 'Video model',
      contentHash: 'abc',
      kind: 'video_model',
      sizeBytes: 2_000_000,
    });
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_announce_group_hot', {
      groupId: 'g1',
      title: 'Video model',
      contentHash: 'abc',
      kind: 'video_model',
      sizeBytes: 2_000_000,
      sourceUrl: null,
      localPath: null,
    });
  });

  it('p2pCdnDownload passes download params', async () => {
    invokeMock.mockResolvedValue({
      jobId: 'cdn-1',
      contentHash: 'h1',
      title: 'File',
      status: 'completed',
      progressPercent: 100,
      bytesDone: 10,
      bytesTotal: 10,
      source: 'p2p_peers',
      peerCount: 2,
    });
    const job = await p2pCdnDownload({
      roomId: 'lobby',
      contentHash: 'h1',
      title: 'File',
      kind: 'article',
      httpUrl: 'https://example.com/f',
    });
    expect(job.source).toBe('p2p_peers');
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_download', {
      roomId: 'lobby',
      contentHash: 'h1',
      title: 'File',
      kind: 'article',
      httpUrl: 'https://example.com/f',
    });
  });

  it('p2pCdnHashFile invokes hash_file', async () => {
    invokeMock.mockResolvedValue({ contentHash: 'abc', sizeBytes: 42 });
    const out = await p2pCdnHashFile('/tmp/model.bin');
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_hash_file', { localPath: '/tmp/model.bin' });
    expect(out.sizeBytes).toBe(42);
  });

  it('p2pCdnSyncGossip invokes sync_gossip', async () => {
    invokeMock.mockResolvedValue(3);
    const n = await p2pCdnSyncGossip('lobby');
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_sync_gossip', { roomId: 'lobby' });
    expect(n).toBe(3);
  });

  it('p2pCdnAnnounceUrlHot invokes announce_url_hot', async () => {
    await p2pCdnAnnounceUrlHot({
      roomId: 'lobby',
      title: 'Big article',
      url: 'https://example.com/long',
      kind: 'article',
      sizeBytes: 600_000,
    });
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_announce_url_hot', {
      roomId: 'lobby',
      title: 'Big article',
      url: 'https://example.com/long',
      kind: 'article',
      sizeBytes: 600_000,
    });
  });

  it('p2pCdnGroupSendMessage invokes group send', async () => {
    invokeMock.mockResolvedValue('msg-1');
    const id = await p2pCdnGroupSendMessage({ message_id: 'm1', group_id: 'g1' });
    expect(id).toBe('msg-1');
    expect(invokeMock).toHaveBeenCalledWith('p2p_cdn_group_send_message', {
      message: { message_id: 'm1', group_id: 'g1' },
    });
  });

  it('listenP2pCdnProgress registers event', async () => {
    const { listen } = await import('@tauri-apps/api/event');
    const fn = vi.mocked(listen);
    fn.mockResolvedValue(() => {});
    await listenP2pCdnProgress(() => {});
    expect(fn).toHaveBeenCalledWith('exodus-p2p-cdn-progress', expect.any(Function));
  });
});
