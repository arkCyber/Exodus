/**
 * Exodus Browser — Picture-in-Picture (画中画) functionality
 * Frontend integration for PiP Tauri commands
 * Supports video element integration similar to Chrome/Firefox
 */

import { invoke } from '@tauri-apps/api/core';

export type PipState = {
  video_url: string;
  is_active: boolean;
  window_width: number;
  window_height: number;
};

/**
 * Enter Picture-in-Picture mode for a video
 */
export async function pipEnter(
  video_url: string,
  width: number = 640,
  height: number = 480
): Promise<void> {
  return invoke('pip_enter', { videoUrl: video_url, width, height });
}

/**
 * Exit Picture-in-Picture mode for a video
 */
export async function pipExit(video_url: string): Promise<void> {
  return invoke('pip_exit', { videoUrl: video_url });
}

/**
 * Resize Picture-in-Picture window
 */
export async function pipResize(
  video_url: string,
  width: number,
  height: number
): Promise<void> {
  return invoke('pip_resize', { videoUrl: video_url, width, height });
}

/**
 * Get Picture-in-Picture state for a specific video
 */
export async function pipGetState(video_url: string): Promise<PipState | null> {
  return invoke('pip_get_state', { videoUrl: video_url });
}

/**
 * Get all active Picture-in-Picture states
 */
export async function pipGetAllActive(): Promise<PipState[]> {
  return invoke('pip_get_all_active');
}

/**
 * Inject Picture-in-Picture API into a webview
 * This adds the HTML5 Picture-in-Picture API to the webview context
 * Security: Validates video URLs and sanitizes inputs before injection
 */
export function injectPictureInPictureAPI(webview: any): void {
  const script = `
    (function() {
      // Security: Check if already injected to prevent duplicate injections
      if (window.__EXODUS_PIP_INJECTED__) {
        console.log('Picture-in-Picture API already injected, skipping');
        return;
      }
      window.__EXODUS_PIP_INJECTED__ = true;
      
      // Security: Validate URL function
      function isValidVideoUrl(url) {
        if (!url || typeof url !== 'string') return false;
        try {
          const parsed = new URL(url);
          return parsed.protocol === 'http:' || 
                 parsed.protocol === 'https:' || 
                 parsed.protocol === 'data:';
        } catch {
          return false;
        }
      }
      
      // Security: Validate dimensions
      function isValidDimensions(width, height) {
        const minSize = 160;
        const maxSize = 4096;
        return width >= minSize && width <= maxSize && 
               height >= minSize && height <= maxSize;
      }
      
      // Override requestPictureInPicture on HTMLVideoElement prototype
      if (HTMLVideoElement.prototype.requestPictureInPicture) {
        const originalRequestPiP = HTMLVideoElement.prototype.requestPictureInPicture;
        
        HTMLVideoElement.prototype.requestPictureInPicture = async function() {
          const video = this;
          const videoUrl = video.src || video.currentSrc;
          const width = video.videoWidth || 640;
          const height = video.videoHeight || 480;
          
          // Security: Validate inputs
          if (!isValidVideoUrl(videoUrl)) {
            console.error('Invalid video URL for Picture-in-Picture:', videoUrl);
            throw new Error('Invalid video URL');
          }
          
          if (!isValidDimensions(width, height)) {
            console.error('Invalid dimensions for Picture-in-Picture:', width, height);
            throw new Error('Invalid dimensions');
          }
          
          // Call Tauri backend
          if (window.__TAURI__) {
            try {
              await window.__TAURI__.invoke('pip_enter', {
                videoUrl: videoUrl,
                width: width,
                height: height
              });
              
              // Return a mock PiP window object
              return {
                onresize: null,
                onenter: null,
                onleave: null,
                width: width,
                height: height
              };
            } catch (error) {
              console.error('Picture-in-Picture failed:', error);
              throw error;
            }
          }
          
          // Fallback to native API if available
          return originalRequestPiP.call(video);
        };
      }
      
      // Add exitPictureInPicture if not present
      if (!HTMLVideoElement.prototype.exitPictureInPicture) {
        HTMLVideoElement.prototype.exitPictureInPicture = async function() {
          const video = this;
          const videoUrl = video.src || video.currentSrc;
          
          // Security: Validate URL
          if (!isValidVideoUrl(videoUrl)) {
            console.error('Invalid video URL for Picture-in-Picture exit:', videoUrl);
            throw new Error('Invalid video URL');
          }
          
          if (window.__TAURI__) {
            try {
              await window.__TAURI__.invoke('pip_exit', {
                videoUrl: videoUrl
              });
              return;
            } catch (error) {
              console.error('Exit Picture-in-Picture failed:', error);
              throw error;
            }
          }
        };
      }
      
      // Listen for Tauri events to update video state
      if (window.__TAURI__) {
        window.__TAURI__.event.listen('exodus-pip-entered', (event) => {
          console.log('Picture-in-Picture entered:', event.payload);
        });
        
        window.__TAURI__.event.listen('exodus-pip-exited', (event) => {
          console.log('Picture-in-Picture exited:', event.payload);
        });
        
        window.__TAURI__.event.listen('exodus-pip-resized', (event) => {
          console.log('Picture-in-Picture resized:', event.payload);
        });
      }
      
      console.log('Picture-in-Picture API injected successfully');
    })();
  `;
  
  webview.executeJavaScript(script);
}

