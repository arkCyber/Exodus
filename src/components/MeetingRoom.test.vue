/**
 * Exodus Browser — MeetingRoom component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import MeetingRoom from './MeetingRoom.vue';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => vi.fn())
}));

vi.mock('$lib/webrtc/rtcMeeting', () => ({
  RtcMeetingMesh: vi.fn().mockImplementation(() => ({
    start: vi.fn(),
    leave: vi.fn()
  }))
}));

vi.mock('$lib/videoRtc', () => ({
  videoRtcMeetingCreate: vi.fn(),
  videoRtcMeetingJoin: vi.fn(),
  videoRtcMeetingLeave: vi.fn(),
  videoRtcMeetingList: vi.fn(),
  videoRtcNodeInfo: vi.fn(),
  videoRtcServiceStart: vi.fn()
}));

describe('MeetingRoom', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders meeting room component', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.meeting-room').exists()).toBe(true);
  });

  it('renders header with title', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('header h2').text()).toBe('Meeting');
  });

  it('renders status message', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.hint').exists()).toBe(true);
  });

  it('renders node ID when available', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-12345678901234567890' });
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.node-id').exists()).toBe(true);
  });

  it('displays truncated node ID', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-12345678901234567890' });
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.node-id').text()).toContain('node-12345678901234567890'.slice(0, 20));
  });

  it('renders create section when not in meeting', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.create').exists()).toBe(true);
  });

  it('renders meeting title input', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.create input').exists()).toBe(true);
    expect(wrapper.find('.create input').attributes('placeholder')).toBe('Meeting title');
  });

  it('renders create room button', () => {
    const wrapper = mount(MeetingRoom);
    
    const buttons = wrapper.findAll('.btn-primary');
    expect(buttons[0].text()).toBe('Create room');
  });

  it('renders join section when not in meeting', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.join').exists()).toBe(true);
  });

  it('renders join ID input', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.join input').exists()).toBe(true);
    expect(wrapper.find('.join input').attributes('placeholder')).toBe('Meeting ID (mtg-…)');
  });

  it('renders join button', () => {
    const wrapper = mount(MeetingRoom);
    
    const buttons = wrapper.findAll('.btn-secondary');
    expect(buttons[0].text()).toBe('Join');
  });

  it('renders list section when not in meeting', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.list').exists()).toBe(true);
  });

  it('renders active rooms header', () => {
    const wrapper = mount(MeetingRoom);
    
    expect(wrapper.find('.list h3').text()).toBe('Active rooms');
  });

  it('shows empty state when no rooms', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([]);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.empty').text()).toBe('No active meetings');
  });

  it('renders room cards', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([
      { meetingId: 'mtg-1', title: 'Test Meeting', participants: ['peer-1'], maxParticipants: 6 }
    ]);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const rooms = wrapper.findAll('.room-card');
    expect(rooms.length).toBe(1);
  });

  it('displays room title', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([
      { meetingId: 'mtg-1', title: 'Test Meeting', participants: ['peer-1'], maxParticipants: 6 }
    ]);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.room-card strong').text()).toBe('Test Meeting');
  });

  it('displays room ID', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([
      { meetingId: 'mtg-1', title: 'Test Meeting', participants: ['peer-1'], maxParticipants: 6 }
    ]);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.room-card').text()).toContain('mtg-1');
  });

  it('displays participant count', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([
      { meetingId: 'mtg-1', title: 'Test Meeting', participants: ['peer-1', 'peer-2'], maxParticipants: 6 }
    ]);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.room-card').text()).toContain('2/6');
  });

  it('renders join button on room card', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([
      { meetingId: 'mtg-1', title: 'Test Meeting', participants: ['peer-1'], maxParticipants: 6 }
    ]);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.room-card .btn-secondary');
    expect(buttons[0].text()).toBe('Join');
  });

  it('creates meeting on create button click', async () => {
    const { videoRtcMeetingCreate } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcMeetingCreate).toHaveBeenCalledWith('Test Meeting', 6);
  });

  it('does not create meeting with empty title', async () => {
    const { videoRtcMeetingCreate } = require('$lib/videoRtc');
    
    const wrapper = mount(MeetingRoom);
    
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    
    expect(videoRtcMeetingCreate).not.toHaveBeenCalled();
  });

  it('emits status on meeting creation', async () => {
    const { videoRtcMeetingCreate, videoRtcServiceStart } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('joins meeting on join button click', async () => {
    const { videoRtcMeetingJoin, videoRtcServiceStart } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingJoin.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.join input').setValue('mtg-1');
    await wrapper.findAll('.btn-secondary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcMeetingJoin).toHaveBeenCalledWith('mtg-1');
  });

  it('does not join meeting with empty ID', async () => {
    const { videoRtcMeetingJoin } = require('$lib/videoRtc');
    
    const wrapper = mount(MeetingRoom);
    
    await wrapper.findAll('.btn-secondary')[0].trigger('click');
    
    expect(videoRtcMeetingJoin).not.toHaveBeenCalled();
  });

  it('joins room from room card', async () => {
    const { videoRtcMeetingList, videoRtcMeetingJoin, videoRtcServiceStart } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingList.mockResolvedValue([
      { meetingId: 'mtg-1', title: 'Test Meeting', participants: ['peer-1'], maxParticipants: 6 }
    ]);
    videoRtcMeetingJoin.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.room-card .btn-secondary').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcMeetingJoin).toHaveBeenCalledWith('mtg-1');
  });

  it('renders conference view when in meeting', async () => {
    const { videoRtcServiceStart, videoRtcMeetingCreate } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.conference').exists()).toBe(true);
  });

  it('renders local video element', async () => {
    const { videoRtcServiceStart, videoRtcMeetingCreate } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.tile.local').exists()).toBe(true);
  });

  it('renders leave button when in meeting', async () => {
    const { videoRtcServiceStart, videoRtcMeetingCreate } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.btn-danger').text()).toBe('Leave meeting');
  });

  it('leaves meeting on leave button click', async () => {
    const { videoRtcServiceStart, videoRtcMeetingCreate, videoRtcMeetingLeave } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    const meshMock = {
      start: vi.fn(),
      leave: vi.fn()
    };
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => meshMock);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-danger').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(meshMock.leave).toHaveBeenCalled();
  });

  it('emits status on leave', async () => {
    const { videoRtcServiceStart, videoRtcMeetingCreate } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => ({
      start: vi.fn(),
      leave: vi.fn()
    }));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.btn-danger').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[1]).toEqual(['Left meeting']);
  });

  it('loads rooms on mount', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockResolvedValue([]);
    
    mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcMeetingList).toHaveBeenCalled();
  });

  it('starts video RTC service on mount', async () => {
    const { videoRtcServiceStart } = require('$lib/videoRtc');
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    
    mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcServiceStart).toHaveBeenCalled();
  });

  it('falls back to node info if service start fails', async () => {
    const { videoRtcServiceStart, videoRtcNodeInfo } = require('$lib/videoRtc');
    videoRtcServiceStart.mockRejectedValue(new Error('Service error'));
    videoRtcNodeInfo.mockResolvedValue({ nodeId: 'node-1' });
    
    mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(videoRtcNodeInfo).toHaveBeenCalled();
  });

  it('emits status on error', async () => {
    const { videoRtcMeetingList } = require('$lib/videoRtc');
    videoRtcMeetingList.mockRejectedValue(new Error('Failed to load'));
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('cleans up on unmount', async () => {
    const { videoRtcServiceStart, videoRtcMeetingCreate } = require('$lib/videoRtc');
    const { RtcMeetingMesh } = require('$lib/webrtc/rtcMeeting');
    const meshMock = {
      start: vi.fn(),
      leave: vi.fn()
    };
    videoRtcServiceStart.mockResolvedValue({ nodeId: 'node-1' });
    videoRtcMeetingCreate.mockResolvedValue({ meetingId: 'mtg-1', title: 'Test', participants: [], maxParticipants: 6 });
    RtcMeetingMesh.mockImplementation(() => meshMock);
    
    const wrapper = mount(MeetingRoom);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.create input').setValue('Test Meeting');
    await wrapper.findAll('.btn-primary')[0].trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    wrapper.unmount();
    
    expect(meshMock.leave).toHaveBeenCalled();
  });
});
