/**
 * Exodus Browser — FileTransfer component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import FileTransfer from './FileTransfer.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => true)
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => vi.fn())
}));

vi.mock('$lib/fileTransfer', () => ({
  exodusWorkspaceInfo: vi.fn(),
  exodusWorkspaceList: vi.fn(),
  exodusWorkspaceWatchStart: vi.fn(),
  exodusWorkspaceWatchStop: vi.fn(),
  fileTransferDashboard: vi.fn(),
  fileTransferInitiate: vi.fn(),
  fileTransferList: vi.fn(),
  fileTransferPickFile: vi.fn(),
  fileTransferServiceStart: vi.fn(),
  fileTransferSetAutoReconnect: vi.fn(),
  fileTransferSetRelayConfig: vi.fn(),
  fileTransferSetRelayServe: vi.fn(),
  fileTransferSetThrottle: vi.fn(),
  fileTransferStartBackgroundDownload: vi.fn(),
  fileTransferVerifyChecksum: vi.fn(),
  wanRelayServerInfo: vi.fn()
}));

describe('FileTransfer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders file transfer component', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.file-transfer').exists()).toBe(true);
  });

  it('renders header with title', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.header h2').text()).toBe('ExodusWorkSpace');
  });

  it('renders dashboard bar', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.dashboard-bar').exists()).toBe(true);
  });

  it('renders throttle input', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('input[type="number"]').exists()).toBe(true);
  });

  it('renders auto-reconnect checkbox', () => {
    const wrapper = mount(FileTransfer);
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[0].exists()).toBe(true);
  });

  it('renders workspace watch checkbox', () => {
    const wrapper = mount(FileTransfer);
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[1].exists()).toBe(true);
  });

  it('renders relay bar', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.relay-bar').exists()).toBe(true);
  });

  it('renders relay enabled checkbox', () => {
    const wrapper = mount(FileTransfer);
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[2].exists()).toBe(true);
  });

  it('renders relay base URL input', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.relay-input').exists()).toBe(true);
  });

  it('renders tabs', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.tabs').exists()).toBe(true);
  });

  it('renders transfer tab button', () => {
    const wrapper = mount(FileTransfer);
    
    const tabButtons = wrapper.findAll('.tab-btn');
    expect(tabButtons[0].text()).toBe('Transfers');
  });

  it('renders workspace tab button', () => {
    const wrapper = mount(FileTransfer);
    
    const tabButtons = wrapper.findAll('.tab-btn');
    expect(tabButtons[1].text()).toContain('Workspace Files');
  });

  it('applies active class to transfers tab by default', () => {
    const wrapper = mount(FileTransfer);
    
    const tabButtons = wrapper.findAll('.tab-btn');
    expect(tabButtons[0].classes()).toContain('active');
  });

  it('switches to workspace tab on click', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const tabButtons = wrapper.findAll('.tab-btn');
    expect(tabButtons[1].classes()).toContain('active');
  });

  it('renders actions bar when transfers tab is active', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.actions').exists()).toBe(true);
  });

  it('renders search input', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.search-input').exists()).toBe(true);
  });

  it('renders filter select', () => {
    const wrapper = mount(FileTransfer);
    
    expect(wrapper.find('.filter-select').exists()).toBe(true);
  });

  it('renders upload button', () => {
    const wrapper = mount(FileTransfer);
    
    const buttons = wrapper.findAll('.btn-primary');
    expect(buttons[0].text()).toBe('Upload File');
  });

  it('renders download button', () => {
    const wrapper = mount(FileTransfer);
    
    const buttons = wrapper.findAll('.btn-secondary');
    expect(buttons.some(b => b.text() === 'Download from Peer')).toBe(true);
  });

  it('shows upload dialog on upload button click', async () => {
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(true);
  });

  it('shows download dialog on download button click', async () => {
    const wrapper = mount(FileTransfer);
    
    const downloadButton = wrapper.findAll('.btn-secondary').find(b => b.text() === 'Download from Peer');
    await downloadButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.dialog-overlay')[1].exists()).toBe(true);
  });

  it('hides upload dialog on overlay click', async () => {
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.dialog-overlay').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog-overlay').exists()).toBe(false);
  });

  it('hides download dialog on overlay click', async () => {
    const wrapper = mount(FileTransfer);
    
    const downloadButton = wrapper.findAll('.btn-secondary').find(b => b.text() === 'Download from Peer');
    await downloadButton.trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.dialog-overlay')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.dialog-overlay')[1].exists()).toBe(false);
  });

  it('has correct ARIA attributes on dialogs', async () => {
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog').attributes('role')).toBe('dialog');
    expect(wrapper.find('.dialog').attributes('aria-modal')).toBe('true');
  });

  it('renders upload dialog title', async () => {
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h3').text()).toBe('Upload File');
  });

  it('renders download dialog title', async () => {
    const wrapper = mount(FileTransfer);
    
    const downloadButton = wrapper.findAll('.btn-secondary').find(b => b.text() === 'Download from Peer');
    await downloadButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.dialog h3')[1].text()).toBe('Background download (resume + checksum)');
  });

  it('renders empty state when no transfers', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.empty-state').exists()).toBe(true);
    expect(wrapper.find('.empty-state').text()).toContain('No transfers found');
  });

  it('renders transfer items', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const transfers = wrapper.findAll('.transfer-item');
    expect(transfers.length).toBe(1);
  });

  it('displays transfer icon based on direction', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.transfer-icon').text()).toBe('⬆️');
  });

  it('displays download icon for downloads', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'download',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.transfer-icon').text()).toBe('⬇️');
  });

  it('displays transfer name', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.transfer-info .name').text()).toBe('test.txt');
  });

  it('displays transfer size', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.transfer-info .size').text()).toBe('1.00 KB');
  });

  it('displays transfer speed when transferring', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'transferring',
          progressPercent: 50,
          speedBps: 1024 * 1024,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.transfer-info .speed').text()).toContain('1.00 MB/s');
  });

  it('displays transfer status', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.transfer-info .status').text()).toBe('completed');
  });

  it('renders progress bar', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'transferring',
          progressPercent: 50,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.progress-bar').exists()).toBe(true);
  });

  it('renders progress text', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'transferring',
          progressPercent: 50,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.progress-text').text()).toBe('50');
  });

  it('renders cancel button for active transfers', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'transferring',
          progressPercent: 50,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.btn-icon').text()).toBe('❌');
  });

  it('renders verify button for completed transfers', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.btn-icon').text()).toBe('✓');
  });

  it('filters transfers by direction', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        },
        {
          transferId: '2',
          fileName: 'test2.txt',
          fileSize: 2048,
          direction: 'download',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-2'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.filter-select').setValue('upload');
    await wrapper.vm.$nextTick();
    
    const transfers = wrapper.findAll('.transfer-item');
    expect(transfers.length).toBe(1);
  });

  it('filters transfers by search query', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [
        {
          transferId: '1',
          fileName: 'test.txt',
          fileSize: 1024,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-1'
        },
        {
          transferId: '2',
          fileName: 'other.txt',
          fileSize: 2048,
          direction: 'upload',
          status: 'completed',
          progressPercent: 100,
          speedBps: 0,
          createdAt: Date.now(),
          senderId: 'peer-2'
        }
      ],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.search-input').setValue('test');
    await wrapper.vm.$nextTick();
    
    const transfers = wrapper.findAll('.transfer-item');
    expect(transfers.length).toBe(1);
  });

  it('renders workspace files when workspace tab is active', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([
      { name: 'file.txt', relativePath: '/file.txt', sizeBytes: 1024, contentHash: 'hash123' }
    ]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const files = wrapper.findAll('.file-item');
    expect(files.length).toBe(1);
  });

  it('displays workspace file name', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([
      { name: 'file.txt', relativePath: '/file.txt', sizeBytes: 1024, contentHash: 'hash123' }
    ]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.file-info .name').text()).toBe('file.txt');
  });

  it('displays workspace file path', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([
      { name: 'file.txt', relativePath: '/folder/file.txt', sizeBytes: 1024, contentHash: 'hash123' }
    ]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.file-info .path').text()).toBe('/folder/file.txt');
  });

  it('displays workspace file size', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([
      { name: 'file.txt', relativePath: '/file.txt', sizeBytes: 1024, contentHash: 'hash123' }
    ]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.file-info .size').text()).toBe('1.00 KB');
  });

  it('displays truncated hash', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([
      { name: 'file.txt', relativePath: '/file.txt', sizeBytes: 1024, contentHash: 'abcdef1234567890abcdef1234567890abcdef12' }
    ]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.file-info .hash').text()).toBe('Hash: abcdef1234567890…');
  });

  it('shows empty state when no workspace files', async () => {
    const { exodusWorkspaceList } = require('$lib/fileTransfer');
    exodusWorkspaceList.mockResolvedValue([]);
    
    const wrapper = mount(FileTransfer);
    
    await wrapper.findAll('.tab-btn')[1].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.empty-state').text()).toContain('No files in workspace');
  });

  it('loads dashboard on mount', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockResolvedValue({
      transfers: [],
      settings: { throttleBytesPerSec: 0, autoReconnect: true },
      relayEnabled: false,
      workspaceWatchActive: false,
      activeBackgroundJobs: 0
    });
    
    mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(fileTransferDashboard).toHaveBeenCalled();
  });

  it('emits status on error', async () => {
    const { fileTransferDashboard } = require('$lib/fileTransfer');
    fileTransferDashboard.mockRejectedValue(new Error('Failed to load'));
    
    const wrapper = mount(FileTransfer);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });
});
