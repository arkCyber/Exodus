import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('$lib/videoRtc', () => ({
  videoRtcServiceStart: vi.fn(async () => ({ nodeId: 'exodus-test-node-12345678', port: 0 })),
  videoRtcNodeInfo: vi.fn(async () => ({ nodeId: 'exodus-test-node-12345678', port: 0 })),
  videoRtcCallStart: vi.fn(),
  videoRtcCallUpdate: vi.fn(),
}));

vi.mock('$lib/webrtc/rtcCall', () => ({
  RtcOneToOneCall: vi.fn(),
}));

import VideoCall from './VideoCall.vue';

describe('VideoCall', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders call UI after init', async () => {
    const wrapper = mount(VideoCall);
    await flushPromises();
    expect(wrapper.text()).toContain('Call');
    expect(wrapper.text()).toContain('New call');
  });
});
