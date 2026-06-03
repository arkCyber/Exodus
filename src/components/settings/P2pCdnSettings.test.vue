/**
 * Exodus Browser — P2pCdnSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import P2pCdnSettings from './P2pCdnSettings.vue';

vi.mock('$lib/p2p/cdn', () => ({
  p2pCdnJoinRoom: vi.fn(),
  p2pCdnRoomFeed: vi.fn(),
  p2pCdnStartMesh: vi.fn(),
  p2pCdnSyncGossip: vi.fn(),
  p2pCdnAnnounceGroupHot: vi.fn(),
  p2pCdnDownload: vi.fn()
}));

describe('P2pCdnSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders settings section', () => {
    const wrapper = mount(P2pCdnSettings);
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(P2pCdnSettings);
    
    expect(wrapper.find('h3').text()).toBe('P2P CDN');
  });

  it('renders room ID label', () => {
    const wrapper = mount(P2pCdnSettings);
    
    expect(wrapper.find('label').text()).toContain('Room ID');
  });

  it('renders room ID input', () => {
    const wrapper = mount(P2pCdnSettings);
    
    expect(wrapper.find('input[type="text"]').exists()).toBe(true);
  });

  it('uses default room ID when not provided', () => {
    const wrapper = mount(P2pCdnSettings);
    
    expect(wrapper.find('input[type="text"]').element.value).toBe('lobby');
  });

  it('uses provided room ID prop', () => {
    const wrapper = mount(P2pCdnSettings, {
      props: { roomId: 'custom-room' }
    });
    
    expect(wrapper.find('input[type="text"]').element.value).toBe('custom-room');
  });

  it('renders toolbar', () => {
    const wrapper = mount(P2pCdnSettings);
    
    expect(wrapper.find('.toolbar').exists()).toBe(true);
  });

  it('renders refresh button', () => {
    const wrapper = mount(P2pCdnSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].text()).toBe('Refresh feed');
  });

  it('renders announce button', () => {
    const wrapper = mount(P2pCdnSettings);
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[1].text()).toBe('Announce asset');
  });

  it('disables refresh button when loading', async () => {
    const { p2pCdnStartMesh } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
      return { nodeId: 'node-1' };
    });
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons[0].attributes('disabled')).toBeDefined();
  });

  it('displays node ID when available', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-123456789012' });
    p2pCdnRoomFeed.mockResolvedValue({ assets: [] });
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.hint').text()).toContain('Node node-123456789012');
  });

  it('displays asset count', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({ assets: [{ title: 'Asset1', contentHash: 'hash1' }] });
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.hint').text()).toContain('1 assets');
  });

  it('renders asset list when feed has assets', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({
      assets: [
        { title: 'Asset1', contentHash: 'hash1', roomId: 'lobby', kind: 'article', sourceUrl: '' }
      ]
    });
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.list').exists()).toBe(true);
  });

  it('displays asset title', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({
      assets: [
        { title: 'Test Asset', contentHash: 'hash1', roomId: 'lobby', kind: 'article', sourceUrl: '' }
      ]
    });
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row span').text()).toBe('Test Asset');
  });

  it('renders download button on asset', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({
      assets: [
        { title: 'Asset1', contentHash: 'hash1', roomId: 'lobby', kind: 'article', sourceUrl: '' }
      ]
    });
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.row .nav-button').text()).toBe('Download');
  });

  it('downloads asset on download button click', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed, p2pCdnDownload } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({
      assets: [
        { title: 'Asset1', contentHash: 'hash1', roomId: 'lobby', kind: 'article', sourceUrl: '' }
      ]
    });
    p2pCdnDownload.mockResolvedValue(undefined);
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(p2pCdnDownload).toHaveBeenCalled();
  });

  it('emits status on successful download', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed, p2pCdnDownload } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({
      assets: [
        { title: 'Asset1', contentHash: 'hash1', roomId: 'lobby', kind: 'article', sourceUrl: '' }
      ]
    });
    p2pCdnDownload.mockResolvedValue(undefined);
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Downloaded Asset1']);
  });

  it('emits status on failed download', async () => {
    const { p2pCdnStartMesh, p2pCdnRoomFeed, p2pCdnDownload } = require('$lib/p2p/cdn');
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnRoomFeed.mockResolvedValue({
      assets: [
        { title: 'Asset1', contentHash: 'hash1', roomId: 'lobby', kind: 'article', sourceUrl: '' }
      ]
    });
    p2pCdnDownload.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.row .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Download failed']);
  });

  it('shows announce form on announce button click', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.announce').exists()).toBe(true);
  });

  it('hides announce form on second click', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.announce').exists()).toBe(false);
  });

  it('renders announce title input', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    expect(inputs[0].attributes('placeholder')).toBe('Title');
  });

  it('renders announce hash input', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    expect(inputs[1].attributes('placeholder')).toBe('Content hash (BLAKE3)');
  });

  it('renders announce URL input', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    expect(inputs[2].attributes('placeholder')).toBe('HTTP URL (optional)');
  });

  it('renders publish button in announce form', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.announce .nav-button');
    expect(buttons[0].text()).toBe('Publish');
  });

  it('announces asset on publish', async () => {
    const { p2pCdnAnnounceGroupHot } = require('$lib/p2p/cdn');
    p2pCdnAnnounceGroupHot.mockResolvedValue(undefined);
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    await inputs[0].setValue('Test Title');
    await inputs[1].setValue('hash123');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(p2pCdnAnnounceGroupHot).toHaveBeenCalled();
  });

  it('does not announce with empty title', async () => {
    const { p2pCdnAnnounceGroupHot } = require('$lib/p2p/cdn');
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    
    expect(p2pCdnAnnounceGroupHot).not.toHaveBeenCalled();
  });

  it('does not announce with empty hash', async () => {
    const { p2pCdnAnnounceGroupHot } = require('$lib/p2p/cdn');
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    await inputs[0].setValue('Test Title');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    
    expect(p2pCdnAnnounceGroupHot).not.toHaveBeenCalled();
  });

  it('emits status on validation error', async () => {
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Title and content hash required']);
  });

  it('emits status on successful announce', async () => {
    const { p2pCdnAnnounceGroupHot } = require('$lib/p2p/cdn');
    p2pCdnAnnounceGroupHot.mockResolvedValue(undefined);
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    await inputs[0].setValue('Test Title');
    await inputs[1].setValue('hash123');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Asset announced']);
  });

  it('hides announce form after successful announce', async () => {
    const { p2pCdnAnnounceGroupHot } = require('$lib/p2p/cdn');
    p2pCdnAnnounceGroupHot.mockResolvedValue(undefined);
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    await inputs[0].setValue('Test Title');
    await inputs[1].setValue('hash123');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.announce').exists()).toBe(false);
  });

  it('emits status on failed announce', async () => {
    const { p2pCdnAnnounceGroupHot } = require('$lib/p2p/cdn');
    p2pCdnAnnounceGroupHot.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(P2pCdnSettings);
    
    await wrapper.findAll('.nav-button')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    const inputs = wrapper.findAll('.announce .field');
    await inputs[0].setValue('Test Title');
    await inputs[1].setValue('hash123');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.announce .nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Announce failed']);
  });

  it('refreshes on refresh button click', async () => {
    const { p2pCdnJoinRoom, p2pCdnStartMesh, p2pCdnSyncGossip, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnJoinRoom.mockResolvedValue(undefined);
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnSyncGossip.mockResolvedValue(undefined);
    p2pCdnRoomFeed.mockResolvedValue({ assets: [] });
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(p2pCdnJoinRoom).toHaveBeenCalled();
  });

  it('loads data on mount', async () => {
    const { p2pCdnJoinRoom, p2pCdnStartMesh, p2pCdnSyncGossip, p2pCdnRoomFeed } = require('$lib/p2p/cdn');
    p2pCdnJoinRoom.mockResolvedValue(undefined);
    p2pCdnStartMesh.mockResolvedValue({ nodeId: 'node-1' });
    p2pCdnSyncGossip.mockResolvedValue(undefined);
    p2pCdnRoomFeed.mockResolvedValue({ assets: [] });
    
    mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(p2pCdnJoinRoom).toHaveBeenCalled();
    expect(p2pCdnStartMesh).toHaveBeenCalled();
    expect(p2pCdnSyncGossip).toHaveBeenCalled();
    expect(p2pCdnRoomFeed).toHaveBeenCalled();
  });

  it('emits status on refresh error', async () => {
    const { p2pCdnJoinRoom } = require('$lib/p2p/cdn');
    p2pCdnJoinRoom.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(P2pCdnSettings);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['P2P CDN feed failed']);
  });
});
