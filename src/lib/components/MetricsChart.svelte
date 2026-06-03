<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  interface DataPoint {
    timestamp: number;
    value: number;
  }

  interface Props {
    title: string;
    data: DataPoint[];
    color?: string;
    height?: number;
    maxPoints?: number;
    unit?: string;
    showGrid?: boolean;
  }

  let {
    title,
    data = [],
    color = '#3b82f6',
    height = 200,
    maxPoints = 50,
    unit = '',
    showGrid = true
  }: Props = $props();

  let canvas: HTMLCanvasElement | undefined;
  let ctx: CanvasRenderingContext2D | null = null;
  let animationFrame: number | undefined;

  function drawChart() {
    if (!canvas || !ctx) return;

    const width = canvas.width;
    const chartHeight = canvas.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, chartHeight);

    // Limit data points
    const displayData = data.slice(-maxPoints);
    if (displayData.length === 0) return;

    // Calculate min/max for scaling
    const values = displayData.map(d => d.value);
    const minValue = Math.min(...values, 0);
    const maxValue = Math.max(...values, 1);
    const valueRange = maxValue - minValue || 1;

    // Padding
    const padding = { top: 20, right: 40, bottom: 30, left: 50 };
    const chartWidth = width - padding.left - padding.right;
    const innerHeight = chartHeight - padding.top - padding.bottom;

    // Draw grid
    if (showGrid) {
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
      ctx.lineWidth = 1;

      // Horizontal grid lines
      for (let i = 0; i <= 4; i++) {
        const y = padding.top + (innerHeight * i) / 4;
        ctx.beginPath();
        ctx.moveTo(padding.left, y);
        ctx.lineTo(width - padding.right, y);
        ctx.stroke();

        // Y-axis labels
        const value = maxValue - (valueRange * i) / 4;
        ctx.fillStyle = '#999';
        ctx.font = '11px sans-serif';
        ctx.textAlign = 'right';
        ctx.fillText(value.toFixed(1) + unit, padding.left - 5, y + 4);
      }

      // Vertical grid lines
      const timeStep = Math.max(1, Math.floor(displayData.length / 5));
      for (let i = 0; i < displayData.length; i += timeStep) {
        const x = padding.left + (chartWidth * i) / (displayData.length - 1 || 1);
        ctx.beginPath();
        ctx.moveTo(x, padding.top);
        ctx.lineTo(x, chartHeight - padding.bottom);
        ctx.stroke();
      }
    }

    // Draw axes
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.3)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(padding.left, padding.top);
    ctx.lineTo(padding.left, chartHeight - padding.bottom);
    ctx.lineTo(width - padding.right, chartHeight - padding.bottom);
    ctx.stroke();

    // Draw line chart
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.beginPath();

    displayData.forEach((point, index) => {
      const x = padding.left + (chartWidth * index) / (displayData.length - 1 || 1);
      const normalizedValue = (point.value - minValue) / valueRange;
      const y = chartHeight - padding.bottom - (normalizedValue * innerHeight);

      if (index === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });

    ctx.stroke();

    // Draw area fill
    if (displayData.length > 0) {
      const gradient = ctx.createLinearGradient(0, padding.top, 0, chartHeight - padding.bottom);
      gradient.addColorStop(0, color + '40');
      gradient.addColorStop(1, color + '00');

      ctx.fillStyle = gradient;
      ctx.beginPath();

      displayData.forEach((point, index) => {
        const x = padding.left + (chartWidth * index) / (displayData.length - 1 || 1);
        const normalizedValue = (point.value - minValue) / valueRange;
        const y = chartHeight - padding.bottom - (normalizedValue * innerHeight);

        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });

      ctx.lineTo(width - padding.right, chartHeight - padding.bottom);
      ctx.lineTo(padding.left, chartHeight - padding.bottom);
      ctx.closePath();
      ctx.fill();
    }

    // Draw data points
    ctx.fillStyle = color;
    displayData.forEach((point, index) => {
      const x = padding.left + (chartWidth * index) / (displayData.length - 1 || 1);
      const normalizedValue = (point.value - minValue) / valueRange;
      const y = chartHeight - padding.bottom - (normalizedValue * innerHeight);

      ctx.beginPath();
      ctx.arc(x, y, 3, 0, Math.PI * 2);
      ctx.fill();
    });

    // Draw current value
    if (displayData.length > 0) {
      const lastValue = displayData[displayData.length - 1].value;
      ctx.fillStyle = '#fff';
      ctx.font = 'bold 14px sans-serif';
      ctx.textAlign = 'right';
      ctx.fillText(lastValue.toFixed(2) + unit, width - padding.right - 5, padding.top + 15);
    }
  }

  function resizeCanvas() {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * window.devicePixelRatio;
    canvas.height = height * window.devicePixelRatio;
    ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    }
    drawChart();
  }

  onMount(() => {
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    const animate = () => {
      drawChart();
      animationFrame = requestAnimationFrame(animate);
    };
    animate();

    return () => {
      window.removeEventListener('resize', resizeCanvas);
      if (animationFrame) {
        cancelAnimationFrame(animationFrame);
      }
    };
  });

  $effect(() => {
    // Redraw when data changes
    data;
    drawChart();
  });
</script>

<div class="metrics-chart">
  <h4>{title}</h4>
  <canvas bind:this={canvas} style="height: {height}px"></canvas>
</div>

<style>
  .metrics-chart {
    background: var(--bg-secondary, #2a2a2a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
  }

  .metrics-chart h4 {
    margin: 0 0 1rem 0;
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  canvas {
    width: 100%;
    display: block;
    cursor: crosshair;
  }
</style>
