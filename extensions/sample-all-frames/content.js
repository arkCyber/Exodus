/**
 * Exodus sample — all_frames content script (document_start).
 * Marks top window vs iframe via data-exodus-all-frames on <html>.
 */
(function () {
  const frameRole = window === window.top ? 'top' : 'iframe';
  if (document.documentElement) {
    document.documentElement.dataset.exodusAllFrames = frameRole;
  }
  console.debug('[Exodus sample-all-frames]', frameRole, location.href);
})();
