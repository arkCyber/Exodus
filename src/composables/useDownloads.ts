import { ref } from 'vue';

interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  totalBytes: number;
  receivedBytes: number;
  state: 'pending' | 'downloading' | 'completed' | 'cancelled' | 'error';
  startTime: number;
  endTime?: number;
  savePath?: string;
  error?: string;
}

const downloads = ref<DownloadItem[]>([]);

export function useDownloads() {
  function startDownload(url: string, filename: string, totalBytes: number): DownloadItem {
    const download: DownloadItem = {
      id: `download-${Date.now()}`,
      url,
      filename,
      totalBytes,
      receivedBytes: 0,
      state: 'pending',
      startTime: Date.now(),
    };
    
    downloads.value.push(download);
    
    try {
      localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
    } catch (e) {
      console.error('Failed to save downloads:', e);
    }
    
    return download;
  }
  
  function updateDownloadProgress(id: string, receivedBytes: number, state?: DownloadItem['state']) {
    const download = downloads.value.find(d => d.id === id);
    if (download) {
      download.receivedBytes = receivedBytes;
      if (state) {
        download.state = state;
      }
      
      try {
        localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
      } catch (e) {
        console.error('Failed to save downloads:', e);
      }
    }
  }
  
  function completeDownload(id: string, savePath: string) {
    const download = downloads.value.find(d => d.id === id);
    if (download) {
      download.state = 'completed';
      download.endTime = Date.now();
      download.savePath = savePath;
      download.receivedBytes = download.totalBytes;
      
      try {
        localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
      } catch (e) {
        console.error('Failed to save downloads:', e);
      }
    }
  }
  
  function cancelDownload(id: string) {
    const download = downloads.value.find(d => d.id === id);
    if (download) {
      download.state = 'cancelled';
      download.endTime = Date.now();
      
      try {
        localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
      } catch (e) {
        console.error('Failed to save downloads:', e);
      }
    }
  }
  
  function failDownload(id: string, error: string) {
    const download = downloads.value.find(d => d.id === id);
    if (download) {
      download.state = 'error';
      download.endTime = Date.now();
      download.error = error;
      
      try {
        localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
      } catch (e) {
        console.error('Failed to save downloads:', e);
      }
    }
  }
  
  function removeDownload(id: string) {
    const index = downloads.value.findIndex(d => d.id === id);
    if (index > -1) {
      downloads.value.splice(index, 1);
      
      try {
        localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
      } catch (e) {
        console.error('Failed to save downloads:', e);
      }
    }
  }
  
  function clearCompletedDownloads() {
    downloads.value = downloads.value.filter(d => d.state !== 'completed');
    
    try {
      localStorage.setItem('browser-downloads', JSON.stringify(downloads.value));
    } catch (e) {
      console.error('Failed to save downloads:', e);
    }
  }
  
  function clearAllDownloads() {
    downloads.value = [];
    
    try {
      localStorage.removeItem('browser-downloads');
    } catch (e) {
      console.error('Failed to clear downloads:', e);
    }
  }
  
  function getDownloadProgress(id: string): number {
    const download = downloads.value.find(d => d.id === id);
    if (!download || download.totalBytes === 0) return 0;
    return (download.receivedBytes / download.totalBytes) * 100;
  }
  
  function getDownloadSpeed(id: string): number {
    const download = downloads.value.find(d => d.id === id);
    if (!download || download.state !== 'downloading') return 0;
    
    const elapsed = (Date.now() - download.startTime) / 1000; // seconds
    if (elapsed === 0) return 0;
    
    return download.receivedBytes / elapsed; // bytes per second
  }
  
  function getActiveDownloads(): DownloadItem[] {
    return downloads.value.filter(d => 
      d.state === 'pending' || d.state === 'downloading'
    );
  }
  
  function getCompletedDownloads(): DownloadItem[] {
    return downloads.value.filter(d => d.state === 'completed');
  }
  
  function loadDownloads() {
    try {
      const saved = localStorage.getItem('browser-downloads');
      if (saved) {
        downloads.value = JSON.parse(saved);
      }
    } catch (e) {
      console.error('Failed to load downloads:', e);
    }
  }
  
  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }
  
  function formatSpeed(bytesPerSecond: number): string {
    return `${formatBytes(bytesPerSecond)}/s`;
  }
  
  function formatTimeRemaining(download: DownloadItem): string {
    if (download.state !== 'downloading' || download.totalBytes === 0) {
      return '';
    }
    
    const remainingBytes = download.totalBytes - download.receivedBytes;
    const speed = getDownloadSpeed(download.id);
    
    if (speed === 0) return '';
    
    const remainingSeconds = remainingBytes / speed;
    
    if (remainingSeconds < 60) {
      return `${Math.round(remainingSeconds)}s`;
    } else if (remainingSeconds < 3600) {
      return `${Math.round(remainingSeconds / 60)}m`;
    } else {
      return `${Math.round(remainingSeconds / 3600)}h`;
    }
  }
  
  return {
    downloads,
    startDownload,
    updateDownloadProgress,
    completeDownload,
    cancelDownload,
    failDownload,
    removeDownload,
    clearCompletedDownloads,
    clearAllDownloads,
    getDownloadProgress,
    getDownloadSpeed,
    getActiveDownloads,
    getCompletedDownloads,
    loadDownloads,
    formatBytes,
    formatSpeed,
    formatTimeRemaining,
  };
}
