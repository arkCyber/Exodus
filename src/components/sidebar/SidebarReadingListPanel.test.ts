/**
 * SidebarReadingListPanel tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import SidebarReadingListPanel from './SidebarReadingListPanel.vue';

vi.mock('$lib/localPocket', () => ({
  pocketGetReadingList: vi.fn(async () => [
    {
      id: '1',
      url: 'https://read.example/a',
      title: 'Article A',
      content: '',
      excerpt: '',
      author: null,
      tags: ['reading-list'],
      saved_at: '2026-05-01T12:00:00Z',
      read_at: null,
      is_favorite: false,
      is_archived: false,
      reading_time_minutes: 2,
      word_count: 200,
      reading_priority: 5,
      reading_progress: 0,
      estimated_completion_date: null,
      reading_list_category: 'tech',
    },
    {
      id: '2',
      url: 'https://read.example/b',
      title: 'Article B',
      content: '',
      excerpt: '',
      author: null,
      tags: ['reading-list'],
      saved_at: '2026-05-02T12:00:00Z',
      read_at: null,
      is_favorite: false,
      is_archived: false,
      reading_time_minutes: 3,
      word_count: 300,
      reading_priority: 3,
      reading_progress: 50,
      estimated_completion_date: null,
      reading_list_category: 'news',
    },
  ]),
  pocketGetReadingListStats: vi.fn(async () => ({
    total_articles: 2,
    unread_articles: 1,
    in_progress_articles: 1,
    completed_articles: 0,
    total_reading_time_minutes: 5,
    categories: ['tech', 'news'],
  })),
  pocketGetReadingListCategories: vi.fn(async () => ['tech', 'news']),
}));

import { pocketGetReadingList, pocketGetReadingListStats, pocketGetReadingListCategories } from '$lib/localPocket';

describe('SidebarReadingListPanel', () => {
  beforeEach(() => {
    vi.mocked(pocketGetReadingList).mockClear();
    vi.mocked(pocketGetReadingListStats).mockClear();
    vi.mocked(pocketGetReadingListCategories).mockClear();
  });

  it('lists articles on mount', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(pocketGetReadingList).toHaveBeenCalled();
    expect(wrapper.text()).toContain('Article A');
  });

  it('shows category select', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(wrapper.find('.category-select').exists()).toBe(true);
  });

  it('shows sort select', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(wrapper.find('.sort-select').exists()).toBe(true);
  });

  it('displays reading list stats', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(wrapper.find('.reading-list-stats').exists()).toBe(true);
    expect(wrapper.text()).toContain('2 total');
    expect(wrapper.text()).toContain('1 unread');
    expect(wrapper.text()).toContain('1 in progress');
  });

  it('displays priority badges', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(wrapper.find('.priority-badge').exists()).toBe(true);
    expect(wrapper.text()).toContain('P5');
  });

  it('displays progress bar for in-progress articles', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(wrapper.find('.progress-bar').exists()).toBe(true);
    expect(wrapper.find('.progress-fill').exists()).toBe(true);
  });

  it('displays category tags', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    expect(wrapper.text()).toContain('tech');
    expect(wrapper.text()).toContain('news');
  });

  it('filters by search query', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    const input = wrapper.find('input[type="search"]');
    await input.setValue('Article B');
    expect(wrapper.text()).toContain('Article B');
    expect(wrapper.text()).not.toContain('Article A');
  });

  it('emits navigate when item clicked', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    await wrapper.find('.list-item').trigger('click');
    expect(wrapper.emitted('navigate')?.[0]).toEqual(['https://read.example/a']);
  });

  it('refreshes on category change', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    const select = wrapper.find('.category-select');
    await select.setValue('tech');
    await select.trigger('change');
    expect(pocketGetReadingList).toHaveBeenCalledWith(
      expect.objectContaining({ category: 'tech' })
    );
  });

  it('refreshes on sort change', async () => {
    const wrapper = mount(SidebarReadingListPanel);
    await flushPromises();
    const select = wrapper.find('.sort-select');
    await select.setValue('priority');
    await select.trigger('change');
    expect(pocketGetReadingList).toHaveBeenCalledWith(
      expect.objectContaining({ sort_by: 'priority' })
    );
  });
});
