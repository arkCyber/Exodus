/**
 * Exodus Browser — PocketPanel component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import PocketPanel from './PocketPanel.vue';

vi.mock('$lib/localPocket', () => ({
  pocketListArticles: vi.fn(),
  pocketSearchArticles: vi.fn()
}));

describe('PocketPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders pocket panel', () => {
    const wrapper = mount(PocketPanel);
    
    expect(wrapper.find('.pocket-panel').exists()).toBe(true);
  });

  it('renders toolbar', () => {
    const wrapper = mount(PocketPanel);
    
    expect(wrapper.find('.pocket-toolbar').exists()).toBe(true);
  });

  it('renders search input', () => {
    const wrapper = mount(PocketPanel);
    
    expect(wrapper.find('.search-input').exists()).toBe(true);
    expect(wrapper.find('.search-input').attributes('placeholder')).toBe('Search saved articles…');
  });

  it('renders refresh button', () => {
    const wrapper = mount(PocketPanel);
    
    expect(wrapper.find('.nav-button').exists()).toBe(true);
    expect(wrapper.find('.nav-button').text()).toBe('Refresh');
  });

  it('shows loading state initially', () => {
    const wrapper = mount(PocketPanel);
    
    expect(wrapper.find('.muted').text()).toBe('Loading…');
  });

  it('renders article list after loading', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-list').exists()).toBe(true);
  });

  it('renders article items', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const articles = wrapper.findAll('.article-item');
    expect(articles.length).toBe(1);
  });

  it('displays article title', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-title').text()).toBe('Test Article');
  });

  it('displays URL when title is missing', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: '', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-title').text()).toBe('https://example.com');
  });

  it('displays formatted date', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test', url: 'https://example.com', saved_at: '2024-01-01T00:00:00Z' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-meta').exists()).toBe(true);
  });

  it('returns original date string on format error', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test', url: 'https://example.com', saved_at: 'invalid-date' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-meta').text()).toBe('invalid-date');
  });

  it('shows no articles message when empty', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.muted').some(m => m.text() === 'No saved articles')).toBe(true);
  });

  it('selects article on click', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.article-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-detail').exists()).toBe(true);
  });

  it('renders article detail when selected', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.article-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-detail h4').text()).toBe('Test Article');
  });

  it('renders article URL as link', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.article-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    const link = wrapper.find('.article-detail a');
    expect(link.attributes('href')).toBe('https://example.com');
    expect(link.attributes('target')).toBe('_blank');
    expect(link.attributes('rel')).toBe('noopener');
  });

  it('renders close button in detail', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.article-item').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-detail .nav-button').text()).toBe('Close');
  });

  it('closes detail on close button click', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([
      { id: '1', title: 'Test Article', url: 'https://example.com', saved_at: '2024-01-01' }
    ]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    await wrapper.find('.article-item').trigger('click');
    await wrapper.vm.$nextTick();
    await wrapper.find('.article-detail .nav-button').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.article-detail').exists()).toBe(false);
  });

  it('calls refresh on mount', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([]);
    
    mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(pocketListArticles).toHaveBeenCalled();
  });

  it('calls refresh when refresh button is clicked', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    pocketListArticles.mockClear();
    
    await wrapper.find('.nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(pocketListArticles).toHaveBeenCalled();
  });

  it('uses search when query is provided', async () => {
    const { pocketListArticles, pocketSearchArticles } = require('$lib/localPocket');
    pocketListArticles.mockResolvedValue([]);
    pocketSearchArticles.mockResolvedValue([]);
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    
    await wrapper.find('.search-input').setValue('test query');
    await wrapper.find('.nav-button').trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    
    expect(pocketSearchArticles).toHaveBeenCalledWith({ query: 'test query', limit: 50, offset: null });
  });

  it('emits status on error', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockRejectedValue(new Error('Failed to load'));
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
    expect(wrapper.emitted('status')?.[0]).toEqual(['Error: Failed to load']);
  });

  it('clears articles on error', async () => {
    const { pocketListArticles } = require('$lib/localPocket');
    pocketListArticles.mockRejectedValue(new Error('Failed to load'));
    
    const wrapper = mount(PocketPanel);
    
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.article-item').length).toBe(0);
  });
});
