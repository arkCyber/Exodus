<script lang="ts">
  /**
   * Exodus Browser — AI Video Analysis UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface AnalysisResult {
    id: string;
    video_url: string;
    status: 'pending' | 'analyzing' | 'completed' | 'error';
    progress: number;
    objects_detected: string[];
    summary: string;
    timestamp: number;
  }

  let analyses: AnalysisResult[] = [];
  let showAnalyzeDialog = false;
  let selectedAnalysis: AnalysisResult | null = null;

  // Analyze form
  let videoUrl = '';
  let analysisType: 'objects' | 'summary' | 'both' = 'both';

  async function loadAnalyses() {
    // In a real implementation, this would load from the backend
    // For now, we'll use a placeholder
    analyses = [];
  }

  async function startAnalysis() {
    if (!videoUrl) return;

    try {
      const analysis: AnalysisResult = {
        id: crypto.randomUUID(),
        video_url: videoUrl,
        status: 'analyzing',
        progress: 0,
        objects_detected: [],
        summary: '',
        timestamp: Date.now() / 1000,
      };

      // In a real implementation, this would start analysis via the backend
      console.log('Starting analysis:', analysis);
      
      analyses.push(analysis);
      showAnalyzeDialog = false;
      videoUrl = '';

      // Simulate analysis progress
      simulateAnalysis(analysis);
    } catch (error) {
      console.error('Failed to start analysis:', error);
    }
  }

  function simulateAnalysis(analysis: AnalysisResult) {
    let progress = 0;
    const interval = setInterval(() => {
      progress += 10;
      analysis.progress = progress;

      if (progress >= 100) {
        clearInterval(interval);
        analysis.status = 'completed';
        analysis.objects_detected = ['person', 'car', 'building', 'tree'];
        analysis.summary = 'Video shows a person walking near a building with cars and trees in the background. The scene appears to be in an urban setting.';
      }
    }, 500);
  }

  async function deleteAnalysis(id: string) {
    if (!confirm('Are you sure you want to delete this analysis?')) return;

    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting analysis:', id);
      
      analyses = analyses.filter((a) => a.id !== id);
      
      if (selectedAnalysis && selectedAnalysis.id === id) {
        selectedAnalysis = null;
      }
    } catch (error) {
      console.error('Failed to delete analysis:', error);
    }
  }

  function selectAnalysis(analysis: AnalysisResult) {
    selectedAnalysis = analysis;
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'completed':
        return '#059669';
      case 'analyzing':
        return '#2563eb';
      case 'error':
        return '#dc2626';
      default:
        return '#d97706';
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  // Load analyses on mount
  loadAnalyses();
</script>

<div class="ai-video-analysis">
  <div class="header">
    <h2>AI Video Analysis</h2>
    <div class="actions">
      <button class="btn btn-primary" on:click={() => (showAnalyzeDialog = true)}>
        Analyze Video
      </button>
    </div>
  </div>

  <div class="analysis-container">
    <div class="analysis-list">
      <div class="list-header">
        <h3>Analyses</h3>
      </div>
      {#if analyses.length === 0}
        <div class="empty-state">
          <p>No analyses yet</p>
        </div>
      {:else}
        {#each analyses as analysis (analysis.id)}
          <div
            class="analysis-item {selectedAnalysis?.id === analysis.id ? 'active' : ''}"
            on:click={() => selectAnalysis(analysis)}
          >
            <div class="analysis-info">
              <div class="url">{analysis.video_url}</div>
              <div class="status" style="color: {getStatusColor(analysis.status)}">
                {analysis.status}
              </div>
              {#if analysis.status === 'analyzing'}
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {analysis.progress}%"></div>
                </div>
              {/if}
              <div class="timestamp">{formatDate(analysis.timestamp)}</div>
            </div>
            <button
              class="btn-icon delete"
              title="Delete"
              on:click|stopPropagation={() => deleteAnalysis(analysis.id)}
            >
              🗑️
            </button>
          </div>
        {/each}
      {/if}
    </div>

    <div class="analysis-details">
      {#if selectedAnalysis}
        <div class="details-header">
          <h3>Analysis Details</h3>
          <button class="btn-icon" on:click={() => (selectedAnalysis = null)}>✕</button>
        </div>
        <div class="details-content">
          <div class="detail-section">
            <h4>Video URL</h4>
            <div class="url">{selectedAnalysis.video_url}</div>
          </div>
          <div class="detail-section">
            <h4>Status</h4>
            <div class="status" style="color: {getStatusColor(selectedAnalysis.status)}">
              {selectedAnalysis.status}
            </div>
            {#if selectedAnalysis.status === 'analyzing'}
              <div class="progress-bar">
                <div class="progress-fill" style="width: {selectedAnalysis.progress}%"></div>
              </div>
              <div class="progress-text">{selectedAnalysis.progress}%</div>
            {/if}
          </div>
          {#if selectedAnalysis.status === 'completed'}
            <div class="detail-section">
              <h4>Objects Detected</h4>
              <div class="objects-list">
                {#each selectedAnalysis.objects_detected as object}
                  <span class="object-tag">{object}</span>
                {/each}
              </div>
            </div>
            <div class="detail-section">
              <h4>Summary</h4>
              <div class="summary">{selectedAnalysis.summary}</div>
            </div>
          {/if}
          <div class="detail-section">
            <h4>Timestamp</h4>
            <div class="timestamp">{formatDate(selectedAnalysis.timestamp)}</div>
          </div>
        </div>
      {:else}
        <div class="no-selection">
          <div class="no-selection-icon">🎬</div>
          <p>Select an analysis to view details</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Analyze Dialog -->
  {#if showAnalyzeDialog}
    <div class="dialog-overlay" on:click={() => (showAnalyzeDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Analyze Video</h3>
        <form on:submit|preventDefault={startAnalysis}>
          <div class="form-group">
            <label>Video URL</label>
            <input type="url" bind:value={videoUrl} placeholder="https://..." required />
          </div>
          <div class="form-group">
            <label>Analysis Type</label>
            <select bind:value={analysisType}>
              <option value="objects">Object Detection</option>
              <option value="summary">Video Summary</option>
              <option value="both">Both</option>
            </select>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showAnalyzeDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Analyze</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .ai-video-analysis {
    padding: 20px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
  }

  .analysis-container {
    flex: 1;
    display: flex;
    gap: 20px;
    min-height: 0;
  }

  .analysis-list {
    width: 350px;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .list-header {
    padding: 15px;
    border-bottom: 1px solid #444;
  }

  .list-header h3 {
    margin: 0;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #888;
  }

  .analysis-item {
    padding: 15px;
    background: #444;
    border-radius: 6px;
    margin-bottom: 10px;
    cursor: pointer;
    position: relative;
    transition: background 0.2s;
  }

  .analysis-item:hover {
    background: #555;
  }

  .analysis-item.active {
    background: #6366f1;
    border-color: #4f46e5;
  }

  .analysis-info {
    margin-bottom: 5px;
  }

  .url {
    color: #eee;
    font-size: 14px;
    margin-bottom: 5px;
    word-break: break-all;
  }

  .status {
    font-size: 12px;
    font-weight: bold;
    text-transform: capitalize;
    margin-bottom: 5px;
  }

  .progress-bar {
    height: 4px;
    background: #555;
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 5px;
  }

  .progress-fill {
    height: 100%;
    background: #2563eb;
    transition: width 0.3s ease;
  }

  .timestamp {
    font-size: 12px;
    color: #888;
  }

  .analysis-item .btn-icon {
    position: absolute;
    top: 10px;
    right: 10px;
    background: transparent;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 4px;
  }

  .analysis-item .btn-icon:hover {
    color: #dc2626;
  }

  .analysis-details {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    border-bottom: 1px solid #444;
  }

  .details-header h3 {
    margin: 0;
  }

  .details-content {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
  }

  .detail-section {
    margin-bottom: 25px;
  }

  .detail-section h4 {
    margin: 0 0 10px 0;
    color: #aaa;
  }

  .detail-section .url {
    color: #eee;
    font-family: monospace;
    word-break: break-all;
  }

  .objects-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .object-tag {
    padding: 6px 12px;
    background: #444;
    border-radius: 16px;
    font-size: 14px;
    color: #eee;
  }

  .summary {
    color: #eee;
    line-height: 1.6;
  }

  .progress-text {
    font-size: 12px;
    color: #aaa;
    margin-top: 5px;
  }

  .no-selection {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    color: #888;
  }

  .no-selection-icon {
    font-size: 64px;
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #333;
    border: 1px solid #555;
    border-radius: 8px;
    padding: 20px;
    min-width: 400px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    color: #aaa;
  }

  .form-group input[type='url'],
  .form-group select {
    width: 100%;
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    border: none;
  }

  .btn-primary {
    background: #6366f1;
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }

  .btn-secondary {
    background: #444;
    color: #eee;
  }

  .btn-secondary:hover {
    background: #555;
  }

  .btn-icon {
    background: transparent;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 8px;
    font-size: 18px;
  }

  .btn-icon:hover {
    color: #eee;
  }
</style>
