/**
 * Exodus Browser — file transfer / workspace API tests.
 */
import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  exodusWorkspaceInfo,
  fileTransferDashboard,
  fileTransferInitiate,
  fileTransferList,
  fileTransferPickFile,
  fileTransferSetThrottle,
  fileTransferServiceStart,
  fileTransferStartBackgroundDownload,
} from './fileTransfer';

describe('fileTransfer API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('fileTransferServiceStart invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({
      root: '/data/ExodusWorkSpace',
      sharedDir: '/data/ExodusWorkSpace/shared',
      inboxDir: '/data/ExodusWorkSpace/inbox',
      outboxDir: '/data/ExodusWorkSpace/outbox',
      nodeId: 'exodus-abc',
      roomId: 'ExodusWorkSpace',
      meshHost: '192.168.1.2',
      meshPort: 9900,
      fileCount: 0,
    });
    const info = await fileTransferServiceStart();
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_service_start');
    expect(info.roomId).toBe('ExodusWorkSpace');
    expect(info.meshPort).toBe(9900);
  });

  it('fileTransferInitiate passes path', async () => {
    invokeMock.mockResolvedValueOnce({
      transferId: 't1',
      fileName: 'a.txt',
      fileSize: 10,
      fileType: 'text/plain',
      blobHash: 'h',
      chunkCount: 1,
      senderId: 'n',
      status: 'pending',
      createdAt: 1,
      retryCount: 0,
      roomId: 'ExodusWorkSpace',
    });
    await fileTransferInitiate('/tmp/a.txt', 'peer-1');
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_initiate', {
      filePath: '/tmp/a.txt',
      receiverId: 'peer-1',
    });
  });

  it('fileTransferList invokes backend', async () => {
    invokeMock.mockResolvedValueOnce([]);
    const list = await fileTransferList();
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_list');
    expect(list).toEqual([]);
  });

  it('exodusWorkspaceInfo invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({ roomId: 'ExodusWorkSpace', fileCount: 2 });
    const info = await exodusWorkspaceInfo();
    expect(invokeMock).toHaveBeenCalledWith('exodus_workspace_info');
    expect(info.fileCount).toBe(2);
  });

  it('fileTransferPickFile invokes backend', async () => {
    invokeMock.mockResolvedValueOnce('/tmp/picked.pdf');
    const path = await fileTransferPickFile();
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_pick_file');
    expect(path).toBe('/tmp/picked.pdf');
  });

  it('fileTransferDashboard invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({
      transfers: [],
      settings: {
        throttleBytesPerSec: 0,
        autoReconnect: true,
        backgroundJobs: true,
        workspaceWatchEnabled: true,
      },
      relayEnabled: false,
      workspaceWatchActive: true,
      activeBackgroundJobs: 0,
    });
    const dash = await fileTransferDashboard();
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_dashboard');
    expect(dash.workspaceWatchActive).toBe(true);
  });

  it('fileTransferSetThrottle passes bytes', async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    await fileTransferSetThrottle(1_000_000);
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_set_throttle', {
      bytesPerSec: 1_000_000,
    });
  });

  it('fileTransferStartBackgroundDownload invokes backend', async () => {
    invokeMock.mockResolvedValueOnce({
      transferId: 'd1',
      fileName: 'big.bin',
      fileSize: 100,
      status: 'pending',
      direction: 'download',
    });
    await fileTransferStartBackgroundDownload('deadbeef', 'big.bin');
    expect(invokeMock).toHaveBeenCalledWith('file_transfer_start_background_download', {
      contentHash: 'deadbeef',
      fileName: 'big.bin',
    });
  });
});
