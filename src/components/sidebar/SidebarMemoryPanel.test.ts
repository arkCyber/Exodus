/**
 * Exodus Browser — SidebarMemoryPanel component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import SidebarMemoryPanel from './SidebarMemoryPanel.vue';

describe('SidebarMemoryPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders list panel', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-panel').exists()).toBe(true);
  });

  it('renders history search input', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0,
      },
    });
    expect(wrapper.find('input.search-input').attributes('placeholder')).toContain('Search');
  });

  it('emits memory-search on input', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0,
      },
    });
    await wrapper.find('input.search-input').setValue('docs');
    expect(wrapper.emitted('memory-search')?.slice(-1)[0]).toEqual(['docs']);
  });

  it('renders history panel actions', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.history-panel-actions').exists()).toBe(true);
  });

  it('renders refresh button', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.history-panel-actions .nav-button').text()).toBe('Refresh');
  });

  it('emits load-memory on refresh button click', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    await wrapper.find('.history-panel-actions .nav-button').trigger('click');
    
    expect(wrapper.emitted('load-memory')).toBeTruthy();
  });

  it('renders indexed memory section title', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 5,
        historyCount: 0
      }
    });
    
    expect(wrapper.findAll('.memory-section-title')[0].text()).toBe('Indexed memory (5)');
  });

  it('renders indexed memory hint', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.memory-section-hint').text()).toContain('Pages saved for /ask search');
  });

  it('shows empty state when no indexed pages', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    const mutedElements = wrapper.findAll('.muted');
    expect(mutedElements[1].text()).toBe('No indexed pages yet.');
  });

  it('renders indexed memory groups', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.history-group-label').text()).toBe('Today');
  });

  it('renders indexed page items', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-item.row').exists()).toBe(true);
  });

  it('displays page title', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-title').text()).toBe('Example');
  });

  it('displays URL when title is missing', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: '', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-title').text()).toBe('https://example.com');
  });

  it('displays page URL', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-sub').text()).toBe('https://example.com');
  });

  it('renders remove button on indexed page', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.tab-close').exists()).toBe(true);
  });

  it('has correct aria-label on remove button', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.tab-close').attributes('aria-label')).toBe('Remove from indexed memory');
  });

  it('emits navigate on page click', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    await wrapper.find('.list-grow').trigger('click');
    
    expect(wrapper.emitted('navigate')).toBeTruthy();
    expect(wrapper.emitted('navigate')?.[0]).toEqual(['https://example.com']);
  });

  it('emits remove-indexed on remove button click', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    await wrapper.find('.tab-close').trigger('click');
    
    expect(wrapper.emitted('remove-indexed')).toBeTruthy();
    expect(wrapper.emitted('remove-indexed')?.[0]).toEqual(['1']);
  });

  it('renders clear indexed button', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    const buttons = wrapper.findAll('.nav-button.danger');
    expect(buttons[0].text()).toBe('Clear indexed memory');
  });

  it('emits clear-indexed on clear button click', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    const buttons = wrapper.findAll('.nav-button.danger');
    await buttons[0].trigger('click');
    
    expect(wrapper.emitted('clear-indexed')).toBeTruthy();
  });

  it('renders browsing history section title', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 10
      }
    });
    
    const titles = wrapper.findAll('.memory-section-title');
    expect(titles[1].text()).toBe('Browsing history (10)');
  });

  it('shows empty state when no history', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [],
        indexedCount: 0,
        historyCount: 0
      }
    });
    
    const mutedElements = wrapper.findAll('.muted');
    expect(mutedElements[mutedElements.length - 1].text()).toBe('No visits recorded yet.');
  });

  it('renders history groups', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const labels = wrapper.findAll('.history-group-label');
    expect(labels[labels.length - 1].text()).toBe('Today');
  });

  it('renders history page items', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const items = wrapper.findAll('.list-item');
    expect(items[items.length - 1].exists()).toBe(true);
  });

  it('displays visit count when > 1', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 5, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const subs = wrapper.findAll('.list-sub');
    expect(subs[subs.length - 1].text()).toContain('5 visits');
  });

  it('does not display visit count when 1', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const subs = wrapper.findAll('.list-sub');
    expect(subs[subs.length - 1].text()).not.toContain('visits');
  });

  it('emits navigate on history page click', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const items = wrapper.findAll('.list-item');
    await items[items.length - 1].trigger('click');
    
    expect(wrapper.emitted('navigate')).toBeTruthy();
    expect(wrapper.emitted('navigate')?.[0]).toEqual(['https://example.com']);
  });

  it('renders clear history button', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const buttons = wrapper.findAll('.nav-button.danger');
    expect(buttons[buttons.length - 1].text()).toBe('Clear browsing history');
  });

  it('emits clear-history on clear button click', async () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const buttons = wrapper.findAll('.nav-button.danger');
    await buttons[buttons.length - 1].trigger('click');
    
    expect(wrapper.emitted('clear-history')).toBeTruthy();
  });

  it('has role link on indexed page', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-grow').attributes('role')).toBe('link');
  });

  it('has tabindex 0 on indexed page', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        historyGroups: [],
        indexedCount: 1,
        historyCount: 0
      }
    });
    
    expect(wrapper.find('.list-grow').attributes('tabindex')).toBe('0');
  });

  it('has role link on history page', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const items = wrapper.findAll('.list-item');
    expect(items[items.length - 1].attributes('role')).toBe('link');
  });

  it('has tabindex 0 on history page', () => {
    const wrapper = mount(SidebarMemoryPanel, {
      props: {
        indexedMemoryGroups: [],
        historyGroups: [
          {
            label: 'Today',
            pages: [
              { id: '1', url: 'https://example.com', title: 'Example', visit_count: 1, timestamp: String(Date.now()) }
            ]
          }
        ],
        indexedCount: 0,
        historyCount: 1
      }
    });
    
    const items = wrapper.findAll('.list-item');
    expect(items[items.length - 1].attributes('tabindex')).toBe('0');
  });
});
