import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useDownloads } from './useDownloads';

describe('useDownloads', () => {
  beforeEach(() => {
    localStorage.clear();
    useDownloads().clearAllDownloads();
  });

  it('starts download', () => {
    const { startDownload, downloads } = useDownloads();
    
    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    
    expect(downloads.value.length).toBe(1);
    expect(download.url).toBe('https://example.com/file.zip');
    expect(download.filename).toBe('file.zip');
    expect(download.totalBytes).toBe(1024);
    expect(download.state).toBe('pending');
  });

  it('updates download progress', () => {
    const { startDownload, updateDownloadProgress, getDownloadProgress } = useDownloads();
    
    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    updateDownloadProgress(download.id, 512, 'downloading');
    
    expect(getDownloadProgress(download.id)).toBe(50);
  });

  it('completes download', () => {
    const { startDownload, completeDownload, downloads } = useDownloads();
    
    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    completeDownload(download.id, '/path/to/file.zip');
    const row = downloads.value.find((d) => d.id === download.id);

    expect(row?.state).toBe('completed');
    expect(row?.savePath).toBe('/path/to/file.zip');
  });

  it('cancels download', () => {
    const { startDownload, cancelDownload, downloads } = useDownloads();
    
    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    cancelDownload(download.id);
    
    const row = downloads.value.find((d) => d.id === download.id);
    expect(row?.state).toBe('cancelled');
  });

  it('fails download', () => {
    const { startDownload, failDownload, downloads } = useDownloads();
    
    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    failDownload(download.id, 'Network error');
    
    const row = downloads.value.find((d) => d.id === download.id);
    expect(row?.state).toBe('error');
    expect(row?.error).toBe('Network error');
  });

  it('removes download', () => {
    const { startDownload, removeDownload, downloads } = useDownloads();
    
    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    removeDownload(download.id);
    
    expect(downloads.value.length).toBe(0);
  });

  it('clears completed downloads', () => {
    const { startDownload, completeDownload, clearCompletedDownloads, downloads } = useDownloads();
    
    const download1 = startDownload('https://example.com/file1.zip', 'file1.zip', 1024);
    const download2 = startDownload('https://example.com/file2.zip', 'file2.zip', 1024);
    
    completeDownload(download1.id, '/path/to/file1.zip');
    clearCompletedDownloads();
    
    expect(downloads.value.length).toBe(1);
    expect(downloads.value[0].id).toBe(download2.id);
  });

  it('clears all downloads', () => {
    const { startDownload, clearAllDownloads, downloads } = useDownloads();
    
    startDownload('https://example.com/file.zip', 'file.zip', 1024);
    clearAllDownloads();
    
    expect(downloads.value.length).toBe(0);
  });

  it('calculates download speed', async () => {
    vi.useFakeTimers();
    const { startDownload, updateDownloadProgress, getDownloadSpeed } = useDownloads();

    const download = startDownload('https://example.com/file.zip', 'file.zip', 1024);
    updateDownloadProgress(download.id, 512, 'downloading');
    vi.advanceTimersByTime(1000);

    const speed = getDownloadSpeed(download.id);
    expect(speed).toBeGreaterThan(0);
    vi.useRealTimers();
  });

  it('gets active downloads', () => {
    const { startDownload, completeDownload, getActiveDownloads } = useDownloads();
    
    const download1 = startDownload('https://example.com/file1.zip', 'file1.zip', 1024);
    const download2 = startDownload('https://example.com/file2.zip', 'file2.zip', 1024);
    
    completeDownload(download1.id, '/path/to/file1.zip');
    
    const active = getActiveDownloads();
    expect(active.length).toBe(1);
    expect(active[0].id).toBe(download2.id);
  });

  it('formats bytes correctly', () => {
    const { formatBytes } = useDownloads();
    
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1024)).toBe('1.00 KB');
    expect(formatBytes(1024 * 1024)).toBe('1.00 MB');
    expect(formatBytes(1024 * 1024 * 1024)).toBe('1.00 GB');
  });

  it('formats speed correctly', () => {
    const { formatSpeed } = useDownloads();
    
    expect(formatSpeed(1024)).toBe('1.00 KB/s');
    expect(formatSpeed(1024 * 1024)).toBe('1.00 MB/s');
  });
});
