/**
 * Exodus Browser — group chat @mentions (resolve contacts → node ids, quick call).
 */
import type { Contact } from '$lib/contactDirectory';
import { isLikelyPeerNodeId } from '$lib/imSession';

export type MentionTarget = {
  nodeId: string;
  displayName: string;
  contactId?: string;
};

/** Token inserted in draft, e.g. @[Alice](node:abc…). */
export function formatMentionToken(target: MentionTarget): string {
  const safeName = target.displayName.replace(/[\[\]]/g, '');
  return `@[${safeName}](node:${target.nodeId})`;
}

/**
 * Parse explicit mention tokens and bare @node: / @digit: / @name from text.
 */
export function extractMentionNodeIds(text: string, contacts: Contact[]): string[] {
  const found = new Set<string>();

  const tokenRe = /@\[([^\]]+)\]\(node:([^)]+)\)/g;
  let m: RegExpExecArray | null;
  while ((m = tokenRe.exec(text)) !== null) {
    if (isLikelyPeerNodeId(m[2])) found.add(m[2]);
  }

  const nodeRe = /@node:([A-Za-z0-9_-]+)/g;
  while ((m = nodeRe.exec(text)) !== null) {
    if (isLikelyPeerNodeId(m[1])) found.add(m[1]);
  }

  const digitRe = /@(\d{12})\b/g;
  while ((m = digitRe.exec(text)) !== null) {
    const c = contacts.find((x) => x.notes.includes(m[1]));
    if (c?.node_id) found.add(c.node_id);
  }

  const bareRe = /@([^\s@]+)/g;
  while ((m = bareRe.exec(text)) !== null) {
    const key = m[1].toLowerCase();
    if (key.startsWith('node:') || key.length === 12) continue;
    const hit = contacts.find((c) => c.name.toLowerCase() === key);
    if (hit?.node_id && isLikelyPeerNodeId(hit.node_id)) found.add(hit.node_id);
  }

  return [...found];
}

/** Map address-book rows to mention targets. */
export function contactsToMentionTargets(contacts: Contact[]): MentionTarget[] {
  return contacts
    .filter((c) => isLikelyPeerNodeId(c.node_id) && !c.is_blocked)
    .map((c) => ({
      nodeId: c.node_id,
      displayName: c.name,
      contactId: c.contact_id,
    }));
}

/**
 * Merge contacts, group member ids, and recent sender names for @ autocomplete.
 */
export function mergeMentionCandidates(
  contacts: Contact[],
  extras: Array<{ nodeId: string; displayName: string }>
): MentionTarget[] {
  const map = new Map<string, MentionTarget>();
  for (const t of contactsToMentionTargets(contacts)) {
    map.set(t.nodeId, t);
  }
  for (const e of extras) {
    if (!isLikelyPeerNodeId(e.nodeId) || map.has(e.nodeId)) continue;
    map.set(e.nodeId, { nodeId: e.nodeId, displayName: e.displayName });
  }
  return [...map.values()];
}

/** Contacts matching an active @ query (after the last @ in draft). */
export function mentionCandidatesForQuery(
  draft: string,
  contacts: Contact[]
): { query: string; candidates: MentionTarget[] } | null {
  return mentionCandidatesForQueryFromTargets(draft, contactsToMentionTargets(contacts));
}

/** @ autocomplete against a pre-built target list (group members + contacts). */
export function mentionCandidatesForQueryFromTargets(
  draft: string,
  targets: MentionTarget[]
): { query: string; candidates: MentionTarget[] } | null {
  const at = draft.lastIndexOf('@');
  if (at < 0) return null;
  const tail = draft.slice(at + 1);
  if (tail.includes(' ') || tail.includes('\n')) return null;
  const q = tail.toLowerCase();
  const candidates = targets
    .filter(
      (t) =>
        !q ||
        t.displayName.toLowerCase().includes(q) ||
        t.nodeId.toLowerCase().includes(q)
    )
    .slice(0, 10);
  return { query: tail, candidates };
}

/** Replace the trailing @query with a mention token. */
export function insertMentionToken(draft: string, target: MentionTarget): string {
  const at = draft.lastIndexOf('@');
  if (at < 0) return `${draft}${formatMentionToken(target)} `;
  return `${draft.slice(0, at)}${formatMentionToken(target)} `;
}

/** Human-readable line (strip markdown tokens). */
export function mentionPlainText(content: string): string {
  return content.replace(/@\[([^\]]+)\]\(node:[^)]+\)/g, '@$1');
}

export type MentionSegment =
  | { kind: 'text'; text: string }
  | { kind: 'mention'; displayName: string; nodeId: string };

/**
 * Split message body into plain text and clickable @mention segments.
 */
export function splitMentionContent(content: string): MentionSegment[] {
  const segments: MentionSegment[] = [];
  const re = /@\[([^\]]+)\]\(node:([^)]+)\)/g;
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(content)) !== null) {
    if (m.index > last) {
      segments.push({ kind: 'text', text: content.slice(last, m.index) });
    }
    const nodeId = m[2];
    if (isLikelyPeerNodeId(nodeId)) {
      segments.push({ kind: 'mention', displayName: m[1], nodeId });
    } else {
      segments.push({ kind: 'text', text: m[0] });
    }
    last = m.index + m[0].length;
  }
  if (last < content.length) {
    segments.push({ kind: 'text', text: content.slice(last) });
  }
  if (segments.length === 0) {
    segments.push({ kind: 'text', text: content });
  }
  return segments;
}

/** Resolve a node id from message `mentions` or contacts. */
export function resolveMentionNodeId(
  nodeOrId: string,
  contacts: Contact[]
): MentionTarget | null {
  if (!isLikelyPeerNodeId(nodeOrId)) return null;
  const c = contacts.find((x) => x.node_id === nodeOrId);
  return {
    nodeId: nodeOrId,
    displayName: c?.name ?? nodeOrId.slice(0, 12),
    contactId: c?.contact_id,
  };
}
