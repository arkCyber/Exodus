import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import DownloadPanel from './DownloadPanel.vue';
import type { DownloadRecord } from '$lib/browserTypes';

describe('DownloadPanel', () => {
  const sample: DownloadRecord[] = [
    {
      id: '1',
      url: 'https://example.com/a.zip',
      filename: 'a.zip',
      status: 'downloading',
      progress: 40,
      received: 40,
      total: 100,
    },
  ];

  it('renders download list when open', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: sample },
      attachTo: document.body,
    });
    expect(wrapper.text()).toContain('a.zip');
    expect(wrapper.text()).toContain('downloading');
    wrapper.unmount();
  });

  it('shows empty state when no downloads', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: [] },
      attachTo: document.body,
    });
    expect(wrapper.text()).toContain('No downloads yet');
    wrapper.unmount();
  });
});
