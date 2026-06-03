/**
 * Exodus Browser — VideoCall component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import VideoCall from './VideoCall.vue';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => vi.fn())
}));

vi.mock('$lib/webrtc/rtcCall', () => ({
  RtcOneToOneCall: vi.fn().mockImplementation(() => ({
    start: vi.fn(),
    accept: vi.fn(),
    stop: vi.fn()
  }))
}));

vi.mock('$lib/videoRtc', () => ({
  videoRtcCallStart: vi.fn(),
  videoRtcCallUpdate: vi.fn(),
  videoRtcNodeInfo: vi.fn(),
  videoRtcServiceStart: vi.fn()
}));

describe('VideoCall', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders video call component', () => {
    const wrapper = mount(VideoCall);
    
    expect(wrapper.find('.video-call').exists()).toBe(true);
  });

  it('renders header with title', () => {
    const wrapper = mount(VideoCall);
    
    expect(wrapper.find('.header h2').text()).toBe('Call');
  });

  it('renders status message', () => {
    const wrapper = mount(VideoCall);
    
    expect(wrapper.find('.hint').exists()).toBe(true);
  });

  it('renders node ID when available', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-12345678901234567890' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.node-id').exists()).toBe(true);
  });

  it('displays truncated node ID', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-12345678901234567890' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.node-id').text()).toContain('node-12345678901234567890'.slice(0, 24));
  });

  it('renders new call button when idle', () => {
    const wrapper = mount(VideoCall);
    
    const buttons = wrapper.findAll('.btn-primary');
    expect(buttons[0].text()).toBe('New call');
  });

  it('shows dial dialog on new call button click', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.overlay').exists()).toBe(true);
  });

  it('hides dial dialog on overlay click', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.overlay').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.overlay').exists()).toBe(false);
  });

  it('has correct ARIA role on dialog', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog').attributes('role')).toBe('dialog');
  });

  it('renders dial dialog title', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.dialog h3').text()).toBe('Call peer');
  });

  it('renders remote node ID input', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('input[type="text"]').exists()).toBe(true);
  });

  it('renders video checkbox', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[0].exists()).toBe(true);
  });

  it('renders audio checkbox', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes[1].exists()).toBe(true);
  });

  it('renders cancel button in dial dialog', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.btn-secondary');
    expect(buttons[0].text()).toBe('Cancel');
  });

  it('hides dialog on cancel button click', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.btn-secondary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.overlay').exists()).toBe(false);
  });

  it('renders call button in dial dialog', async () => {
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.dialog .btn-primary');
    expect(buttons[0].text()).toBe('Call');
  });

  it('dials peer on call button click', async () => {
    const { videoRtcCallStart, videoRtcServiceStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcCallStart).toHaveBeenCalled();
  });

  it('does not dial with empty node ID', async () => {
    const { videoRtcCallStart } = require('$lib/videoRtc');
    
    const wrapper = mount(VideoCall);
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    
    expect(videoRtcCallStart).not.toHaveBeenCalled();
  });

  it('emits status on dial', async () => {
    const { videoRtcCallStart, videoRtcServiceStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('renders idle state when not in call', () => {
    const wrapper = mount(VideoCall);
    
    expect(wrapper.find('.idle').exists()).toBe(true);
    expect(wrapper.find('.idle').text()).toContain('Ready — share your node ID');
  });

  it('renders connecting state when connecting', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
      return { sessionId: 'session-1', calleeNode: 'node-2', calleeName: 'Peer' };
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.connecting').exists()).toBe(true);
  });

  it('renders ringing state when ringing', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(() => {
        // Simulate state change to ringing
      }),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.connecting').text()).toContain('Ringing');
  });

  it('renders video grid when connected', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    const callMock = {
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    };
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => callMock);
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Manually set connected state
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.video-grid').exists()).toBe(true);
  });

  it('renders local video element', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.tile.local').exists()).toBe(true);
  });

  it('renders remote video element', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.tile.remote').exists()).toBe(true);
  });

  it('renders control bar when connected', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.bar').exists()).toBe(true);
  });

  it('displays remote name in control bar', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.bar').text()).toContain('Peer');
  });

  it('displays call duration', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    wrapper.vm.duration = 65;
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.bar').text()).toContain('1:05');
  });

  it('renders hang up button when in call', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.btn-danger').text()).toBe('Hang up');
  });

  it('ends call on hang up button click', async () => {
    const { videoRtcServiceStart, videoRtcCallStart, videoRtcCallUpdate } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    const callMock = {
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    };
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => callMock);
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-danger').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(callMock.stop).toHaveBeenCalled();
  });

  it('emits status on end call', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-danger').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Call ended']);
  });

  it('toggles audio on mute button click', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.icon')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.useAudio).toBe(false);
  });

  it('toggles video on video button click', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    await wrapper.findAll('.icon')[1].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.useVideo).toBe(false);
  });

  it('shows incoming call dialog', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    wrapper.vm.incomingFrom = 'Peer';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.overlay.incoming').exists()).toBe(true);
  });

  it('renders incoming call dialog title', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.incoming .dialog h3').text()).toBe('Incoming call');
  });

  it('displays incoming caller name', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    wrapper.vm.incomingFrom = 'Peer';
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.incoming .dialog p').text()).toBe('Peer');
  });

  it('renders decline button in incoming dialog', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.incoming .btn-danger').text()).toBe('Decline');
  });

  it('renders accept button in incoming dialog', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.incoming .btn-primary').text()).toBe('Accept');
  });

  it('rejects incoming call on decline', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.incoming .btn-danger').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.vm.showIncoming).toBe(false);
  });

  it('accepts incoming call on accept', async () => {
    const { videoRtcServiceStart, videoRtcCallUpdate } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    RtcOneToOneCall.mockImplementation(() => ({
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    }));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.vm.showIncoming = true;
    wrapper.vm.incomingSession = 'session-1';
    wrapper.vm.incomingFrom = 'Peer';
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.incoming .btn-primary').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.vm.showIncoming).toBe(false);
  });

  it('initializes RTC service on mount', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcServiceStart).toHaveBeenCalled();
  });

  it('falls back to node info if service start fails', async () => {
    const { videoRtcServiceStart, videoRtcNodeInfo } = require('$lib/videoRtc');
    videoRtcServiceStart.mockRejectedValue(new Error('Service error'));
    videoRtcNodeInfo.mockResolvedValue({ nodeId: 'node-1' });
    
    mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcNodeInfo).toHaveBeenCalled();
  });

  it('emits status on error', async () => {
    const { videoRtcCallStart, videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockRejectedValue(new Error('Call failed'));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('renders error state on error', async () => {
    const { videoRtcCallStart, videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockRejectedValue(new Error('Call failed'));
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.error').exists()).toBe(true);
  });

  it('cleans up on unmount', async () => {
    const { videoRtcServiceStart, videoRtcCallStart } = require('$lib/videoRtc');
    const { RtcOneToOneCall } = require('$lib/webrtc/rtcCall');
    const callMock = {
      start: vi.fn(),
      accept: vi.fn(),
      stop: vi.fn()
    };
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcCallStart.mockResolvedValue({
      sessionId: 'session-1',
      calleeNode: 'node-2',
      calleeName: 'Peer'
    });
    RtcOneToOneCall.mockImplementation(() => callMock);
    
    const wrapper = mount(VideoCall);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    await wrapper.find('input[type="text"]').setValue('node-2');
    await wrapper.findAll('.dialog .btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    wrapper.vm.callState = 'connected';
    await wrapper.vm.$nextTick();
    
    wrapper.unmount();
    
    expect(callMock.stop).toHaveBeenCalled();
  });
});
