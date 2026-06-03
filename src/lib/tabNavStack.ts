/**
 * Exodus Browser — per-tab navigation stack for iframe dev mode.
 * Native tabs use Rust TabNavTracker via browser_get_nav_state.
 */

export type TabNavTrack = {
  stack: string[];
  index: number;
};

/** Record a navigation or history move for a tab id. */
export function recordTabNavigation(
  tracks: Map<string, TabNavTrack>,
  tabId: string,
  url: string,
): TabNavTrack {
  if (!url || url.startsWith('data:')) {
    return tracks.get(tabId) ?? { stack: [], index: 0 };
  }

  let track = tracks.get(tabId);
  if (!track) {
    track = { stack: [url], index: 0 };
    tracks.set(tabId, track);
    return track;
  }

  const current = track.stack[track.index];
  if (current === url) {
    return track;
  }
  if (track.index > 0 && track.stack[track.index - 1] === url) {
    track.index -= 1;
    return track;
  }
  if (track.index + 1 < track.stack.length && track.stack[track.index + 1] === url) {
    track.index += 1;
    return track;
  }

  track.stack = track.stack.slice(0, track.index + 1);
  if (track.stack[track.stack.length - 1] !== url) {
    track.stack.push(url);
  }
  track.index = track.stack.length - 1;
  tracks.set(tabId, track);
  return track;
}

/** Whether back/forward are available for a tab stack. */
export function navFlagsFromTrack(track: TabNavTrack | undefined): {
  canGoBack: boolean;
  canGoForward: boolean;
} {
  if (!track || track.stack.length === 0) {
    return { canGoBack: false, canGoForward: false };
  }
  return {
    canGoBack: track.index > 0,
    canGoForward: track.index + 1 < track.stack.length,
  };
}
