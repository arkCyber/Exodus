import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('$lib/fileTransfer', () => ({
  fileTransferDashboard: vi.fn(async () => ({
    transfers: [],
    settings: {
      throttleBytesPerSec: 0,
      autoReconnect: true,
      backgroundJobs: false,
      workspaceWatchEnabled: false,
    },
    relayEnabled: false,
    workspaceWatchActive: false,
    activeBackgroundJobs: 0,
  })),
  exodusWorkspaceInfo: vi.fn(async () => ({
    root: '/tmp',
    sharedDir: '/tmp/shared',
    inboxDir: '/tmp/in',
    outboxDir: '/tmp/out',
    nodeId: 'node-1',
    roomId: 'lobby',
    fileCount: 0,
  })),
  exodusWorkspaceList: vi.fn(async () => []),
  wanRelayServerInfo: vi.fn(async () => ({ running: false, port: 8790, baseUrl: '', bindHost: '127.0.0.1' })),
  fileTransferList: vi.fn(async () => []),
}));

import FileTransfer from './FileTransfer.vue';

describe('FileTransfer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders ExodusWorkSpace header', async () => {
    const wrapper = mount(FileTransfer);
    await flushPromises();
    expect(wrapper.text()).toContain('ExodusWorkSpace');
    expect(wrapper.text()).toContain('Transfers');
  });

  it('switches to workspace tab', async () => {
    const wrapper = mount(FileTransfer);
    await flushPromises();
    const tabs = wrapper.findAll('.tab-btn');
    await tabs[1].trigger('click');
    expect(wrapper.text()).toContain('Workspace Files');
  });
});
