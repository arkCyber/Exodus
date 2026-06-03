<!--
  Exodus Browser — WebChat icons for ImMessenger (nav, toolbar, header) using Lucide.
-->
<template>
  <component
    :is="iconComponent"
    :size="size"
    class="im-icon"
    :class="{ 'im-icon--active': active }"
    aria-hidden="true"
  />
</template>

<script setup lang="ts">
/**
 * WebChat icon set for ImMessenger navigation and toolbars using Lucide icons.
 */
import { computed } from 'vue';
import {
  MessageSquare,
  MessageCircle,
  Bookmark,
  Star,
  Users,
  Layers,
  Clock,
  Menu,
  Search,
  Plus,
  Smile,
  Folder,
  Scissors,
  Mic,
  Phone,
  Video,
  MoreHorizontal,
  BellOff,
  Send,
  UserPlus,
  Tag,
  FileText,
  ChevronRight,
  ChevronDown,
} from '@lucide/vue';

export type ImMessengerIconName =
  | 'chat'
  | 'collections'
  | 'starred'
  | 'contacts'
  | 'groups'
  | 'public'
  | 'timeline'
  | 'menu'
  | 'search'
  | 'plus'
  | 'emoji'
  | 'folder'
  | 'scissors'
  | 'chat-history'
  | 'mic'
  | 'phone'
  | 'video'
  | 'more'
  | 'mute'
  | 'send'
  | 'webchat-logo'
  | 'contact-new'
  | 'contact-groups'
  | 'contact-tags'
  | 'contact-official'
  | 'chevron-right'
  | 'chevron-down'
  | 'contacts-manage';

const props = withDefaults(
  defineProps<{
    name: ImMessengerIconName;
    size?: number;
    active?: boolean;
  }>(),
  {
    size: 22,
    active: false,
  },
);

const iconComponent = computed(() => {
  const iconMap: Record<ImMessengerIconName, any> = {
    chat: MessageCircle,
    collections: Bookmark,
    starred: Star,
    contacts: Users,
    groups: Users,
    public: Layers,
    timeline: Clock,
    menu: Menu,
    search: Search,
    plus: Plus,
    emoji: Smile,
    folder: Folder,
    scissors: Scissors,
    'chat-history': MessageSquare,
    mic: Mic,
    phone: Phone,
    video: Video,
    more: MoreHorizontal,
    mute: BellOff,
    send: Send,
    'webchat-logo': MessageCircle,
    'contact-new': UserPlus,
    'contact-groups': Users,
    'contact-tags': Tag,
    'contact-official': FileText,
    'chevron-right': ChevronRight,
    'chevron-down': ChevronDown,
    'contacts-manage': Users,
  };
  return iconMap[props.name] || MessageSquare;
});
</script>

<style scoped>
.im-icon {
  display: block;
  flex-shrink: 0;
}

.im-icon--active {
  color: var(--wx-green, #07c160);
}
</style>
