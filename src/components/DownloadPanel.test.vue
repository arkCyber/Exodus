/**
 * Exodus Browser — DownloadPanel component tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import DownloadPanel from './DownloadPanel.vue';

vi.mock('@/components/BrowserPanel.vue', () => ({
  default: {
    name: 'BrowserPanel',
    template: '<div class="browser-panel"><slot /></div>',
    props: ['open', 'title'],
    emits: ['close']
  }
}));

describe('DownloadPanel', () => {
  const mockDownloads = [
    {
      id: '1',
      filename: 'file1.pdf',
      status: 'completed',
      progress: 100,
      total: 1000,
      path: '/downloads/file1.pdf'
    },
    {
      id: '2',
      filename: 'file2.zip',
      status: 'downloading',
      progress: 45,
      total: 2000,
      path: '/downloads/file2.zip'
    },
    {
      id: '3',
      filename: 'file3.jpg',
      status: 'failed',
      progress: 0,
      total: 500,
      path: null
    },
    {
      id: '4',
      filename: 'file4.txt',
      status: 'pending',
      progress: 0,
      total: 100,
      path: null
    }
  ];

  it('renders BrowserPanel', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: [] }
    });
    
    expect(wrapper.find('.browser-panel').exists()).toBe(true);
  });

  it('passes open prop to BrowserPanel', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: [] }
    });
    
    expect(wrapper.findComponent({ name: 'BrowserPanel' }).props('open')).toBe(true);
  });

  it('passes title to BrowserPanel', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: [] }
    });
    
    expect(wrapper.findComponent({ name: 'BrowserPanel' }).props('title')).toBe('Downloads');
  });

  it('emits close when BrowserPanel emits close', async () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: [] }
    });
    
    await wrapper.findComponent({ name: 'BrowserPanel' }).vm.$emit('close');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('shows empty state when no downloads', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: [] }
    });
    
    expect(wrapper.find('.empty-state').exists()).toBe(true);
    expect(wrapper.find('.empty-state').text()).toBe('No downloads yet. Use menu → Save page as download.');
  });

  it('renders downloads list when downloads exist', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    expect(wrapper.find('.downloads-list').exists()).toBe(true);
    expect(wrapper.find('.empty-state').exists()).toBe(false);
  });

  it('renders all download items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const items = wrapper.findAll('.download-item');
    expect(items.length).toBe(4);
  });

  it('displays download filename', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const firstItem = wrapper.findAll('.download-item')[0];
    expect(firstItem.find('.download-name').text()).toBe('file1.pdf');
  });

  it('displays download status', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const firstItem = wrapper.findAll('.download-item')[0];
    expect(firstItem.find('.download-status').text()).toBe('completed');
  });

  it('displays progress percentage for downloading items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const downloadingItem = wrapper.findAll('.download-item')[1];
    expect(downloadingItem.find('.download-status').text()).toContain('45%');
  });

  it('applies done class to completed status', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const completedItem = wrapper.findAll('.download-item')[0];
    expect(completedItem.find('.download-status').classes()).toContain('done');
  });

  it('applies failed class to failed status', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const failedItem = wrapper.findAll('.download-item')[2];
    expect(failedItem.find('.download-status').classes()).toContain('failed');
  });

  it('shows progress bar for downloading items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const downloadingItem = wrapper.findAll('.download-item')[1];
    expect(downloadingItem.find('.download-progress-track').exists()).toBe(true);
    expect(downloadingItem.find('.download-progress-bar').exists()).toBe(true);
  });

  it('shows progress bar for pending items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const pendingItem = wrapper.findAll('.download-item')[3];
    expect(pendingItem.find('.download-progress-track').exists()).toBe(true);
  });

  it('does not show progress bar for completed items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const completedItem = wrapper.findAll('.download-item')[0];
    expect(completedItem.find('.download-progress-track').exists()).toBe(false);
  });

  it('sets progress bar width correctly', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const downloadingItem = wrapper.findAll('.download-item')[1];
    const progressBar = downloadingItem.find('.download-progress-bar');
    expect(progressBar.attributes('style')).toContain('width: 45%');
  });

  it('shows action buttons for completed downloads with path', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const completedItem = wrapper.findAll('.download-item')[0];
    expect(completedItem.find('.download-item-actions').exists()).toBe(true);
    const actions = completedItem.findAll('.download-action');
    expect(actions.length).toBe(2);
  });

  it('does not show action buttons for downloading items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const downloadingItem = wrapper.findAll('.download-item')[1];
    expect(downloadingItem.find('.download-item-actions').exists()).toBe(false);
  });

  it('does not show action buttons for failed items', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const failedItem = wrapper.findAll('.download-item')[2];
    expect(failedItem.find('.download-item-actions').exists()).toBe(false);
  });

  it('does not show action buttons for completed items without path', () => {
    const downloadsWithoutPath = [
      { id: '1', filename: 'file1.pdf', status: 'completed', progress: 100, total: 1000, path: null }
    ];
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: downloadsWithoutPath }
    });
    
    expect(wrapper.find('.download-item-actions').exists()).toBe(false);
  });

  it('emits open-file when Open button is clicked', async () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const completedItem = wrapper.findAll('.download-item')[0];
    const openButton = completedItem.findAll('.download-action')[0];
    await openButton.trigger('click');
    
    expect(wrapper.emitted('open-file')).toBeTruthy();
    expect(wrapper.emitted('open-file')?.[0]).toEqual(['/downloads/file1.pdf']);
  });

  it('emits reveal-file when Show in folder button is clicked', async () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const completedItem = wrapper.findAll('.download-item')[0];
    const revealButton = completedItem.findAll('.download-action')[1];
    await revealButton.trigger('click');
    
    expect(wrapper.emitted('reveal-file')).toBeTruthy();
    expect(wrapper.emitted('reveal-file')?.[0]).toEqual(['/downloads/file1.pdf']);
  });

  it('applies secondary class to Show in folder button', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const completedItem = wrapper.findAll('.download-item')[0];
    const revealButton = completedItem.findAll('.download-action')[1];
    expect(revealButton.classes()).toContain('secondary');
  });

  it('renders download actions section', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    expect(wrapper.find('.download-actions').exists()).toBe(true);
  });

  it('renders Open folder button', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const buttons = wrapper.findAll('.download-actions .nav-button');
    expect(buttons[0].text()).toBe('Open folder');
  });

  it('emits open-folder when Open folder button is clicked', async () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const buttons = wrapper.findAll('.download-actions .nav-button');
    await buttons[0].trigger('click');
    
    expect(wrapper.emitted('open-folder')).toBeTruthy();
  });

  it('renders Clear list button', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const buttons = wrapper.findAll('.download-actions .nav-button');
    expect(buttons[1].text()).toBe('Clear list');
  });

  it('emits clear when Clear list button is clicked', async () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const buttons = wrapper.findAll('.download-actions .nav-button');
    await buttons[1].trigger('click');
    
    expect(wrapper.emitted('clear')).toBeTruthy();
  });

  it('applies secondary class to action buttons', () => {
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: mockDownloads }
    });
    
    const buttons = wrapper.findAll('.download-actions .nav-button');
    buttons.forEach(button => {
      expect(button.classes()).toContain('secondary');
    });
  });

  it('ensures minimum progress bar width of 2%', () => {
    const downloadsWithZeroProgress = [
      { id: '1', filename: 'file.zip', status: 'downloading', progress: 0, total: 1000, path: null }
    ];
    const wrapper = mount(DownloadPanel, {
      props: { showDownloads: true, downloads: downloadsWithZeroProgress }
    });
    
    const progressBar = wrapper.find('.download-progress-bar');
    expect(progressBar.attributes('style')).toContain('width: 2%');
  });
});
