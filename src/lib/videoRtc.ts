/**
 * Exodus Browser — Video RTC API (1:1 calls + meetings).
 */
import { invoke } from '@tauri-apps/api/core';

export type VideoRtcNodeInfo = {
  nodeId: string;
  displayName: string;
};

export type RtcCallSession = {
  sessionId: string;
  callerNode: string;
  calleeNode: string;
  callerName: string;
  calleeName?: string;
  status: string;
  videoEnabled: boolean;
  audioEnabled: boolean;
  createdAt: number;
  connectedAt?: number;
  endedAt?: number;
};

export type RtcMeetingParticipant = {
  nodeId: string;
  displayName: string;
  joinedAt: number;
  videoEnabled: boolean;
  audioEnabled: boolean;
};

export type RtcMeetingRoom = {
  meetingId: string;
  title: string;
  hostNode: string;
  status: string;
  maxParticipants: number;
  participants: RtcMeetingParticipant[];
  createdAt: number;
};

export async function videoRtcServiceStart(): Promise<VideoRtcNodeInfo> {
  return invoke<VideoRtcNodeInfo>('video_rtc_service_start');
}

export async function videoRtcNodeInfo(): Promise<VideoRtcNodeInfo> {
  return invoke<VideoRtcNodeInfo>('video_rtc_node_info');
}

export async function videoRtcCallStart(
  calleeNode: string,
  calleeName?: string,
  videoEnabled = true,
  audioEnabled = true
): Promise<RtcCallSession> {
  return invoke<RtcCallSession>('video_rtc_call_start', {
    calleeNode,
    calleeName: calleeName ?? null,
    videoEnabled,
    audioEnabled,
  });
}

export async function videoRtcCallUpdate(
  sessionId: string,
  status: string
): Promise<RtcCallSession> {
  return invoke<RtcCallSession>('video_rtc_call_update', { sessionId, status });
}

export async function videoRtcCallList(): Promise<RtcCallSession[]> {
  return invoke<RtcCallSession[]>('video_rtc_call_list');
}

export async function videoRtcMeetingCreate(
  title: string,
  maxParticipants?: number
): Promise<RtcMeetingRoom> {
  return invoke<RtcMeetingRoom>('video_rtc_meeting_create', {
    title,
    maxParticipants: maxParticipants ?? null,
  });
}

export async function videoRtcMeetingJoin(
  meetingId: string,
  displayName?: string
): Promise<RtcMeetingRoom> {
  return invoke<RtcMeetingRoom>('video_rtc_meeting_join', {
    meetingId,
    displayName: displayName ?? null,
  });
}

export async function videoRtcMeetingLeave(meetingId: string): Promise<RtcMeetingRoom> {
  return invoke<RtcMeetingRoom>('video_rtc_meeting_leave', { meetingId });
}

export async function videoRtcMeetingGet(meetingId: string): Promise<RtcMeetingRoom> {
  return invoke<RtcMeetingRoom>('video_rtc_meeting_get', { meetingId });
}

export async function videoRtcMeetingList(): Promise<RtcMeetingRoom[]> {
  return invoke<RtcMeetingRoom[]>('video_rtc_meeting_list');
}

export async function videoRtcPeerTopic(remoteNode: string): Promise<string> {
  return invoke<string>('video_rtc_peer_topic', { remoteNode });
}
