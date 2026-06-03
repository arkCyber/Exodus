<!--
  Exodus Browser — WebChat IM panel (contacts + direct messages).
  Aerospace-grade implementation with robust error handling and type safety.
  Two-level sidebar: Level 1 (Navigation) + Level 2 (Content List)
-->
<template>
  <div class="im-messenger" :class="{ 'dark-mode': isDarkTheme, 'full-width': fullWidth, 'webchat-desktop': isWebChatDesktop }">
    <!-- Level 1 Sidebar: Navigation -->
    <div class="nav-sidebar">
      <div class="nav-header">
        <div class="user-avatar-small">
          <img :src="userAvatar" :alt="localName" />
        </div>
      </div>
      <nav class="nav-menu nav-menu-primary">
        <button
          v-for="item in primaryNavItemsList"
          :key="item.id"
          type="button"
          class="nav-item"
          :class="{ active: activeNav === item.id }"
          :title="navItemTitle(item, isWebChatDesktop)"
          @click="handlePrimaryNavClick(item.id)"
        >
          <ImMessengerIcon
            :name="item.icon"
            :size="24"
            :active="activeNav === item.id && (item.icon === 'chat' || item.icon === 'starred' || item.icon === 'contacts')"
          />
          <span v-if="item.id === 'chats' && totalUnread > 0" class="nav-badge">{{ totalUnread > 99 ? '99+' : totalUnread }}</span>
          <span v-else-if="item.id === 'collections' && collectionCount > 0" class="nav-badge nav-badge-muted">{{ collectionCount > 99 ? '99+' : collectionCount }}</span>
          <span v-else-if="item.id === 'favorites' && favoriteCount > 0" class="nav-badge nav-badge-muted">{{ favoriteCount > 99 ? '99+' : favoriteCount }}</span>
        </button>
      </nav>
      <nav class="nav-menu nav-menu-footer">
        <button
          type="button"
          class="nav-item"
          :class="{ active: activeNav === 'settings' }"
          :title="settingsNavTitle(isWebChatDesktop)"
          @click="activeNav = 'settings'"
        >
          <ImMessengerIcon name="menu" :size="24" />
        </button>
      </nav>
    </div>

    <!-- Level 2 Sidebar: Content List -->
    <div class="content-sidebar">
      <div v-if="isWebChatDesktop && showListSearch" class="webchat-list-toolbar">
        <div class="search-container webchat-search">
          <ImMessengerIcon name="search" :size="16" class="search-icon-svg" />
          <input v-model="search" type="search" class="search-input" :placeholder="searchPlaceholder" />
        </div>
        <button
          v-if="activeNav === 'chats' || activeNav === 'contacts'"
          type="button"
          class="webchat-toolbar-btn"
          :title="isWebChatDesktop ? '添加朋友' : 'Add Contact'"
          @click="showAddContactDialog = true"
        >
          <ImMessengerIcon name="plus" :size="18" />
        </button>
      </div>
      <template v-else>
      <div class="sidebar-header">
        <h2 class="sidebar-title">{{ navTitle }}</h2>
        <div class="sidebar-actions">
          <button v-if="activeNav === 'chats' || activeNav === 'contacts'" type="button" class="icon-button" title="Add Contact" @click="showAddContactDialog = true">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
        </div>
      </div>

      <div v-if="showListSearch" class="search-container">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input v-model="search" type="search" class="search-input" :placeholder="searchPlaceholder" />
      </div>
      </template>

      <!-- Chats List -->
      <ul v-if="activeNav === 'chats'" class="chat-list">
        <template v-if="isWebChatDesktop">
          <li
            v-for="row in webchatChatListRows"
            :key="row.kind === 'contact' ? row.contact.node_id : row.group.groupId"
            class="chat-item"
            :class="{
              active: row.kind === 'contact'
                ? active?.node_id === row.contact.node_id
                : activeGroup?.groupId === row.group.groupId,
            }"
            @click="() => void (row.kind === 'contact' ? selectContact(row.contact) : selectGroup(row.group))"
          >
            <div class="chat-avatar">
              <div
                v-if="row.kind === 'group'"
                class="group-grid-avatar"
                :class="groupGridClass(row.group)"
              >
                <img
                  v-for="(avatarUrl, index) in getGroupGridAvatarUrls(row.group)"
                  :key="`${row.group.groupId}-${index}`"
                  :src="avatarUrl"
                  alt=""
                  loading="lazy"
                />
              </div>
              <template v-else>
                <img :src="getAvatarUrl(row.contact.node_id)" :alt="row.contact.name" loading="lazy" />
                <span v-if="isOnline(row.contact.node_id)" class="online-dot"></span>
              </template>
            </div>
            <div class="chat-info">
              <div class="chat-header-row">
                <span class="chat-name">{{ row.kind === 'contact' ? row.contact.name : row.group.name }}</span>
                <div class="chat-header-actions">
                  <span
                    v-if="isConversationMuted(row.kind === 'contact' ? conversationIdForContact(row.contact.node_id) : conversationIdForGroup(row.group.groupId))"
                    class="mute-indicator"
                    title="消息免打扰"
                    aria-label="消息免打扰"
                  >
                    <ImMessengerIcon name="mute" :size="14" />
                  </span>
                  <span class="chat-time">
                    {{ row.kind === 'contact' ? getLastMessageTime(row.contact.node_id) : formatMessageTime(row.group.lastActivity) }}
                  </span>
                </div>
              </div>
              <div class="chat-preview">
                <span class="preview-text">
                  {{ row.kind === 'contact' ? getLastMessagePreview(row.contact.node_id) : (row.group.description || `${row.group.memberIds.length} members`) }}
                </span>
                <span
                  v-if="row.kind === 'contact' && getUnreadCount(row.contact.node_id) > 0 && !isConversationMuted(conversationIdForContact(row.contact.node_id))"
                  class="unread-badge"
                >
                  {{ getUnreadCount(row.contact.node_id) }}
                </span>
              </div>
            </div>
          </li>
          <li v-if="webchatChatListRows.length === 0" class="empty-state">
            <p>暂无聊天</p>
          </li>
        </template>
        <template v-else>
        <li
          v-for="c in filteredContacts"
          :key="c.node_id"
          class="chat-item"
          :class="{ active: active?.node_id === c.node_id, favorited: c.is_favorite }"
          @click="() => void selectContact(c)"
        >
          <div class="chat-avatar">
            <img :src="getAvatarUrl(c.node_id)" :alt="c.name" loading="lazy" />
            <span v-if="isOnline(c.node_id)" class="online-dot"></span>
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">{{ c.name }}</span>
              <div class="chat-header-actions">
                <button
                  v-if="!isWebChatDesktop"
                  type="button"
                  class="favorite-btn"
                  :class="{ active: c.is_favorite }"
                  :title="c.is_favorite ? 'Remove from Starred' : 'Add to Starred'"
                  @click.stop="() => void toggleContactFavorite(c)"
                >
                  {{ c.is_favorite ? '★' : '☆' }}
                </button>
                <span class="chat-time">{{ getLastMessageTime(c.node_id) }}</span>
              </div>
            </div>
            <div class="chat-preview">
              <span class="preview-text">{{ getLastMessagePreview(c.node_id) }}</span>
              <span v-if="getUnreadCount(c.node_id) > 0" class="unread-badge">{{ getUnreadCount(c.node_id) }}</span>
            </div>
          </div>
        </li>
        <li v-if="filteredContacts.length === 0" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
          </svg>
          <p>No contacts yet</p>
        </li>
        </template>
      </ul>

      <!-- Favorites List -->
      <ul v-else-if="activeNav === 'favorites'" class="chat-list">
        <li
          v-for="c in filteredContacts"
          :key="c.node_id"
          class="chat-item"
          :class="{ active: active?.node_id === c.node_id, favorited: true }"
          @click="() => void selectContact(c)"
        >
          <div class="chat-avatar">
            <img :src="getAvatarUrl(c.node_id)" :alt="c.name" loading="lazy" />
            <span v-if="isOnline(c.node_id)" class="online-dot"></span>
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">{{ c.name }}</span>
              <div class="chat-header-actions">
                <button
                  type="button"
                  class="favorite-btn active"
                  title="Remove from Starred"
                  @click.stop="() => void toggleContactFavorite(c)"
                >
                  ★
                </button>
                <span class="chat-time">{{ getLastMessageTime(c.node_id) }}</span>
              </div>
            </div>
            <div class="chat-preview">
              <span class="preview-text">{{ getLastMessagePreview(c.node_id) }}</span>
              <span v-if="getUnreadCount(c.node_id) > 0" class="unread-badge">{{ getUnreadCount(c.node_id) }}</span>
            </div>
          </div>
        </li>
        <li v-if="filteredContacts.length === 0" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
          </svg>
          <p>No starred contacts yet</p>
          <p class="empty-hint">Star a contact from Chats to add them here.</p>
        </li>
      </ul>

      <!-- Collections (WebChat 收藏) -->
      <ul v-else-if="activeNav === 'collections'" class="chat-list collection-list">
        <li
          v-for="item in filteredCollections"
          :key="item.id"
          class="chat-item collection-item"
          :class="{ active: selectedCollection?.id === item.id }"
          @click="selectedCollection = item"
        >
          <div class="chat-avatar">
            <div class="collection-type-badge">{{ collectionTypeLabel(item.content_type) }}</div>
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">{{ item.sender_name }}</span>
              <span class="chat-time">{{ formatMessageTime(item.saved_at) }}</span>
            </div>
            <div class="chat-preview">
              <span class="preview-text">{{ collectionItemPreview(item) }}</span>
              <button
                type="button"
                class="collection-delete-btn"
                title="Remove from Collections"
                @click.stop="() => void deleteCollectionItem(item)"
              >
                ×
              </button>
            </div>
            <div class="collection-meta">{{ item.conversation_title }}</div>
          </div>
        </li>
        <li v-if="filteredCollections.length === 0" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
          </svg>
          <p>No saved messages yet</p>
          <p class="empty-hint">Long-press a message and choose 收藏 to save it here.</p>
        </li>
      </ul>

      <!-- Contacts List -->
      <ul v-else-if="activeNav === 'contacts'" class="chat-list contact-directory-list">
        <template v-if="isWebChatDesktop && !search.trim()">
          <li class="contact-manage-row">
            <button type="button" class="contact-manage-btn" @click="showContactManageMenu = !showContactManageMenu">
              <ImMessengerIcon name="contacts-manage" :size="18" />
              <span>通讯录管理</span>
            </button>
          </li>
          <li v-if="showContactManageMenu" class="contact-manage-actions">
            <button type="button" class="contact-manage-action" @click="() => void exportContactsJson()">导出通讯录</button>
            <button type="button" class="contact-manage-action" @click="triggerContactImport">导入通讯录</button>
            <input
              ref="contactImportInputRef"
              type="file"
              accept=".json,application/json"
              class="hidden-input"
              @change="handleContactImportFile"
            />
          </li>
          <template v-for="cat in contactDirectoryCategories" :key="cat.id">
            <li class="contact-category-row" @click="handleContactCategoryClick(cat.id)">
              <ImMessengerIcon
                :name="isContactCategoryExpanded(cat.id) ? 'chevron-down' : 'chevron-right'"
                :size="14"
                class="category-chevron"
              />
              <span class="category-label">{{ contactDirectoryCategoryLabel(cat, isWebChatDesktop) }}</span>
              <span v-if="cat.showCount" class="category-count">
                {{ formatContactDirectoryCount(contactDirectoryCountForCategory(cat.id, contactDirectoryCounts) ?? 0) }}
              </span>
            </li>
            <template v-if="isContactCategoryExpanded(cat.id) && cat.expandable">
              <template v-if="cat.id === 'group_chats'">
                <li
                  v-for="g in realGroupChats"
                  :key="`dir-group-${g.groupId}`"
                  class="contact-nested-item"
                  @click.stop="() => void selectGroup(g)"
                >
                  <div class="chat-avatar chat-avatar-small">
                    <div class="group-grid-avatar" :class="groupGridClass(g)">
                      <img
                        v-for="(avatarUrl, index) in getGroupGridAvatarUrls(g)"
                        :key="`${g.groupId}-${index}`"
                        :src="avatarUrl"
                        alt=""
                        loading="lazy"
                      />
                    </div>
                  </div>
                  <span class="nested-item-name">{{ g.name }}</span>
                </li>
                <li v-if="realGroupChats.length === 0" class="contact-nested-empty">暂无群聊</li>
              </template>
              <template v-else-if="cat.id === 'official_accounts'">
                <li
                  v-for="account in subscribedPublicAccountList"
                  :key="`dir-pa-${account.account_id}`"
                  class="contact-nested-item"
                  @click.stop="() => void selectPublicAccount(account)"
                >
                  <div class="chat-avatar chat-avatar-small">
                    <img :src="account.avatar_url || getAvatarUrl(account.account_id)" :alt="account.name" loading="lazy" />
                  </div>
                  <span class="nested-item-name">{{ account.name }}</span>
                </li>
                <li v-if="subscribedPublicAccountList.length === 0" class="contact-nested-empty">暂无公众号</li>
              </template>
              <template v-else-if="cat.id === 'contacts'">
                <li
                  v-for="c in filteredContacts"
                  :key="`dir-contact-${c.node_id}`"
                  class="contact-nested-item"
                  :class="{ active: active?.node_id === c.node_id }"
                  @click.stop="() => void selectContact(c)"
                >
                  <div class="chat-avatar chat-avatar-small">
                    <img :src="getAvatarUrl(c.node_id)" :alt="c.name" loading="lazy" />
                    <span v-if="isOnline(c.node_id)" class="online-dot"></span>
                  </div>
                  <span class="nested-item-name">{{ c.name }}</span>
                </li>
                <li v-if="filteredContacts.length === 0" class="contact-nested-empty">暂无联系人</li>
              </template>
              <template v-else>
                <li class="contact-nested-empty">暂无内容</li>
              </template>
            </template>
          </template>
        </template>
        <template v-else>
        <li
          v-for="c in filteredContacts"
          :key="c.node_id"
          class="chat-item contact-item"
          :class="{ active: active?.node_id === c.node_id, favorited: c.is_favorite, blocked: c.is_blocked }"
        >
          <div class="chat-avatar" @click="() => void selectContact(c)">
            <img :src="getAvatarUrl(c.node_id)" :alt="c.name" loading="lazy" />
            <span v-if="isOnline(c.node_id)" class="online-dot"></span>
          </div>
          <div class="chat-info" @click="() => void selectContact(c)">
            <div class="chat-header-row">
              <span class="chat-name">{{ c.name }}</span>
              <div class="chat-header-actions">
                <button
                  v-if="!isWebChatDesktop"
                  type="button"
                  class="favorite-btn"
                  :class="{ active: c.is_favorite }"
                  :title="c.is_favorite ? 'Remove from Starred' : 'Add to Starred'"
                  @click.stop="() => void toggleContactFavorite(c)"
                >
                  {{ c.is_favorite ? '★' : '☆' }}
                </button>
              </div>
            </div>
            <div class="chat-preview">
              <span class="preview-text">{{ c.node_id.slice(0, 16) }}…</span>
              <span v-if="c.is_blocked" class="blocked-badge">Blocked</span>
            </div>
          </div>
          <div class="contact-actions">
            <button
              type="button"
              class="action-btn"
              title="Start Chat"
              @click.stop="() => void selectContact(c)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
              </svg>
            </button>
            <button
              type="button"
              class="action-btn"
              title="Voice Call"
              @click.stop="() => void startVoiceCall(c)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"></path>
              </svg>
            </button>
            <button
              type="button"
              class="action-btn"
              title="Edit Contact"
              @click.stop="() => void openEditContactDialog(c)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
              </svg>
            </button>
            <button
              type="button"
              class="action-btn delete-btn"
              title="Delete Contact"
              @click.stop="() => void deleteContact(c)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"></polyline>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
              </svg>
            </button>
          </div>
        </li>
        <li v-if="filteredContacts.length === 0 && !(isWebChatDesktop && !search.trim())" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
          </svg>
          <p>{{ isWebChatDesktop ? '暂无联系人' : 'No contacts yet' }}</p>
          <p class="empty-hint">{{ isWebChatDesktop ? '点击 + 添加朋友' : 'Click the + button to add a contact' }}</p>
        </li>
        </template>
      </ul>

      <!-- Groups List -->
      <ul v-else-if="activeNav === 'groups'" class="chat-list">
        <li class="chat-item" @click="showCreateGroup = true">
          <div class="chat-avatar">
            <ImMessengerIcon name="plus" :size="24" />
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">Create Group</span>
            </div>
            <div class="chat-preview">
              <span class="preview-text">Start a new group chat</span>
            </div>
          </div>
        </li>
        <li v-for="g in groups" :key="g.groupId" class="chat-item" :class="{ active: activeGroup?.groupId === g.groupId }" @click="() => void selectGroup(g)">
          <div class="chat-avatar">
            <div v-if="isWebChatDesktop && isRealGroupChat(g.groupId)" class="group-grid-avatar" :class="groupGridClass(g)">
              <img
                v-for="(avatarUrl, index) in getGroupGridAvatarUrls(g)"
                :key="`${g.groupId}-${index}`"
                :src="avatarUrl"
                :alt="g.name"
                loading="lazy"
              />
            </div>
            <img v-else :src="getGroupAvatarUrl(g)" :alt="g.name" loading="lazy" />
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">{{ g.name }}</span>
              <div class="chat-header-actions">
                <span v-if="isConversationMuted(conversationIdForGroup(g.groupId))" class="mute-indicator" title="消息免打扰" aria-label="消息免打扰">
                  <ImMessengerIcon name="mute" :size="14" />
                </span>
                <span class="chat-time">{{ formatMessageTime(g.lastActivity) }}</span>
              </div>
            </div>
            <div class="chat-preview">
              <span class="preview-text">{{ g.description || `${g.memberIds.length} members` }}</span>
            </div>
          </div>
        </li>
        <li v-if="groups.length === 0" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
          </svg>
          <p>No groups yet</p>
        </li>
      </ul>

      <!-- Public Accounts List -->
      <ul v-else-if="activeNav === 'public_accounts'" class="chat-list">
        <li class="chat-item" @click="showPublicAccountSearch = true">
          <div class="chat-avatar">
            <ImMessengerIcon name="search" :size="24" />
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">Search Public Accounts</span>
            </div>
            <div class="chat-preview">
              <span class="preview-text">Find and subscribe to public accounts</span>
            </div>
          </div>
        </li>
        <li
          v-for="account in publicAccounts"
          :key="account.account_id"
          class="chat-item public-account-item"
          :class="{ active: activePublicAccount?.account_id === account.account_id, subscribed: subscribedAccounts.includes(account.account_id) }"
          @click="() => void selectPublicAccount(account)"
        >
          <div class="chat-avatar">
            <img :src="account.avatar_url || getAvatarUrl(account.account_id)" :alt="account.name" loading="lazy" />
            <span v-if="account.is_verified" class="verified-badge" title="Verified">✓</span>
          </div>
          <div class="chat-info">
            <div class="chat-header-row">
              <span class="chat-name">{{ account.name }}</span>
              <div class="chat-header-actions">
                <button
                  type="button"
                  class="subscribe-btn"
                  :class="{ active: subscribedAccounts.includes(account.account_id) }"
                  :title="subscribedAccounts.includes(account.account_id) ? 'Unsubscribe' : 'Subscribe'"
                  @click.stop="() => void toggleSubscribe(account)"
                >
                  {{ subscribedAccounts.includes(account.account_id) ? '✓' : '+' }}
                </button>
              </div>
            </div>
            <div class="chat-preview">
              <span class="preview-text">{{ account.description || account.category }}</span>
              <span class="follower-count">{{ account.follower_count }} followers</span>
            </div>
          </div>
        </li>
        <li v-if="publicAccounts.length === 0" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
          </svg>
          <p>No public accounts yet</p>
          <p class="empty-hint">Search to discover public accounts</p>
        </li>
      </ul>

      <!-- Timeline -->
      <div v-else-if="activeNav === 'timeline'" class="timeline-container">
        <SocialTimeline @status="onStatus" />
      </div>

      <!-- Settings -->
      <div v-else-if="activeNav === 'settings'" class="settings-list">
        <div class="settings-item" @click="toggleTheme">
          <span class="settings-label">Theme</span>
          <span class="settings-value">{{ settings.theme }}</span>
        </div>
        <div class="settings-item" @click="toggleNotifications">
          <span class="settings-label">Notifications</span>
          <span class="settings-value">{{ settings.notifications ? 'On' : 'Off' }}</span>
        </div>
        <div class="settings-item" @click="toggleSound">
          <span class="settings-label">Sound</span>
          <span class="settings-value">{{ settings.sound ? 'On' : 'Off' }}</span>
        </div>
        <p v-if="myDigit" class="settings-item settings-item-static">
          <span class="settings-label">My 12-digit ID</span>
          <button type="button" class="link-button" @click="() => void copyMyDigit()">{{ myDigit }}</button>
        </p>
      </div>
    </div>

    <!-- Right main: Chat window -->
    <div v-if="active || activeGroup" class="chat-main">
      <header class="chat-window-header" :class="{ 'webchat-header': isWebChatDesktop }">
        <div class="header-left">
          <div v-if="!isWebChatDesktop" class="header-avatar">
            <img :src="active ? getAvatarUrl(active.node_id) : (activeGroup ? getGroupAvatarUrl(activeGroup) : '')" :alt="active?.name || activeGroup?.name" loading="lazy" />
            <span v-if="active && isOnline(active.node_id)" class="online-indicator"></span>
          </div>
          <div class="header-info">
            <h3 class="header-name">{{ active?.name || activeGroup?.name }}</h3>
            <p v-if="!isWebChatDesktop" class="header-status">{{ active ? (isOnline(active.node_id) ? 'Online' : 'Offline') : `${activeGroup?.memberIds.length || 0} members` }}</p>
          </div>
        </div>
        <div class="header-actions">
          <button v-if="active && !isWebChatDesktop" type="button" class="icon-button" title="Voice Call" @click="voiceCall">
            <ImMessengerIcon name="phone" :size="20" />
          </button>
          <button v-if="active && !isWebChatDesktop" type="button" class="icon-button" title="Video Call" @click="videoCall">
            <ImMessengerIcon name="video" :size="20" />
          </button>
          <button v-if="!isWebChatDesktop" type="button" class="icon-button" title="Search" @click="showSearch = !showSearch">
            <ImMessengerIcon name="search" :size="20" />
          </button>
          <button type="button" class="icon-button" :title="isWebChatDesktop ? '更多' : 'More'" @click="showConversationMenu = !showConversationMenu">
            <ImMessengerIcon name="more" :size="20" />
          </button>
          <div v-if="showConversationMenu" class="conversation-menu">
            <button type="button" class="conversation-menu-item" @click="toggleActiveConversationMute">
              {{ isActiveConversationMuted() ? '关闭消息免打扰' : '消息免打扰' }}
            </button>
            <button
              v-if="activeGroup"
              type="button"
              class="conversation-menu-item"
              @click="showGroupSettings = true; showConversationMenu = false"
            >
              群聊信息
            </button>
          </div>
        </div>
      </header>

      <!-- Search Bar -->
      <div v-if="showSearch" class="search-bar">
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="Search messages..."
          @input="performSearch"
        />
        <div v-if="searchResults.length > 0" class="search-results-info">
          {{ searchResults.length }} results found
          <button type="button" class="search-nav-btn" @click="prevSearchResult" :disabled="searchIndex <= 0">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
          </button>
          <span>{{ searchIndex + 1 }} / {{ searchResults.length }}</span>
          <button type="button" class="search-nav-btn" @click="nextSearchResult" :disabled="searchIndex >= searchResults.length - 1">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
        </div>
        <button type="button" class="close-search-btn" @click="showSearch = false">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <div class="messages-container" ref="messagesContainer">
        <div v-if="loading" class="loading-state">
          <div class="spinner"></div>
          <p>Loading messages...</p>
        </div>
        <div v-else-if="messages.length === 0" class="empty-chat">
          <div class="empty-chat-content">
            <div class="empty-chat-avatar">
              <img :src="getAvatarUrl(active?.node_id || '')" :alt="active?.name || ''" />
            </div>
            <h3>{{ active?.name }}</h3>
            <p>开始与 {{ active?.name }} 聊天吧</p>
            <div class="quick-actions">
              <button type="button" class="quick-action-btn" @click="draft = '你好！'">
                👋 打个招呼
              </button>
              <button type="button" class="quick-action-btn" @click="draft = '最近怎么样？'">
                💬 问候近况
              </button>
            </div>
          </div>
        </div>
        <div v-else class="messages-list">
          <template v-for="item in messageTimelineItems" :key="item.key">
            <div v-if="item.kind === 'divider'" class="message-time-divider">
              <span>{{ item.label }}</span>
            </div>
            <div
              v-else
              class="message-wrapper"
              :class="{ own: isOwnMessage(item.message) }"
            >
              <div class="message-avatar">
                <img :src="getAvatarUrl(item.message.senderId)" :alt="item.message.senderId" loading="lazy" />
              </div>
              <div class="message-content">
                <div
                  class="message-bubble"
                  @contextmenu.prevent="showContextMenu($event, item.message)"
                  @touchstart="handleTouchStart($event, item.message)"
                  @touchend="handleTouchEnd"
                >
                  <div v-if="item.message.replyTo" class="message-reply-quote">
                    {{ getMessageReplyPreview(item.message) }}
                  </div>
                  <MentionMessageBody
                    v-if="hasMentionTokens(item.message.content)"
                    :content="item.message.content"
                    class="message-text"
                    @mention-action="handleMentionAction"
                  />
                  <p v-else class="message-text" v-html="sanitizeMessage(item.message.content)"></p>
                  <div v-if="!isWebChatDesktop" class="message-meta">
                    <span class="message-time">{{ formatMessageTime(item.message.timestamp) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </template>
        </div>

        <!-- Context Menu -->
        <div v-if="contextMenu.visible" class="context-menu" :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
          <button type="button" class="context-menu-item" :disabled="contextMenu.saved" @click="() => void saveMessageToCollection()">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
            </svg>
            {{ contextMenu.saved ? '已收藏' : '收藏' }}
          </button>
          <button type="button" class="context-menu-item" @click="copyMessage">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            复制
          </button>
          <button type="button" class="context-menu-item" @click="startReplyMessage">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 17 4 12 9 7"></polyline>
              <path d="M20 18v-2a4 4 0 0 0-4-4H4"></path>
            </svg>
            回复
          </button>
          <button v-if="contextMenu.message && isOwnMessage(contextMenu.message) && canRecallMessage(contextMenu.message)" type="button" class="context-menu-item" @click="() => void recallMessage()">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 2v6h6"></path>
              <path d="M21 12A9 9 0 0 0 6 5.3L3 8"></path>
              <path d="M21 22v-6h-6"></path>
              <path d="M3 12a9 9 0 0 0 15 6.7l3-2.7"></path>
            </svg>
            撤回
          </button>
          <button v-if="contextMenu.message && isOwnMessage(contextMenu.message)" type="button" class="context-menu-item" @click="startEditMessage">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
            </svg>
            编辑
          </button>
          <button v-if="contextMenu.message && isOwnMessage(contextMenu.message)" type="button" class="context-menu-item" @click="() => void deleteMessage()">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"></polyline>
              <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            </svg>
            删除
          </button>
          <button type="button" class="context-menu-item" @click="hideContextMenu">
            取消
          </button>
        </div>
      </div>

      <div class="input-area">
        <div class="input-toolbar">
          <button type="button" class="icon-button" :title="isWebChatDesktop ? '表情' : 'Emoji'" @click="showEmojiPicker = !showEmojiPicker">
            <ImMessengerIcon name="emoji" :size="20" />
          </button>
          <button type="button" class="icon-button" :title="isWebChatDesktop ? '发送文件' : 'Attach File'" @click="triggerFileUpload">
            <ImMessengerIcon name="folder" :size="20" />
          </button>
          <input
            ref="fileInputRef"
            type="file"
            multiple
            accept="image/*,.pdf,.doc,.docx,.txt"
            style="display: none"
            @change="handleFileSelect"
          />
          <button type="button" class="icon-button" :title="isWebChatDesktop ? '截图' : 'Screenshot'" disabled>
            <ImMessengerIcon name="scissors" :size="20" />
          </button>
          <button type="button" class="icon-button" :title="isWebChatDesktop ? '聊天记录' : 'Chat History'" @click="showSearch = !showSearch">
            <ImMessengerIcon name="chat-history" :size="20" />
          </button>
          <button type="button" class="icon-button" :title="isWebChatDesktop ? '语音' : 'Voice Message'" disabled>
            <ImMessengerIcon name="mic" :size="20" />
          </button>
        </div>
        <form class="message-form webchat-message-form" @submit.prevent="handleSubmit">
          <div v-if="selectedFiles.length > 0" class="file-preview-bar">
            <div class="file-preview-list">
              <div v-for="(fileItem, index) in selectedFiles" :key="index" class="file-preview-item">
                <div v-if="fileItem.preview" class="file-image-preview">
                  <img :src="fileItem.preview" :alt="fileItem.file.name" />
                </div>
                <div v-else class="file-icon-preview">
                  <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
                    <polyline points="13 2 13 9 20 9"></polyline>
                  </svg>
                </div>
                <span class="file-name">{{ fileItem.file.name }}</span>
                <button type="button" class="remove-file-btn" @click="removeFile(index)" title="Remove">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <div v-if="replyingTo" class="reply-indicator">
            <div class="reply-content">
              <span class="reply-label">回复 {{ replyingTo.senderName }}</span>
              <span class="reply-text">{{ replyingTo.content }}</span>
            </div>
            <button type="button" class="cancel-reply-btn" @click="cancelReplyMessage" title="取消回复">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
          <div v-if="editingMessage" class="edit-indicator">
            <span>编辑消息</span>
            <button type="button" class="cancel-edit-btn" @click="cancelEditMessage" title="取消编辑">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
          <textarea
            ref="messageInput"
            v-model="messageDraft"
            class="message-input"
            :placeholder="messageInputPlaceholder"
            rows="1"
            @input="handleInput"
            @keydown.enter.prevent="handleEnterKey"
            @keydown="handleKeyDown"
            @blur="hideMentionAutocomplete"
            :disabled="loading"
          ></textarea>
          <div v-if="showEmojiPicker" class="emoji-picker">
            <div class="emoji-grid">
              <button
                v-for="emoji in commonEmojis"
                :key="emoji"
                type="button"
                class="emoji-button"
                @click="insertEmoji(emoji)"
              >
                {{ emoji }}
              </button>
            </div>
          </div>
          <div v-if="showMentionAutocomplete" class="mention-autocomplete">
            <div
              v-for="(suggestion, index) in mentionSuggestions"
              :key="suggestion.id"
              class="mention-item"
              :class="{ active: index === mentionIndex }"
              @click="selectMention(suggestion)"
            >
              <img :src="getAvatarUrl(suggestion.id)" :alt="suggestion.name" class="mention-avatar" />
              <span class="mention-name">{{ suggestion.name }}</span>
            </div>
          </div>
          <button
            type="submit"
            class="send-button"
            :class="{ 'webchat-send-button': isWebChatDesktop }"
            :disabled="!(editingMessage ? editDraft.trim() : draft.trim()) || loading"
            :title="editingMessage ? '保存' : (isWebChatDesktop ? '发送(S)' : 'Send')"
          >
            <span v-if="isWebChatDesktop">{{ editingMessage ? '保存' : '发送(S)' }}</span>
            <ImMessengerIcon v-else name="send" :size="20" />
          </button>
        </form>
      </div>
    </div>

    <!-- Empty state when no chat selected -->
    <div v-else class="empty-main" :class="{ 'webchat-empty-main': isWebChatDesktop }">
      <div class="empty-content">
        <div v-if="isWebChatDesktop" class="webchat-empty-logo" aria-hidden="true">
          <ImMessengerIcon name="webchat-logo" :size="96" />
        </div>
        <svg v-else width="96" height="96" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
        </svg>
        <h2>{{ isWebChatDesktop ? 'WebChat' : 'Select a chat' }}</h2>
        <p>{{ isWebChatDesktop ? '选择一个聊天开始对话' : 'Choose a contact or group from the list to start messaging' }}</p>
      </div>
    </div>

    <!-- Create Group Dialog -->
    <div v-if="showCreateGroup" class="modal-overlay" @click.self="showCreateGroup = false">
      <div class="modal-content">
        <h3>Create Group</h3>
        <div class="form-group">
          <label>Group Name</label>
          <input v-model="newGroupName" type="text" class="form-input" placeholder="Enter group name" />
        </div>
        <div class="form-group">
          <label>Description (Optional)</label>
          <input v-model="newGroupDescription" type="text" class="form-input" placeholder="Enter description" />
        </div>
        <div class="form-group">
          <label>Select Members</label>
          <div class="member-selection">
            <div v-for="c in contacts" :key="c.node_id" class="member-item" @click="toggleGroupMember(c.node_id)">
              <input type="checkbox" :checked="selectedGroupMembers.includes(c.node_id)" class="member-checkbox" />
              <img :src="getAvatarUrl(c.node_id)" :alt="c.name" class="member-avatar" />
              <span class="member-name">{{ c.name }}</span>
            </div>
          </div>
        </div>
        <div class="modal-actions">
          <button type="button" class="secondary-button" @click="showCreateGroup = false">Cancel</button>
          <button type="button" class="primary-button" @click="() => void createGroup()" :disabled="!newGroupName.trim() || selectedGroupMembers.length === 0">Create</button>
        </div>
      </div>
    </div>

    <!-- Group Settings Dialog -->
    <div v-if="showGroupSettings && activeGroup" class="modal-overlay" @click.self="showGroupSettings = false">
      <div class="modal-content group-settings-content">
        <h3>Group Settings</h3>
        <div class="group-info-section">
          <div class="group-avatar-large">
            <img :src="getGroupAvatarUrl(activeGroup)" :alt="activeGroup.name" />
          </div>
          <div class="group-details">
            <h4>{{ activeGroup.name }}</h4>
            <p>{{ activeGroup.description || 'No description' }}</p>
            <p class="group-meta">{{ activeGroup.memberIds.length }} members • Created {{ formatMessageTime(activeGroup.createdAt) }}</p>
          </div>
        </div>
        <div class="form-group">
          <label>Group Members</label>
          <div class="member-list">
            <div v-for="member in groupMembers" :key="member.agentId" class="member-list-item">
              <img :src="getAvatarUrl(member.agentId)" :alt="member.agentName" class="member-avatar" />
              <div class="member-info">
                <span class="member-name">{{ member.agentName }}</span>
                <span class="member-role">{{ member.role }}</span>
              </div>
              <span v-if="member.isOnline" class="online-dot"></span>
            </div>
          </div>
        </div>
        <div class="modal-actions">
          <button type="button" class="secondary-button" @click="showGroupSettings = false">Close</button>
          <button type="button" class="danger-button" @click="() => void leaveGroup()">Leave Group</button>
        </div>
      </div>
    </div>

    <!-- Add Contact Dialog -->
    <div v-if="showAddContactDialog" class="modal-overlay" @click.self="showAddContactDialog = false">
      <div class="modal-content">
        <h3>Add Contact</h3>
        <p v-if="myDigit" class="form-hint">
          My 12-digit ID:
          <code class="digit-code">{{ myDigit }}</code>
          <button type="button" class="link-button" @click="() => void copyMyDigit()">Copy</button>
        </p>
        <div class="form-tabs">
          <button
            type="button"
            class="tab-button"
            :class="{ active: addContactMode === 'digit' }"
            @click="addContactMode = 'digit'"
          >
            By 12-digit ID
          </button>
          <button
            type="button"
            class="tab-button"
            :class="{ active: addContactMode === 'manual' }"
            @click="addContactMode = 'manual'"
          >
            By Node ID
          </button>
        </div>
        <div class="form-group">
          <label>Name</label>
          <input v-model="addContactName" type="text" class="form-input" placeholder="Enter contact name" />
        </div>
        <div v-if="addContactMode === 'digit'" class="form-group">
          <label>12-digit Exodus ID</label>
          <input
            v-model="addContactDigit"
            type="text"
            class="form-input"
            maxlength="12"
            inputmode="numeric"
            placeholder="Enter 12-digit ID"
          />
        </div>
        <template v-else>
          <div class="form-group">
            <label>Node ID</label>
            <input v-model="addContactNode" type="text" class="form-input" placeholder="Enter node ID" />
          </div>
          <div class="form-group">
            <label>Notes (Optional)</label>
            <input v-model="addContactNotes" type="text" class="form-input" placeholder="Add notes about this contact" />
          </div>
        </template>
        <div class="modal-actions">
          <button type="button" class="secondary-button" @click="showAddContactDialog = false">Cancel</button>
          <button
            v-if="addContactMode === 'digit'"
            type="button"
            class="primary-button"
            @click="() => void addContactByDigit()"
            :disabled="addContactDigit.replace(/\D/g, '').length !== 12"
          >
            Add Friend
          </button>
          <button
            v-else
            type="button"
            class="primary-button"
            @click="() => void addContact()"
            :disabled="!addContactName.trim() || !addContactNode.trim()"
          >
            Add
          </button>
        </div>
      </div>
    </div>

    <!-- Edit Contact Dialog -->
    <div v-if="showEditContactDialog && editingContact" class="modal-overlay" @click.self="showEditContactDialog = false">
      <div class="modal-content">
        <h3>Edit Contact</h3>
        <div class="form-group">
          <label>Name</label>
          <input v-model="editContactName" type="text" class="form-input" placeholder="Enter contact name" />
        </div>
        <div class="form-group">
          <label>Notes (Optional)</label>
          <input v-model="editContactNotes" type="text" class="form-input" placeholder="Add notes about this contact" />
        </div>
        <div class="form-group">
          <label>
            <input v-model="editContactBlocked" type="checkbox" />
            Block this contact
          </label>
        </div>
        <div class="modal-actions">
          <button type="button" class="secondary-button" @click="showEditContactDialog = false">Cancel</button>
          <button type="button" class="primary-button" @click="() => void saveContactEdit()">Save</button>
        </div>
      </div>
    </div>

    <!-- Public Account Search Dialog -->
    <div v-if="showPublicAccountSearch" class="modal-overlay" @click.self="showPublicAccountSearch = false">
      <div class="modal-content">
        <h3>Search Public Accounts</h3>
        <div class="form-group">
          <input v-model="publicAccountSearchQuery" type="text" class="form-input" placeholder="Search by name or category" @keydown.enter="() => void searchPublicAccounts()" />
        </div>
        <div class="modal-actions">
          <button type="button" class="secondary-button" @click="showPublicAccountSearch = false">Cancel</button>
          <button type="button" class="primary-button" @click="() => void searchPublicAccounts()" :disabled="!publicAccountSearchQuery.trim()">Search</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import type { Contact } from '$lib/contactDirectory';
import {
  buildHumanContact,
  contactAdd,
  contactAddFriendByDigit,
  contactDirectoryServiceStart,
  contactExportJson,
  contactGetLocalDigit,
  contactImportJson,
  contactList,
  contactRemove,
  contactToggleFavorite,
  contactUpdate,
  downloadContactExport,
  touchContactLastContacted,
} from '$lib/contactDirectory';
import {
  IM_OPEN_CONTACT_EVENT,
  dmRoomId,
  ensureDmGroup,
  loadDmMessages,
  notifyImNewMessage,
  openImChat,
  sendDmText,
  startCallFromUi,
  type ImOpenContactDetail,
  applyMessageCacheUpdate,
} from '$lib/imChat';
import {
  imStore,
  setActiveContactNode,
  setActiveGroupId,
  setActiveNav,
  getStoreMessages,
  setStoreMessages,
  getStoreDraft,
  setStoreDraft,
  getMessageCacheEntry,
  setMessageCacheEntry,
} from '$lib/imStore';
import { ensureImMessageSync } from '$lib/imMessageSync';
import {
  buildGroupPayload,
  groupChatServiceStart,
  groupCreate,
  groupDeleteMessage,
  groupEditMessage,
  groupGetMembers,
  groupGetMessages,
  groupListUser,
  groupRemoveMember,
  type GroupChat,
  type GroupMember,
  type GroupMessage,
} from '$lib/groupChat';
import { 
  sendGroupMessageWithCdn,
  prepareBrowserFileAttachment,
  type GroupChatMessage,
  type GroupMessageAttachment,
} from '$lib/p2p/cdnIntegrations';
import { extractMentionNodeIds, type MentionTarget } from '$lib/groupMentions';
import { resolveLocalIdentity } from '$lib/imSession';
import SocialTimeline from './SocialTimeline.vue';
import ImMessengerIcon from './ImMessengerIcon.vue';
import MentionMessageBody from './MentionMessageBody.vue';
import { startPresenceHeartbeat, stopPresenceHeartbeat, fetchOnlinePeers, isNodeOnline, type PresenceEntry } from '$lib/presence';
import { logInfo, logWarn, logError } from '@/lib/logger';
import {
  buildSaveChatItemRequest,
  chatCollectionDelete,
  chatCollectionList,
  chatCollectionSave,
  chatCollectionIsSaved,
  collectionItemPreview,
  type SavedChatItem,
} from '$lib/chatCollections';
import {
  publicAccountServiceStart,
  publicAccountList,
  publicAccountSubscribe,
  publicAccountUnsubscribe,
  publicAccountGetSubscriptions,
  publicAccountListArticles,
  publicAccountSearch,
  type PublicAccount,
  type Article,
} from '$lib/publicAccount';
import {
  buildMessageTimelineItems,
  contactDirectoryCategoryLabel,
  contactDirectoryCountForCategory,
  CONTACT_DIRECTORY_CATEGORIES,
  conversationIdForContact,
  conversationIdForGroup,
  findReplySourceMessage,
  formatContactDirectoryCount,
  groupGridCountClass,
  isRealGroupChat,
  loadContactDirectoryExpanded,
  loadMutedChatIds,
  navItemTitle,
  pickGroupGridMemberIds,
  primaryNavItems,
  replyQuotePreview,
  saveContactDirectoryExpanded,
  saveMutedChatIds,
  settingsNavTitle,
  toggleContactDirectoryExpanded,
  type ContactDirectoryCategoryId,
  type ImNavId,
  type MessageTimelineItem,
} from '$lib/imMessengerWebchat';

const props = withDefaults(defineProps<{ fullWidth?: boolean }>(), { fullWidth: false });

const emit = defineEmits<{ status: [message: string] }>();

// Helper function to normalize errors
function normalizeError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e));
}

const localUserId = ref('exodus-local-user');
const localName = ref('You');
const localNode = ref('');
const contacts = ref<Contact[]>([]);
const active = computed(() => {
  if (!imStore.activeContactNodeId) return null;
  return contacts.value.find((c) => c.node_id === imStore.activeContactNodeId) ?? null;
});
const messages = computed({
  get(): GroupMessage[] {
    const id = activeConversationId();
    return id ? getStoreMessages(id) : [];
  },
  set(msgs: GroupMessage[]) {
    const id = activeConversationId();
    if (id) setStoreMessages(id, msgs);
  },
});
const draft = computed({
  get(): string {
    const id = activeConversationId();
    return id ? getStoreDraft(id) : '';
  },
  set(value: string) {
    const id = activeConversationId();
    if (id) setStoreDraft(id, value);
  },
});
const search = ref('');
const loading = ref(false);
const messagesContainer = ref<HTMLElement | null>(null);
const messageInput = ref<HTMLTextAreaElement | null>(null);

// Contact management state
const showAddContactDialog = ref(false);
const addContactMode = ref<'digit' | 'manual'>('digit');
const addContactName = ref('');
const addContactDigit = ref('');
const addContactNode = ref('');
const addContactNotes = ref('');
const myDigit = ref('');
const showEditContactDialog = ref(false);
const editingContact = ref<Contact | null>(null);
const editContactName = ref('');
const editContactNotes = ref('');
const editContactBlocked = ref(false);

// Group chat state
const groups = ref<GroupChat[]>([]);
const activeGroup = computed(() => {
  if (!imStore.activeGroupId) return null;
  return groups.value.find((g) => g.groupId === imStore.activeGroupId) ?? null;
});
const groupMembers = ref<GroupMember[]>([]);
const showCreateGroup = ref(false);
const newGroupName = ref('');
const newGroupDescription = ref('');
const selectedGroupMembers = ref<string[]>([]);
const showGroupSettings = ref(false);

// Public account state
const publicAccounts = ref<PublicAccount[]>([]);
const subscribedAccounts = ref<string[]>([]);
const activePublicAccount = ref<PublicAccount | null>(null);
const publicAccountArticles = ref<Article[]>([]);
const showPublicAccountSearch = ref(false);
const publicAccountSearchQuery = ref('');


// Context menu state
const contextMenu = ref<{ visible: boolean; x: number; y: number; message: GroupMessage | null; saved: boolean }>({
  visible: false,
  x: 0,
  y: 0,
  message: null,
  saved: false,
});

// Message editing state
const editingMessage = ref<GroupMessage | null>(null);
const editDraft = ref('');

// Message reply state
const replyingTo = ref<GroupMessage | null>(null);

// Mention autocomplete state
const showMentionAutocomplete = ref(false);
const mentionQuery = ref('');
const mentionSuggestions = ref<Array<{ id: string; name: string }>>([]);
const mentionIndex = ref(0);
const mentionStartIndex = ref(0);

// Emoji picker state
const showEmojiPicker = ref(false);
const commonEmojis = ['😀', '😂', '😍', '🥰', '😎', '🤔', '😢', '😡', '👍', '👎', '❤️', '🔥', '✨', '🎉', '👋', '🙏', '💪', '🤝', '👀', '💯'];

// File attachment state
const fileInputRef = ref<HTMLInputElement | null>(null);
const selectedFiles = ref<Array<{ file: File; preview?: string }>>([]);

// Search state
const showSearch = ref(false);
const showConversationMenu = ref(false);
const mutedChatIds = ref<Set<string>>(new Set());
const searchQuery = ref('');
const searchResults = ref<GroupMessage[]>([]);
const searchIndex = ref(0);

// Navigation state for two-level sidebar
const activeNav = computed({
  get: () => imStore.activeNav,
  set: (nav: typeof imStore.activeNav) => setActiveNav(nav),
});

// Collections (WebChat 收藏) state
const savedItems = ref<SavedChatItem[]>([]);
const selectedCollection = ref<SavedChatItem | null>(null);

// Settings state
const settings = ref({
  theme: 'Light',
  notifications: true,
  sound: true,
});

/** WebChat desktop layout when embedded in main browser content area. */
const isWebChatDesktop = computed(() => props.fullWidth);

const primaryNavItemsList = computed(() => primaryNavItems(isWebChatDesktop.value));

const contactDirectoryCategories = CONTACT_DIRECTORY_CATEGORIES;
const contactDirectoryExpanded = ref<Set<ContactDirectoryCategoryId>>(loadContactDirectoryExpanded());
const showContactManageMenu = ref(false);
const contactImportInputRef = ref<HTMLInputElement | null>(null);

const realGroupChats = computed(() => groups.value.filter((g) => isRealGroupChat(g.groupId)));

const contactDirectoryCounts = computed(() => ({
  groupChats: realGroupChats.value.length,
  officialAccounts: subscribedPublicAccountList.value.filter(
    (a) => !a.category.toLowerCase().includes('service'),
  ).length,
  serviceAccounts: subscribedPublicAccountList.value.filter(
    (a) => a.category.toLowerCase().includes('service'),
  ).length,
  wecomContacts: contacts.value.filter(
    (c) => !c.is_blocked && (c.tags.includes('wecom') || c.groups.includes('wecom')),
  ).length,
  myEnterprises: contacts.value.filter(
    (c) => !c.is_blocked && (c.tags.includes('enterprise') || c.groups.includes('enterprise')),
  ).length,
  contacts: contacts.value.filter((c) => !c.is_blocked).length,
}));

const subscribedPublicAccountList = computed(() =>
  publicAccounts.value.filter((account) => subscribedAccounts.value.includes(account.account_id)),
);

function isContactCategoryExpanded(id: ContactDirectoryCategoryId): boolean {
  return contactDirectoryExpanded.value.has(id);
}

function handlePrimaryNavClick(id: ImNavId): void {
  if (id === 'collections') {
    void openCollectionsNav();
    return;
  }
  activeNav.value = id;
}

function handleContactCategoryClick(id: ContactDirectoryCategoryId): void {
  if (id === 'new_friends') {
    showAddContactDialog.value = true;
    return;
  }
  const category = contactDirectoryCategories.find((item) => item.id === id);
  if (category?.expandable) {
    contactDirectoryExpanded.value = toggleContactDirectoryExpanded(contactDirectoryExpanded.value, id);
    saveContactDirectoryExpanded(contactDirectoryExpanded.value);
  }
}

function getMessageReplyPreview(message: GroupMessage): string {
  const source = findReplySourceMessage(messages.value, message.replyTo);
  const label = source?.senderName ?? '消息';
  return `${label}: ${replyQuotePreview(source)}`;
}

function hasMentionTokens(content: string): boolean {
  return /@\[[^\]]+\]\(node:[^)]+\)/.test(content);
}

async function handleMentionAction(
  target: MentionTarget,
  action: 'chat' | 'voice' | 'video',
): Promise<void> {
  if (action === 'chat') {
    const contact = contacts.value.find((item) => item.node_id === target.nodeId);
    if (contact) {
      await selectContact(contact);
      return;
    }
    openImChat({
      contactId: target.contactId ?? target.nodeId,
      name: target.displayName,
      nodeId: target.nodeId,
    });
    return;
  }
  startCallFromUi({
    nodeId: target.nodeId,
    name: target.displayName,
    video: action === 'video',
    audio: true,
  });
}

async function exportContactsJson(): Promise<void> {
  try {
    const json = await contactExportJson();
    downloadContactExport(json);
    onStatus('通讯录已导出');
  } catch (e) {
    const error = normalizeError(e);
    onStatus(`导出失败: ${error.message}`);
  }
}

function triggerContactImport(): void {
  contactImportInputRef.value?.click();
}

async function handleContactImportFile(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = '';
  if (!file) return;
  try {
    const text = await file.text();
    const merge = window.confirm('合并导入？\n\n确定 = 按 node id 合并\n取消 = 替换全部联系人');
    const count = await contactImportJson(text.trim(), merge);
    await refreshContacts();
    onStatus(`已导入 ${count} 位联系人`);
  } catch (e) {
    const error = normalizeError(e);
    onStatus(`导入失败: ${error.message}`);
  }
}

const isDarkTheme = computed(() => isWebChatDesktop.value || settings.value.theme === 'Dark');

const showListSearch = computed(() =>
  ['chats', 'favorites', 'contacts', 'collections'].includes(activeNav.value),
);

const searchPlaceholder = computed(() => (isWebChatDesktop.value ? '搜索' : 'Search'));

const messageInputPlaceholder = computed(() => {
  if (editingMessage.value) return '编辑消息...';
  if (replyingTo.value) return '回复消息...';
  if (selectedFiles.value.length > 0) return isWebChatDesktop.value ? '添加消息...' : 'Add a message...';
  return isWebChatDesktop.value ? '输入消息...' : 'Type a message...';
});

type ChatListRow =
  | { kind: 'contact'; contact: Contact; lastTime: number }
  | { kind: 'group'; group: GroupChat; lastTime: number };

const webchatChatListRows = computed((): ChatListRow[] => {
  if (!isWebChatDesktop.value) return [];
  const q = search.value.trim().toLowerCase();
  const rows: ChatListRow[] = [];

  for (const c of contacts.value.filter((x) => !x.is_blocked)) {
    if (q && !c.name.toLowerCase().includes(q) && !c.node_id.toLowerCase().includes(q)) continue;
    const cache = getMessageCacheEntry(c.node_id);
    rows.push({
      kind: 'contact',
      contact: c,
      lastTime: cache?.lastTime ?? c.last_contacted ?? 0,
    });
  }

  for (const g of groups.value) {
    if (!isRealGroupChat(g.groupId)) continue;
    if (q && !g.name.toLowerCase().includes(q) && !(g.description || '').toLowerCase().includes(q)) continue;
    rows.push({
      kind: 'group',
      group: g,
      lastTime: g.lastActivity ?? g.createdAt ?? 0,
    });
  }

  return rows.sort((a, b) => b.lastTime - a.lastTime);
});

const messageTimelineItems = computed((): MessageTimelineItem[] => {
  if (!isWebChatDesktop.value) {
    return messages.value.map((message) => ({
      kind: 'message' as const,
      key: message.messageId,
      message,
    }));
  }
  return buildMessageTimelineItems(messages.value);
});

// Message cache for preview, timestamps, and unread counts (shared via imStore)
const onlineMap = ref<Map<string, PresenceEntry>>(new Map());

const IM_SETTINGS_KEY = 'exodus-im-settings';

// Computed properties
const navTitle = computed(() => {
  if (isWebChatDesktop.value) {
    const zhTitles = {
      chats: 'WebChat',
      collections: '收藏',
      favorites: '星标',
      contacts: '通讯录',
      groups: '群聊',
      public_accounts: '公众号',
      timeline: '朋友圈',
      settings: '设置',
    };
    return zhTitles[activeNav.value];
  }
  const titles = {
    chats: 'Chats',
    collections: 'Collections',
    favorites: 'Starred',
    contacts: 'Contacts',
    groups: 'Groups',
    public_accounts: 'Public Accounts',
    timeline: 'Timeline',
    settings: 'Settings',
  };
  return titles[activeNav.value];
});

const totalUnread = computed(() => {
  let total = 0;
  for (const cache of Object.values(imStore.messageCache)) {
    total += cache.unread;
  }
  return total;
});

const userAvatar = computed(() => getAvatarUrl(localNode.value));

const roomId = computed(() =>
  active.value && localNode.value ? dmRoomId(localNode.value, active.value.node_id) : '',
);

const favoriteCount = computed(() => contacts.value.filter((c) => c.is_favorite).length);

const collectionCount = computed(() => savedItems.value.length);

const filteredCollections = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return savedItems.value;
  return savedItems.value.filter(
    (item) =>
      item.content.toLowerCase().includes(q) ||
      item.sender_name.toLowerCase().includes(q) ||
      item.conversation_title.toLowerCase().includes(q),
  );
});

const messageDraft = computed({
  get: () => (editingMessage.value ? editDraft.value : draft.value),
  set: (value: string) => {
    if (editingMessage.value) {
      editDraft.value = value;
    } else {
      draft.value = value;
    }
  },
});

const filteredContacts = computed(() => {
  let list = contacts.value.filter((c) => !c.is_blocked);
  if (activeNav.value === 'favorites') {
    list = list.filter((c) => c.is_favorite);
  }
  const q = search.value.trim().toLowerCase();
  if (q) {
    list = list.filter(
      (c) =>
        c.name.toLowerCase().includes(q) ||
        c.node_id.toLowerCase().includes(q) ||
        c.notes.toLowerCase().includes(q),
    );
  }
  return [...list].sort((a, b) => {
    if (activeNav.value !== 'favorites' && a.is_favorite !== b.is_favorite) {
      return a.is_favorite ? -1 : 1;
    }
    return b.last_contacted - a.last_contacted;
  });
});

function onStatus(msg: string): void {
  emit('status', msg);
}

function activeConversationId(): string | null {
  if (activeGroup.value) return activeGroup.value.groupId;
  if (active.value && roomId.value) return roomId.value;
  return null;
}

function updateMessageCache(nodeId: string, msgs: GroupMessage[], markRead: boolean): void {
  const cache = getMessageCacheEntry(nodeId);
  setMessageCacheEntry(
    nodeId,
    applyMessageCacheUpdate(cache, msgs, localUserId.value, markRead),
  );
}

async function refreshPresence(): Promise<void> {
  if (!localNode.value) return;
  try {
    onlineMap.value = await fetchOnlinePeers(localNode.value);
  } catch {
    /* presence optional */
  }
}

async function syncContactMessagePreviews(): Promise<void> {
  if (!localNode.value) return;
  for (const c of contacts.value) {
    if (c.is_blocked) continue;
    const isActiveDm = active.value?.node_id === c.node_id && !activeGroup.value;
    try {
      const rid = dmRoomId(localNode.value, c.node_id);
      const msgs = await loadDmMessages(rid);
      updateMessageCache(c.node_id, msgs, isActiveDm);
    } catch {
      /* room may not exist yet */
    }
  }
}


async function pollMessages(): Promise<void> {
  const conversationId = activeConversationId();
  if (conversationId && !loading.value) {
    try {
      const fresh = activeGroup.value
        ? await groupGetMessages(conversationId)
        : await loadDmMessages(conversationId);
      const prevCount = messages.value.length;
      messages.value = fresh;
      if (active.value && !activeGroup.value) {
        updateMessageCache(active.value.node_id, fresh, true);
      }
      if (fresh.length > prevCount) {
        await scrollToBottom();
      }
    } catch {
      /* ignore poll errors */
    }
  }
  await syncContactMessagePreviews();
}

function startMessagePolling(): void {
  ensureImMessageSync({
    pollActiveConversation: pollMessages,
    syncContactPreviews: syncContactMessagePreviews,
    refreshPresence,
  });
}


function loadImSettings(): void {
  try {
    const raw = localStorage.getItem(IM_SETTINGS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<typeof settings.value>;
      settings.value = { ...settings.value, ...parsed };
    }
    if (props.fullWidth && settings.value.theme === 'Light') {
      settings.value.theme = 'Dark';
    }
  } catch {
    /* ignore corrupt settings */
  }
  loadMutedChats();
}

function persistImSettings(): void {
  try {
    localStorage.setItem(IM_SETTINGS_KEY, JSON.stringify(settings.value));
  } catch {
    /* storage may be unavailable */
  }
}

async function bootstrap(): Promise<void> {
  loadImSettings();
  await contactDirectoryServiceStart();
  await groupChatServiceStart();
  await publicAccountServiceStart();
  const id = await resolveLocalIdentity();
  localUserId.value = id.userId;
  localName.value = id.displayName;
  localNode.value = id.nodeId;
  await startPresenceHeartbeat(localNode.value, localName.value);
  await refreshPresence();
  await refreshContacts();
  await refreshGroups();
  await refreshCollections();
  await loadPublicAccounts();
  await syncContactMessagePreviews();
  try {
    myDigit.value = await contactGetLocalDigit();
  } catch (e) {
    const error = normalizeError(e);
    logWarn('ImMessenger', 'Failed to load local digit ID', error);
    myDigit.value = '';
  }
  startMessagePolling();
}

async function refreshCollections(): Promise<void> {
  try {
    savedItems.value = await chatCollectionList(localUserId.value);
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to refresh collections', error);
    onStatus(`Failed to load collections: ${error.message}`);
    savedItems.value = [];
  }
}

async function openCollectionsNav(): Promise<void> {
  activeNav.value = 'collections';
  await refreshCollections();
}

function collectionTypeLabel(contentType: string): string {
  switch (contentType) {
    case 'link':
      return 'Link';
    case 'image':
      return 'Image';
    case 'file':
      return 'File';
    case 'mixed':
      return 'Mixed';
    default:
      return 'Text';
  }
}

async function saveMessageToCollection(): Promise<void> {
  const message = contextMenu.value.message;
  if (!message) return;
  const conversationType = activeGroup.value ? 'group' : 'dm';
  const conversationTitle = activeGroup.value?.name ?? active.value?.name ?? 'Chat';
  try {
    const payload = buildSaveChatItemRequest({
      userId: localUserId.value,
      message,
      conversationType,
      conversationTitle,
    });
    const saved = await chatCollectionSave(payload);
    savedItems.value = [saved, ...savedItems.value.filter((i) => i.id !== saved.id)];
    onStatus('已收藏');
    logInfo('ImMessenger', 'Message saved to collections', { id: saved.id });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to save message to collections', error);
    onStatus(error.message.includes('already saved') ? '已在收藏中' : `收藏失败: ${error.message}`);
  }
  hideContextMenu();
}

async function deleteCollectionItem(item: SavedChatItem): Promise<void> {
  try {
    const deleted = await chatCollectionDelete(item.id, localUserId.value);
    if (deleted) {
      savedItems.value = savedItems.value.filter((i) => i.id !== item.id);
      if (selectedCollection.value?.id === item.id) {
        selectedCollection.value = null;
      }
      onStatus('Removed from Collections');
    }
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to delete collection item', error);
    onStatus(`Failed to remove: ${error.message}`);
  }
}

async function refreshContacts(): Promise<void> {
  try {
    contacts.value = await contactList();
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to refresh contacts', error);
    onStatus(`Failed to load contacts: ${error.message}`);
    contacts.value = [];
  }
}

async function toggleContactFavorite(contact: Contact): Promise<void> {
  try {
    const isFavorite = await contactToggleFavorite(contact.contact_id);
    const index = contacts.value.findIndex((c) => c.contact_id === contact.contact_id);
    if (index >= 0) {
      contacts.value[index] = { ...contacts.value[index], is_favorite: isFavorite };
    }
    if (active.value?.contact_id === contact.contact_id) {
      /* active is derived from contacts list */
    }
    onStatus(isFavorite ? `${contact.name} added to Starred` : `${contact.name} removed from Starred`);
    logInfo('ImMessenger', 'Contact favorite toggled', { contactId: contact.contact_id, isFavorite });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to toggle contact favorite', error);
    onStatus(`Failed to update favorite: ${error.message}`);
  }
}

async function refreshGroups(): Promise<void> {
  try {
    groups.value = await groupListUser(localUserId.value);
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to refresh groups', error);
    onStatus(`Failed to load groups: ${error.message}`);
    groups.value = [];
  }
}

async function selectContact(c: Contact, fromStore = false): Promise<void> {
  if (!fromStore) setActiveContactNode(c.node_id);
  if (!localNode.value) {
    logWarn('ImMessenger', 'Cannot select contact: local node not initialized');
    return;
  }
  const rid = dmRoomId(localNode.value, c.node_id);
  loading.value = true;
  try {
    await ensureDmGroup(rid, localUserId.value, localName.value, c.node_id, c.name);
    messages.value = await loadDmMessages(rid);
    updateMessageCache(c.node_id, messages.value, true);
    await scrollToBottom();
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to select contact', error);
    onStatus(`Failed to load messages: ${error.message}`);
  } finally {
    loading.value = false;
  }
}

async function prepareSelectedAttachments(conversationId: string): Promise<GroupMessageAttachment[]> {
  if (selectedFiles.value.length === 0) return [];
  onStatus('Uploading attachments…');
  return Promise.all(
    selectedFiles.value.map((entry) => prepareBrowserFileAttachment(conversationId, entry.file)),
  );
}

async function handleSubmit(): Promise<void> {
  if (editingMessage.value) {
    await saveEditMessage();
  } else {
    await sendMessage();
  }
}

async function sendMessage(): Promise<void> {
  const text = draft.value.trim();
  const hasFiles = selectedFiles.value.length > 0;
  
  if (!text && !hasFiles) {
    logWarn('ImMessenger', 'Cannot send message: empty text and no files');
    return;
  }
  
  const replyToId = replyingTo.value?.messageId || null;
  
  // Send to group or DM
  if (activeGroup.value) {
    // Group message
    const mentionNodeIds = extractMentionNodeIds(text, groupMembers.value.map(m => ({ contact_id: m.agentId, name: m.agentName, contact_type: 'group', agent_ids: [m.agentId], node_id: m.agentId, groups: [], tags: [], notes: '', is_favorite: false, is_blocked: false, created_at: m.joinedAt, last_contacted: m.lastSeen, contact_count: 0 })));
    try {
      const attachments = hasFiles
        ? await prepareSelectedAttachments(activeGroup.value.groupId)
        : [];
      
      const msg: GroupChatMessage = {
        messageId: `msg-${Date.now()}`,
        groupId: activeGroup.value.groupId,
        senderId: localUserId.value,
        senderName: localName.value,
        content: text,
        messageType: hasFiles ? 'file' : 'text',
        attachments,
        mentions: mentionNodeIds,
        replyTo: replyToId,
        timestamp: Date.now(),
        isEdited: false,
      };
      await sendGroupMessageWithCdn(msg);
      notifyImNewMessage(activeGroup.value.groupId);
      draft.value = '';
      selectedFiles.value = [];
      replyingTo.value = null;
      messages.value = await groupGetMessages(activeGroup.value.groupId);
      await scrollToBottom();
      onStatus('Message sent');
    } catch (e) {
      const error = normalizeError(e);
      logError('ImMessenger', 'Failed to send group message', error);
      onStatus(`Failed to send message: ${error.message}`);
    }
  } else if (active.value && roomId.value) {
    // DM message (same CDN room as group chat)
    const mentionNodeIds = extractMentionNodeIds(text, contacts.value);
    try {
      const attachments = hasFiles ? await prepareSelectedAttachments(roomId.value) : [];
      await sendDmText(
        roomId.value,
        localUserId.value,
        localName.value,
        text,
        mentionNodeIds,
        replyToId,
        attachments,
      );
      draft.value = '';
      selectedFiles.value = [];
      replyingTo.value = null;
      messages.value = await loadDmMessages(roomId.value);
      if (active.value) {
        updateMessageCache(active.value.node_id, messages.value, true);
      }
      void touchContactLastContacted(active.value.node_id);
      await scrollToBottom();
    } catch (e) {
      const error = normalizeError(e);
      logError('ImMessenger', 'Failed to send DM message', error);
      onStatus(`Failed to send message: ${error.message}`);
    }
  } else {
    logWarn('ImMessenger', 'Cannot send message: no active chat');
  }
}

async function handleEnterKey(event: KeyboardEvent): Promise<void> {
  if (event.shiftKey) {
    // Allow new line with Shift+Enter
    return;
  }
  // Send with Enter
  await sendMessage();
}

async function scrollToBottom(): Promise<void> {
  await nextTick();
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
  }
}

function voiceCall(): void {
  if (!active.value) return;
  startCallFromUi({
    nodeId: active.value.node_id,
    name: active.value.name,
    video: false,
    audio: true,
  });
}

function videoCall(): void {
  if (!active.value) return;
  startCallFromUi({
    nodeId: active.value.node_id,
    name: active.value.name,
    video: true,
    audio: true,
  });
}

function toggleTheme(): void {
  settings.value.theme = settings.value.theme === 'Light' ? 'Dark' : 'Light';
  persistImSettings();
  logInfo('ImMessenger', 'Theme toggled', { theme: settings.value.theme });
  onStatus(`Theme changed to ${settings.value.theme}`);
}

function toggleNotifications(): void {
  settings.value.notifications = !settings.value.notifications;
  persistImSettings();
  logInfo('ImMessenger', 'Notifications toggled', { enabled: settings.value.notifications });
  onStatus(`Notifications ${settings.value.notifications ? 'enabled' : 'disabled'}`);
}

function toggleSound(): void {
  settings.value.sound = !settings.value.sound;
  persistImSettings();
  logInfo('ImMessenger', 'Sound toggled', { enabled: settings.value.sound });
  onStatus(`Sound ${settings.value.sound ? 'enabled' : 'disabled'}`);
}

// Group chat functions
async function createGroup(): Promise<void> {
  if (!newGroupName.value.trim()) {
    logWarn('ImMessenger', 'Cannot create group: missing name');
    return;
  }
  if (selectedGroupMembers.value.length === 0) {
    logWarn('ImMessenger', 'Cannot create group: no members selected');
    return;
  }
  
  const groupId = `group-${Date.now()}`;
  const memberIds = [localUserId.value, ...selectedGroupMembers.value];
  
  try {
    await groupCreate(
      buildGroupPayload({
        groupId,
        name: newGroupName.value.trim(),
        description: newGroupDescription.value.trim() || '',
        ownerId: localUserId.value,
        memberIds,
      })
    );
    showCreateGroup.value = false;
    newGroupName.value = '';
    newGroupDescription.value = '';
    selectedGroupMembers.value = [];
    await refreshGroups();
    onStatus('Group created successfully');
    logInfo('ImMessenger', 'Group created', { groupId, name: newGroupName.value });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to create group', error);
    onStatus(`Failed to create group: ${error.message}`);
  }
}

async function selectGroup(group: GroupChat, fromStore = false): Promise<void> {
  if (!fromStore) setActiveGroupId(group.groupId);
  loading.value = true;
  try {
    messages.value = await groupGetMessages(group.groupId);
    groupMembers.value = await groupGetMembers(group.groupId);
    await scrollToBottom();
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to select group', error);
    onStatus(`Failed to load group messages: ${error.message}`);
  } finally {
    loading.value = false;
  }
}

function toggleGroupMember(nodeId: string): void {
  const index = selectedGroupMembers.value.indexOf(nodeId);
  if (index > -1) {
    selectedGroupMembers.value.splice(index, 1);
  } else {
    selectedGroupMembers.value.push(nodeId);
  }
}

function loadMutedChats(): void {
  mutedChatIds.value = loadMutedChatIds();
}

function isConversationMuted(conversationId: string): boolean {
  return mutedChatIds.value.has(conversationId);
}

function toggleConversationMute(conversationId: string): void {
  const next = new Set(mutedChatIds.value);
  if (next.has(conversationId)) {
    next.delete(conversationId);
    onStatus('已关闭消息免打扰');
  } else {
    next.add(conversationId);
    onStatus('已开启消息免打扰');
  }
  mutedChatIds.value = next;
  saveMutedChatIds(next);
}

function activeConversationMuteId(): string | null {
  if (activeGroup.value) return conversationIdForGroup(activeGroup.value.groupId);
  if (active.value) return conversationIdForContact(active.value.node_id);
  return null;
}

function isActiveConversationMuted(): boolean {
  const id = activeConversationMuteId();
  return id ? isConversationMuted(id) : false;
}

function toggleActiveConversationMute(): void {
  const id = activeConversationMuteId();
  if (!id) return;
  toggleConversationMute(id);
  showConversationMenu.value = false;
}

function getGroupGridAvatarUrls(group: GroupChat): string[] {
  return pickGroupGridMemberIds(group.memberIds).map((memberId) => getAvatarUrl(memberId));
}

function groupGridClass(group: GroupChat): string {
  const count = Math.max(1, Math.min(pickGroupGridMemberIds(group.memberIds).length, 9));
  return groupGridCountClass(count);
}

function getGroupAvatarUrl(group: GroupChat): string {
  if (group.avatarUrl) return group.avatarUrl;
  // Generate group avatar from group name
  const hash = group.name.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const colors = ['#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F'];
  const color = colors[hash % colors.length];
  const initial = group.name.charAt(0).toUpperCase();
  return `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
      <rect width="32" height="32" fill="${color}"/>
      <text x="16" y="20" font-size="16" font-weight="bold" fill="white" text-anchor="middle">${initial}</text>
    </svg>`
  )}`;
}

async function leaveGroup(): Promise<void> {
  if (!activeGroup.value) return;
  
  try {
    const leftGroupId = activeGroup.value.groupId;
    await groupRemoveMember(leftGroupId, localUserId.value);
    groups.value = groups.value.filter((g) => g.groupId !== leftGroupId);
    setActiveGroupId(null);
    groupMembers.value = [];
    showGroupSettings.value = false;
    onStatus('Left group');
    logInfo('ImMessenger', 'Left group', { groupId: leftGroupId });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to leave group', error);
    onStatus(`Failed to leave group: ${error.message}`);
  }
}

async function onOpenContact(ev: Event): Promise<void> {
  const detail = (ev as CustomEvent<ImOpenContactDetail>).detail;
  if (!detail?.nodeId) return;
  let c = contacts.value.find((x) => x.node_id === detail.nodeId);
  if (!c) {
    await refreshContacts();
    c = contacts.value.find((x) => x.node_id === detail.nodeId);
  }
  if (c) {
    activeNav.value = 'chats';
    await selectContact(c);
  } else {
    onStatus(`Contact not found: ${detail.name || detail.nodeId}`);
  }
}

// Helper functions for UI
function getAvatarUrl(nodeId: string): string {
  // Use DiceBear API for consistent, professional avatars
  return `https://api.dicebear.com/7.x/avataaars/svg?seed=${nodeId}`;
}

function isOnline(nodeId: string): boolean {
  return isNodeOnline(onlineMap.value, nodeId);
}

function getLastMessageTime(nodeId: string): string {
  const cache = getMessageCacheEntry(nodeId);
  if (cache.lastTime === 0) return '';
  return formatMessageTime(cache.lastTime);
}

function getLastMessagePreview(nodeId: string): string {
  const cache = getMessageCacheEntry(nodeId);
  return cache.lastMessage || 'No messages';
}

function getUnreadCount(nodeId: string): number {
  return getMessageCacheEntry(nodeId).unread;
}

function isOwnMessage(message: GroupMessage): boolean {
  return message.senderId === localUserId.value;
}

function autoResizeTextarea(): void {
  const textarea = messageInput.value;
  if (!textarea) return;
  
  textarea.style.height = 'auto';
  const newHeight = Math.min(textarea.scrollHeight, 120);
  textarea.style.height = `${newHeight}px`;
}

// Contact management functions
async function addContact(): Promise<void> {
  if (!addContactName.value.trim() || !addContactNode.value.trim()) {
    logWarn('ImMessenger', 'Cannot add contact: missing name or node ID');
    return;
  }

  const name = addContactName.value.trim();
  try {
    await contactAdd(buildHumanContact({
      name,
      nodeId: addContactNode.value.trim(),
      notes: addContactNotes.value.trim(),
    }));
    showAddContactDialog.value = false;
    addContactName.value = '';
    addContactDigit.value = '';
    addContactNode.value = '';
    addContactNotes.value = '';
    await refreshContacts();
    onStatus('Contact added successfully');
    logInfo('ImMessenger', 'Contact added', { name });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to add contact', error);
    onStatus(`Failed to add contact: ${error.message}`);
  }
}

async function addContactByDigit(): Promise<void> {
  const digit = addContactDigit.value.replace(/\D/g, '');
  if (digit.length !== 12) {
    onStatus('Enter 12-digit Exodus ID');
    return;
  }

  const name = addContactName.value.trim() || `Friend ${digit}`;
  try {
    const contact = await contactAddFriendByDigit(digit, name, localUserId.value);
    showAddContactDialog.value = false;
    addContactName.value = '';
    addContactDigit.value = '';
    addContactNode.value = '';
    addContactNotes.value = '';
    await refreshContacts();
    await selectContact(contact);
    onStatus('Friend added');
    logInfo('ImMessenger', 'Friend added by digit', { digit, name: contact.name });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to add friend by digit', error);
    onStatus(`Failed to add friend: ${error.message}`);
  }
}

async function copyMyDigit(): Promise<void> {
  if (!myDigit.value) return;
  try {
    await navigator.clipboard.writeText(myDigit.value);
    onStatus('12-digit ID copied');
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to copy digit ID', error);
    onStatus('Copy failed');
  }
}

function openEditContactDialog(contact: Contact): void {
  editingContact.value = contact;
  editContactName.value = contact.name;
  editContactNotes.value = contact.notes || '';
  editContactBlocked.value = contact.is_blocked || false;
  showEditContactDialog.value = true;
}

async function saveContactEdit(): Promise<void> {
  if (!editingContact.value) return;
  const contactId = editingContact.value.contact_id;

  try {
    await contactUpdate({
      ...editingContact.value,
      name: editContactName.value.trim(),
      notes: editContactNotes.value.trim(),
      is_blocked: editContactBlocked.value,
    });
    showEditContactDialog.value = false;
    editingContact.value = null;
    await refreshContacts();
    onStatus('Contact updated successfully');
    logInfo('ImMessenger', 'Contact updated', { contactId });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to update contact', error);
    onStatus(`Failed to update contact: ${error.message}`);
  }
}

async function deleteContact(contact: Contact): Promise<void> {
  if (!confirm(`Are you sure you want to delete ${contact.name}?`)) return;
  
  try {
    await contactRemove(contact.contact_id);
    await refreshContacts();
    if (active.value?.contact_id === contact.contact_id) {
      setActiveContactNode(null);
      const id = activeConversationId();
      if (id) setStoreMessages(id, []);
    }
    onStatus('Contact deleted');
    logInfo('ImMessenger', 'Contact deleted', { contactId: contact.contact_id });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to delete contact', error);
    onStatus(`Failed to delete contact: ${error.message}`);
  }
}

function startVoiceCall(contact: Contact): void {
  startCallFromUi({ nodeId: contact.node_id, name: contact.name, video: false, audio: true });
  onStatus(`Calling ${contact.name}...`);
  logInfo('ImMessenger', 'Voice call started', { nodeId: contact.node_id });
}

// Public account functions
async function loadPublicAccounts(): Promise<void> {
  try {
    await publicAccountServiceStart();
    publicAccounts.value = await publicAccountList();
    subscribedAccounts.value = await publicAccountGetSubscriptions(localUserId.value);
    logInfo('ImMessenger', 'Public accounts loaded', { count: publicAccounts.value.length });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to load public accounts', error);
    onStatus(`Failed to load public accounts: ${error.message}`);
    publicAccounts.value = [];
  }
}

async function searchPublicAccounts(): Promise<void> {
  if (!publicAccountSearchQuery.value.trim()) return;
  
  try {
    const results = await publicAccountSearch(publicAccountSearchQuery.value.trim(), 20);
    publicAccounts.value = results;
    showPublicAccountSearch.value = false;
    publicAccountSearchQuery.value = '';
    onStatus(`Found ${results.length} public accounts`);
    logInfo('ImMessenger', 'Public accounts searched', { query: publicAccountSearchQuery.value, count: results.length });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to search public accounts', error);
    onStatus(`Failed to search: ${error.message}`);
  }
}

async function toggleSubscribe(account: PublicAccount): Promise<void> {
  const isSubscribed = subscribedAccounts.value.includes(account.account_id);
  
  try {
    if (isSubscribed) {
      await publicAccountUnsubscribe(localUserId.value, account.account_id);
      subscribedAccounts.value = subscribedAccounts.value.filter(id => id !== account.account_id);
      onStatus(`Unsubscribed from ${account.name}`);
      logInfo('ImMessenger', 'Unsubscribed from public account', { accountId: account.account_id });
    } else {
      await publicAccountSubscribe(localUserId.value, account.account_id);
      subscribedAccounts.value.push(account.account_id);
      onStatus(`Subscribed to ${account.name}`);
      logInfo('ImMessenger', 'Subscribed to public account', { accountId: account.account_id });
    }
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to toggle subscription', error);
    onStatus(`Failed to ${isSubscribed ? 'unsubscribe' : 'subscribe'}: ${error.message}`);
  }
}

async function selectPublicAccount(account: PublicAccount): Promise<void> {
  activePublicAccount.value = account;
  
  try {
    publicAccountArticles.value = await publicAccountListArticles(account.account_id);
    onStatus(`Loaded articles from ${account.name}`);
    logInfo('ImMessenger', 'Public account selected', { accountId: account.account_id, articleCount: publicAccountArticles.value.length });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to load articles', error);
    onStatus(`Failed to load articles: ${error.message}`);
    publicAccountArticles.value = [];
  }
}

// Context menu functions
let touchTimer: number | null = null;

function showContextMenu(event: MouseEvent, message: GroupMessage): void {
  event.preventDefault();
  void (async () => {
    let saved = false;
    try {
      saved = await chatCollectionIsSaved(localUserId.value, message.messageId);
    } catch {
      saved = false;
    }
    contextMenu.value = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      message,
      saved,
    };
  })();
}

function handleTouchStart(event: TouchEvent, message: GroupMessage): void {
  touchTimer = window.setTimeout(() => {
    if (event.touches.length === 0) return;
    const touch = event.touches[0];
    void (async () => {
      let saved = false;
      try {
        saved = await chatCollectionIsSaved(localUserId.value, message.messageId);
      } catch {
        saved = false;
      }
      contextMenu.value = {
        visible: true,
        x: touch.clientX,
        y: touch.clientY,
        message,
        saved,
      };
    })();
  }, 500);
}

function handleTouchEnd(): void {
  if (touchTimer) {
    clearTimeout(touchTimer);
    touchTimer = null;
  }
}

function hideContextMenu(): void {
  contextMenu.value = {
    visible: false,
    x: 0,
    y: 0,
    message: null,
    saved: false,
  };
}

function copyMessage(): void {
  if (contextMenu.value.message) {
    navigator.clipboard.writeText(contextMenu.value.message.content);
    logInfo('ImMessenger', 'Message copied to clipboard');
    onStatus('Message copied');
  }
  hideContextMenu();
}

async function deleteMessage(): Promise<void> {
  if (!contextMenu.value.message) return;
  const messageId = contextMenu.value.message.messageId;
  const conversationId = activeConversationId();

  try {
    if (conversationId) {
      await groupDeleteMessage(conversationId, messageId);
    }
    messages.value = messages.value.filter((m) => m.messageId !== messageId);
    logInfo('ImMessenger', 'Message deleted', { messageId });
    onStatus('Message deleted');
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to delete message', error);
    onStatus(`Failed to delete message: ${error.message}`);
  }
  hideContextMenu();
}

function canRecallMessage(message: GroupMessage | null): boolean {
  if (!message) return false;
  // Allow recall within 2 minutes of sending
  const recallTimeLimit = 2 * 60 * 1000; // 2 minutes
  const timeSinceSent = Date.now() - message.timestamp;
  return timeSinceSent < recallTimeLimit;
}

async function recallMessage(): Promise<void> {
  if (!contextMenu.value.message) return;
  const messageId = contextMenu.value.message.messageId;
  const conversationId = activeConversationId();

  try {
    if (conversationId) {
      await groupDeleteMessage(conversationId, messageId);
    }

    const msgIndex = messages.value.findIndex((m) => m.messageId === messageId);
    if (msgIndex === -1) {
      logWarn('ImMessenger', 'Message not found for recall', { messageId });
      return;
    }
    messages.value[msgIndex] = {
      ...messages.value[msgIndex],
      content: '消息已撤回',
      isEdited: true,
      editedAt: Date.now(),
    };

    hideContextMenu();
    onStatus('Message recalled');
    logInfo('ImMessenger', 'Message recalled', { messageId });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to recall message', error);
    onStatus(`Failed to recall message: ${error.message}`);
  }
}

function startEditMessage(): void {
  if (contextMenu.value.message) {
    editingMessage.value = contextMenu.value.message;
    editDraft.value = contextMenu.value.message.content;
    hideContextMenu();
    // Focus on input after a short delay
    nextTick(() => {
      if (messageInput.value) {
        messageInput.value.focus();
        autoResizeTextarea();
      }
    });
  }
}

function cancelEditMessage(): void {
  editingMessage.value = null;
  editDraft.value = '';
}

function startReplyMessage(): void {
  if (contextMenu.value.message) {
    replyingTo.value = contextMenu.value.message;
    hideContextMenu();
    // Focus on input after a short delay
    nextTick(() => {
      if (messageInput.value) {
        messageInput.value.focus();
        autoResizeTextarea();
      }
    });
  }
}

function cancelReplyMessage(): void {
  replyingTo.value = null;
}

// Mention autocomplete functions
function handleInput(event: Event): void {
  const target = event.target as HTMLTextAreaElement;
  const text = editingMessage.value ? editDraft.value : draft.value;
  const cursorPos = target.selectionStart;
  
  // Check if we're typing a mention
  const textBeforeCursor = text.substring(0, cursorPos);
  const lastAtIndex = textBeforeCursor.lastIndexOf('@');
  
  if (lastAtIndex > -1 && (lastAtIndex === 0 || textBeforeCursor[lastAtIndex - 1] === ' ')) {
    // We're typing a mention
    const query = textBeforeCursor.substring(lastAtIndex + 1);
    mentionQuery.value = query;
    mentionStartIndex.value = lastAtIndex;
    
    // Get available users based on context
    const availableUsers = activeGroup.value 
      ? groupMembers.value.map(m => ({ id: m.agentId, name: m.agentName }))
      : contacts.value.map(c => ({ id: c.node_id, name: c.name }));
    
    // Filter suggestions
    mentionSuggestions.value = availableUsers
      .filter(u => u.name.toLowerCase().includes(query.toLowerCase()))
      .slice(0, 5);
    
    showMentionAutocomplete.value = mentionSuggestions.value.length > 0;
    mentionIndex.value = 0;
  } else {
    hideMentionAutocomplete();
  }
  
  autoResizeTextarea();
}

function handleKeyDown(event: KeyboardEvent): void {
  if (!showMentionAutocomplete.value) return;
  
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    mentionIndex.value = Math.min(mentionIndex.value + 1, mentionSuggestions.value.length - 1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    mentionIndex.value = Math.max(mentionIndex.value - 1, 0);
  } else if (event.key === 'Escape') {
    event.preventDefault();
    hideMentionAutocomplete();
  } else if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    if (mentionSuggestions.value[mentionIndex.value]) {
      selectMention(mentionSuggestions.value[mentionIndex.value]);
    }
  }
}

function hideMentionAutocomplete(): void {
  showMentionAutocomplete.value = false;
  mentionSuggestions.value = [];
  mentionQuery.value = '';
  mentionIndex.value = 0;
}

function selectMention(suggestion: { id: string; name: string }): void {
  const text = editingMessage.value ? editDraft.value : draft.value;
  const beforeMention = text.substring(0, mentionStartIndex.value);
  const afterCursor = text.substring(mentionStartIndex.value + mentionQuery.value.length + 1);
  
  const newText = beforeMention + '@' + suggestion.name + ' ' + afterCursor;
  
  if (editingMessage.value) {
    editDraft.value = newText;
  } else {
    draft.value = newText;
  }
  
  hideMentionAutocomplete();
  
  // Focus back on input and move cursor
  nextTick(() => {
    if (messageInput.value) {
      const newCursorPos = mentionStartIndex.value + suggestion.name.length + 2;
      messageInput.value.setSelectionRange(newCursorPos, newCursorPos);
      messageInput.value.focus();
      autoResizeTextarea();
    }
  });
}

function insertEmoji(emoji: string): void {
  const text = editingMessage.value ? editDraft.value : draft.value;
  const newText = text + emoji;
  
  if (editingMessage.value) {
    editDraft.value = newText;
  } else {
    draft.value = newText;
  }
  
  showEmojiPicker.value = false;
  
  // Focus back on input
  nextTick(() => {
    if (messageInput.value) {
      messageInput.value.focus();
      autoResizeTextarea();
    }
  });
}

// File attachment functions
function triggerFileUpload(): void {
  if (fileInputRef.value) {
    fileInputRef.value.click();
  }
}

function handleFileSelect(event: Event): void {
  const target = event.target as HTMLInputElement;
  const files = target.files;
  if (!files) return;
  
  Array.from(files).forEach(file => {
    // Create preview for images
    if (file.type.startsWith('image/')) {
      const reader = new FileReader();
      reader.onload = (e) => {
        selectedFiles.value.push({
          file,
          preview: e.target?.result as string,
        });
      };
      reader.readAsDataURL(file);
    } else {
      selectedFiles.value.push({ file });
    }
  });
  
  // Reset input to allow selecting same file again
  if (target) {
    target.value = '';
  }
}

function removeFile(index: number): void {
  selectedFiles.value.splice(index, 1);
}


// Search functions
function performSearch(): void {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) {
    searchResults.value = [];
    searchIndex.value = 0;
    return;
  }
  
  searchResults.value = messages.value.filter(m => 
    m.content.toLowerCase().includes(query) ||
    m.senderName.toLowerCase().includes(query)
  );
  
  searchIndex.value = 0;
  
  if (searchResults.value.length > 0) {
    scrollToSearchResult();
  }
}

function prevSearchResult(): void {
  if (searchIndex.value > 0) {
    searchIndex.value--;
    scrollToSearchResult();
  }
}

function nextSearchResult(): void {
  if (searchIndex.value < searchResults.value.length - 1) {
    searchIndex.value++;
    scrollToSearchResult();
  }
}

function scrollToSearchResult(): void {
  const result = searchResults.value[searchIndex.value];
  if (!result || !messagesContainer.value) return;
  
  const messageElements = messagesContainer.value.querySelectorAll('.message-bubble');
  const targetElement = Array.from(messageElements).find(el => 
    el.textContent?.includes(result.content.substring(0, 20))
  );
  
  if (targetElement) {
    targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
    // Highlight the message temporarily
    targetElement.classList.add('search-highlight');
    setTimeout(() => {
      targetElement.classList.remove('search-highlight');
    }, 2000);
  }
}

async function saveEditMessage(): Promise<void> {
  if (!editingMessage.value || !editDraft.value.trim()) {
    logWarn('ImMessenger', 'Cannot save edit: missing message or empty content');
    return;
  }

  const messageId = editingMessage.value.messageId;
  const conversationId = activeConversationId();
  const newContent = editDraft.value.trim();

  try {
    if (conversationId) {
      await groupEditMessage(conversationId, messageId, newContent);
    }

    const msgIndex = messages.value.findIndex((m) => m.messageId === messageId);
    if (msgIndex === -1) {
      logWarn('ImMessenger', 'Message not found for edit', { messageId });
      return;
    }
    messages.value[msgIndex] = {
      ...messages.value[msgIndex],
      content: newContent,
      isEdited: true,
      editedAt: Date.now(),
    };

    editingMessage.value = null;
    editDraft.value = '';
    onStatus('Message edited');
    logInfo('ImMessenger', 'Message edited', { messageId });
  } catch (e) {
    const error = normalizeError(e);
    logError('ImMessenger', 'Failed to edit message', error);
    onStatus(`Failed to edit message: ${error.message}`);
  }
}

function formatMessageTime(timestamp?: number): string {
  if (!timestamp) return '';
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  
  // Less than 1 minute
  if (diff < 60000) return '刚刚';
  
  // Less than 1 hour
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
  
  // Less than 24 hours
  if (diff < 86400000) {
    const hours = Math.floor(diff / 3600000);
    if (hours === 1) return '1小时前';
    return `${hours}小时前`;
  }
  
  // Yesterday
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);
  if (date.toDateString() === yesterday.toDateString()) {
    return `昨天 ${date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })}`;
  }
  
  // This week
  const weekAgo = new Date(now);
  weekAgo.setDate(weekAgo.getDate() - 7);
  if (date > weekAgo) {
    const days = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    return `${days[date.getDay()]} ${date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })}`;
  }
  
  // This year
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' });
  }
  
  // Older
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

function sanitizeMessage(content: string): string {
  // Basic XSS prevention - escape HTML special characters
  const div = document.createElement('div');
  div.textContent = content;
  return div.innerHTML;
}

// Watch for new messages and update cache
watch(messages, (newMessages) => {
  if (active.value && !activeGroup.value && Array.isArray(newMessages) && newMessages.length > 0) {
    updateMessageCache(active.value.node_id, newMessages, true);
  }
});

watch(
  () => imStore.revision,
  () => {
    void nextTick(() => {
      if (activeConversationId()) void scrollToBottom();
    });
  },
);

watch(
  () => imStore.activeGroupId,
  async (groupId, prev) => {
    if (!groupId || groupId === prev) return;
    try {
      groupMembers.value = await groupGetMembers(groupId);
    } catch {
      /* optional */
    }
  },
);

onMounted(() => {
  void bootstrap();
  window.addEventListener(IM_OPEN_CONTACT_EVENT, onOpenContact as EventListener);
});

onUnmounted(() => {
  window.removeEventListener(IM_OPEN_CONTACT_EVENT, onOpenContact as EventListener);
  if (localNode.value) void stopPresenceHeartbeat();
  if (touchTimer) {
    clearTimeout(touchTimer);
    touchTimer = null;
  }
});
</script>

<style scoped>
/* WebChat IM Messenger - Aerospace-grade implementation with two-level sidebar */

.im-messenger {
  display: flex;
  height: 100%;
  background: #f5f5f5;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

/* Full-width variant for main content area */
.im-messenger.full-width {
  border-radius: 0;
  box-shadow: none;
  width: 100%;
  flex: 1;
  min-height: 0;
}

/* Level 1 Sidebar - Navigation (narrow) */
.nav-sidebar {
  width: 60px;
  background: #2e2e2e;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.nav-header {
  padding: 16px 8px;
  display: flex;
  justify-content: center;
  border-bottom: 1px solid #3e3e3e;
}

.user-avatar-small {
  width: 40px;
  height: 40px;
  border-radius: 4px;
  overflow: hidden;
}

.user-avatar-small img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 4px;
  flex: 1;
}

.nav-menu-footer {
  flex: 0;
  margin-top: auto;
  padding-bottom: 12px;
  border-top: 1px solid #3e3e3e;
}

.nav-item {
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  color: #999;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all 0.2s ease;
}

.nav-item:hover {
  background: #3e3e3e;
  color: #fff;
}

.nav-item.active {
  background: transparent;
  color: #07c160;
}

.nav-badge {
  position: absolute;
  top: 6px;
  right: 6px;
  min-width: 18px;
  height: 18px;
  background: #ff4d4f;
  color: #fff;
  font-size: 11px;
  font-weight: bold;
  border-radius: 9px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
}

.nav-badge-muted {
  background: #faad14;
}

/* Level 2 Sidebar - Content List (wider) */
.content-sidebar {
  width: 280px;
  background: #ffffff;
  border-right: 1px solid #e5e5e5;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-header {
  padding: 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #f0f0f0;
}

.sidebar-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
  margin: 0;
}

.user-avatar {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: 4px;
  overflow: hidden;
}

.user-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.online-indicator {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 10px;
  height: 10px;
  background: #52c41a;
  border: 2px solid #fff;
  border-radius: 50%;
}

.sidebar-actions {
  display: flex;
  gap: 8px;
}

.icon-button {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: #666;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.icon-button:hover {
  background: #f5f5f5;
  color: #1890ff;
}

.icon-button:active {
  background: #e6f7ff;
}

.search-container {
  padding: 12px 16px;
  position: relative;
}

.search-icon {
  position: absolute;
  left: 24px;
  top: 50%;
  transform: translateY(-50%);
  color: #999;
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  background: #f5f5f5;
  font-size: 13px;
  color: #333;
  outline: none;
  transition: all 0.2s ease;
}

.search-input:focus {
  background: #fff;
  border-color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.1);
}

.search-input::placeholder {
  color: #999;
}

.add-contact-form {
  padding: 12px 16px;
  background: #fafafa;
  border-top: 1px solid #e5e5e5;
  border-bottom: 1px solid #e5e5e5;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-input {
  padding: 8px 12px;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  background: #fff;
  font-size: 13px;
  color: #333;
  outline: none;
}

.form-input:focus {
  border-color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.1);
}

.primary-button {
  padding: 8px 16px;
  background: #1890ff;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.primary-button:hover {
  background: #40a9ff;
}

.primary-button:active {
  background: #096dd9;
}

.chat-list {
  flex: 1;
  overflow-y: auto;
  list-style: none;
  margin: 0;
  padding: 0;
}

.chat-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
  border-bottom: 1px solid #f5f5f5;
}

.chat-item.contact-item {
  padding: 10px 12px;
}

.chat-item.blocked {
  opacity: 0.6;
}

.chat-item:hover {
  background: #f5f5f5;
}

.chat-item.active {
  background: #e6f7ff;
}

.chat-item.favorited .chat-name {
  color: #d48806;
}

.chat-avatar {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 4px;
  overflow: hidden;
  margin-right: 12px;
  flex-shrink: 0;
}

.chat-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.online-dot {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 12px;
  height: 12px;
  background: #52c41a;
  border: 2px solid #fff;
  border-radius: 50%;
}

.chat-info {
  flex: 1;
  min-width: 0;
}

.chat-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.chat-header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.favorite-btn {
  background: none;
  border: none;
  color: #bfbfbf;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  padding: 0 2px;
  transition: color 0.2s ease, transform 0.15s ease;
}

.favorite-btn:hover {
  color: #faad14;
  transform: scale(1.1);
}

.favorite-btn.active {
  color: #faad14;
}

.collection-type-badge {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fff7e6;
  color: #d48806;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
}

.collection-meta {
  font-size: 11px;
  color: #999;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.collection-delete-btn {
  background: none;
  border: none;
  color: #bbb;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  padding: 0 4px;
  flex-shrink: 0;
}

.collection-delete-btn:hover {
  color: #ff4d4f;
}

.collection-item.active {
  background: #fffbe6;
}

.chat-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chat-time {
  font-size: 12px;
  color: #999;
  flex-shrink: 0;
  margin-left: 8px;
}

.chat-preview {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.preview-text {
  font-size: 13px;
  color: #999;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.unread-badge {
  background: #ff4d4f;
  color: #fff;
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 10px;
  min-width: 18px;
  text-align: center;
  margin-left: 8px;
  flex-shrink: 0;
}

.blocked-badge {
  background: #ff4d4f;
  color: #fff;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  margin-left: 8px;
  flex-shrink: 0;
}

.contact-actions {
  display: flex;
  gap: 4px;
  margin-left: 8px;
  flex-shrink: 0;
}

.action-btn {
  background: none;
  border: none;
  color: #666;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-btn:hover {
  background: #f0f0f0;
  color: #1890ff;
}

.action-btn.delete-btn:hover {
  background: #fff1f0;
  color: #ff4d4f;
}

.public-account-item {
  position: relative;
}

.public-account-item.subscribed .chat-name {
  color: #1890ff;
}

.verified-badge {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 14px;
  height: 14px;
  background: #52c41a;
  color: white;
  border-radius: 50%;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid #fff;
}

.subscribe-btn {
  background: none;
  border: 1px solid #d9d9d9;
  color: #666;
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.subscribe-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}

.subscribe-btn.active {
  background: #52c41a;
  border-color: #52c41a;
  color: white;
}

.follower-count {
  font-size: 11px;
  color: #999;
  margin-left: 8px;
  flex-shrink: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: #999;
}

.empty-state svg {
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 13px;
  margin: 0;
}

.empty-hint {
  margin-top: 8px !important;
  font-size: 12px !important;
  color: #bbb !important;
}

.settings-list {
  padding: 16px;
}

.settings-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  transition: background 0.2s ease;
}

.settings-item:hover {
  background: #f5f5f5;
}

.settings-item:last-child {
  border-bottom: none;
}

.settings-label {
  font-size: 14px;
  color: #333;
}

.settings-value {
  font-size: 14px;
  color: #666;
}

/* Right Main - Chat Window */
.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
  min-width: 0;
}

.chat-window-header {
  padding: 12px 16px;
  background: #fff;
  border-bottom: 1px solid #e5e5e5;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-avatar {
  position: relative;
  width: 36px;
  height: 36px;
  border-radius: 4px;
  overflow: hidden;
}

.header-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.header-info {
  display: flex;
  flex-direction: column;
}

.header-name {
  font-size: 15px;
  font-weight: 500;
  color: #333;
  margin: 0;
}

.header-status {
  font-size: 12px;
  color: #999;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 4px;
  position: relative;
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.loading-state,
.empty-chat {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: #999;
  min-height: 200px;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #1890ff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.loading-state p,
.empty-chat p {
  font-size: 13px;
  margin: 0;
}

.empty-chat-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
}

.empty-chat-avatar {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  overflow: hidden;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.empty-chat-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.empty-chat h3 {
  font-size: 18px;
  font-weight: 500;
  color: #333;
  margin: 0 0 8px 0;
}

.empty-chat p {
  font-size: 14px;
  color: #999;
  margin: 0 0 24px 0;
}

.quick-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: center;
}

.quick-action-btn {
  padding: 8px 16px;
  border: 1px solid #e5e5e5;
  background: #fff;
  color: #333;
  border-radius: 20px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.quick-action-btn:hover {
  background: #f5f5f5;
  border-color: #d9d9d9;
}

.im-messenger.dark-mode .empty-chat h3 {
  color: #e0e0e0;
}

.im-messenger.dark-mode .empty-chat p {
  color: #888;
}

.im-messenger.dark-mode .quick-action-btn {
  background: #2a2a2a;
  border-color: #333;
  color: #e0e0e0;
}

.im-messenger.dark-mode .quick-action-btn:hover {
  background: #3a3a3a;
  border-color: #444;
}

.messages-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message-wrapper {
  display: flex;
  gap: 8px;
  max-width: 70%;
  animation: messageSlideIn 0.3s ease-out;
}

@keyframes messageSlideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message-wrapper.own {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.message-avatar {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}

.message-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.message-content {
  flex: 1;
  min-width: 0;
}

.message-bubble {
  background: #fff;
  padding: 8px 12px;
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  position: relative;
}

.message-wrapper.own .message-bubble {
  background: #95ec69;
  border-radius: 8px 8px 0 8px;
}

.message-wrapper:not(.own) .message-bubble {
  border-radius: 8px 8px 8px 0;
}

.message-text {
  font-size: 14px;
  color: #333;
  margin: 0 0 4px 0;
  word-wrap: break-word;
  line-height: 1.5;
}

.message-meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
}

.message-time {
  font-size: 11px;
  color: #999;
}

.message-status {
  display: flex;
  align-items: center;
  color: #999;
}

.message-status svg {
  opacity: 0.7;
}

/* Input Area */
.input-area {
  background: #fff;
  border-top: 1px solid #e5e5e5;
  padding: 12px 16px;
  flex-shrink: 0;
}

.input-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.message-form {
  display: flex;
  gap: 8px;
  align-items: flex-end;
}

.message-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  background: #fff;
  font-size: 14px;
  color: #333;
  outline: none;
  resize: none;
  min-height: 36px;
  max-height: 120px;
  font-family: inherit;
  line-height: 1.5;
  transition: height 0.1s ease;
}

.message-input:focus {
  border-color: #1890ff;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.1);
}

.message-input::placeholder {
  color: #999;
}

.edit-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  background: #e8f5e9;
  border-radius: 12px;
  margin-bottom: 8px;
  font-size: 12px;
  color: #2e7d32;
}

.cancel-edit-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.cancel-edit-btn:hover {
  opacity: 1;
}

.reply-indicator {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #f0f7ff;
  border-left: 3px solid #1890ff;
  border-radius: 4px;
  margin-bottom: 8px;
}

.reply-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.reply-label {
  font-size: 12px;
  font-weight: 500;
  color: #1890ff;
}

.timeline-container {
  height: 100%;
  overflow: hidden;
}

.reply-text {
  font-size: 13px;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cancel-reply-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
  transition: opacity 0.2s ease;
  margin-left: 8px;
}

.cancel-reply-btn:hover {
  opacity: 1;
}

.file-preview-bar {
  padding: 8px 12px;
  background: #f5f5f5;
  border-radius: 8px;
  margin-bottom: 8px;
}

.file-preview-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.file-preview-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: #fff;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  position: relative;
}

.file-image-preview {
  width: 48px;
  height: 48px;
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}

.file-image-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.file-icon-preview {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f0f0f0;
  border-radius: 4px;
  flex-shrink: 0;
  color: #666;
}

.file-name {
  font-size: 12px;
  color: #333;
  max-width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.remove-file-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
  transition: opacity 0.2s ease;
  color: #666;
}

.remove-file-btn:hover {
  opacity: 1;
  color: #ff4d4f;
}

.im-messenger.dark-mode .file-preview-bar {
  background: #2a2a2a;
}

.im-messenger.dark-mode .file-preview-item {
  background: #1a1a1a;
  border-color: #333;
}

.im-messenger.dark-mode .file-icon-preview {
  background: #333;
  color: #999;
}

.im-messenger.dark-mode .file-name {
  color: #e0e0e0;
}

.im-messenger.dark-mode .remove-file-btn {
  color: #999;
}

.im-messenger.dark-mode .remove-file-btn:hover {
  color: #ff7875;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: #f5f5f5;
  border-bottom: 1px solid #e5e5e5;
}

.search-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s ease;
}

.search-input:focus {
  border-color: #1890ff;
}

.search-results-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #666;
}

.search-nav-btn {
  background: none;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  padding: 4px 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.search-nav-btn:hover:not(:disabled) {
  background: #f5f5f5;
  border-color: #1890ff;
}

.search-nav-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.close-search-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
  transition: opacity 0.2s ease;
  color: #666;
}

.close-search-btn:hover {
  opacity: 1;
  color: #ff4d4f;
}

.search-highlight {
  animation: highlight-pulse 2s ease-in-out;
}

@keyframes highlight-pulse {
  0%, 100% {
    background-color: transparent;
  }
  50% {
    background-color: rgba(24, 144, 255, 0.2);
  }
}

.im-messenger.dark-mode .search-bar {
  background: #2a2a2a;
  border-bottom-color: #333;
}

.im-messenger.dark-mode .search-input {
  background: #1a1a1a;
  border-color: #333;
  color: #e0e0e0;
}

.im-messenger.dark-mode .search-input:focus {
  border-color: #07c160;
}

.im-messenger.dark-mode .search-results-info {
  color: #999;
}

.im-messenger.dark-mode .search-nav-btn {
  border-color: #333;
  color: #e0e0e0;
}

.im-messenger.dark-mode .search-nav-btn:hover:not(:disabled) {
  background: #3a3a3a;
  border-color: #07c160;
}

.im-messenger.dark-mode .close-search-btn {
  color: #999;
}

.im-messenger.dark-mode .close-search-btn:hover {
  color: #ff7875;
}

.mention-autocomplete {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  max-height: 200px;
  overflow-y: auto;
  margin-bottom: 8px;
  z-index: 100;
}

.mention-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.mention-item:hover,
.mention-item.active {
  background: #f5f5f5;
}

.mention-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
}

.mention-name {
  font-size: 14px;
  color: #333;
}

.im-messenger.dark-mode .mention-autocomplete {
  background: #2a2a2a;
  border-color: #333;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.im-messenger.dark-mode .mention-item:hover,
.im-messenger.dark-mode .mention-item.active {
  background: #3a3a3a;
}

.im-messenger.dark-mode .mention-name {
  color: #e0e0e0;
}

.emoji-picker {
  position: absolute;
  bottom: 100%;
  left: 0;
  background: #fff;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 12px;
  margin-bottom: 8px;
  z-index: 100;
}

.emoji-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}

.emoji-button {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: background 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.emoji-button:hover {
  background: #f5f5f5;
}

.im-messenger.dark-mode .emoji-picker {
  background: #2a2a2a;
  border-color: #333;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.im-messenger.dark-mode .emoji-button:hover {
  background: #3a3a3a;
}

.send-button {
  width: 36px;
  height: 36px;
  border: none;
  background: #1890ff;
  color: #fff;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.send-button:hover:not(:disabled) {
  background: #40a9ff;
}

.send-button:active:not(:disabled) {
  background: #096dd9;
}

.send-button:disabled {
  background: #d9d9d9;
  cursor: not-allowed;
}

/* Empty Main State */
.empty-main {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
}

.empty-content {
  text-align: center;
  color: #999;
}

.empty-content svg {
  margin-bottom: 16px;
  opacity: 0.3;
}

.empty-content h2 {
  font-size: 18px;
  font-weight: 500;
  color: #666;
  margin: 0 0 8px 0;
}

.empty-content p {
  font-size: 14px;
  margin: 0;
}

/* Scrollbar Styling */
.chat-list::-webkit-scrollbar,
.messages-container::-webkit-scrollbar {
  width: 6px;
}

.chat-list::-webkit-scrollbar-track,
.messages-container::-webkit-scrollbar-track {
  background: transparent;
}

.chat-list::-webkit-scrollbar-thumb,
.messages-container::-webkit-scrollbar-thumb {
  background: #d9d9d9;
  border-radius: 3px;
}

.chat-list::-webkit-scrollbar-thumb:hover,
.messages-container::-webkit-scrollbar-thumb:hover {
  background: #bfbfbf;
}

/* Dark Mode */
.im-messenger.dark-mode {
  background: #1a1a1a;
}

.im-messenger.dark-mode .nav-sidebar {
  background: #1e1e1e;
}

.im-messenger.dark-mode .nav-header {
  border-bottom-color: #333;
}

.im-messenger.dark-mode .nav-item {
  color: #888;
}

.im-messenger.dark-mode .nav-item:hover {
  background: #333;
  color: #fff;
}

.im-messenger.dark-mode .nav-item.active {
  background: transparent;
  color: #07c160;
}

.im-messenger.dark-mode .content-sidebar {
  background: #1e1e1e;
  border-right-color: #333;
}

.im-messenger.dark-mode .sidebar-header {
  background: #1e1e1e;
  border-bottom-color: #333;
}

.im-messenger.dark-mode .sidebar-title {
  color: #e0e0e0;
}

.im-messenger.dark-mode .chat-item {
  border-bottom-color: #2a2a2a;
}

.im-messenger.dark-mode .chat-item:hover {
  background: #2a2a2a;
}

.im-messenger.dark-mode .chat-item.active {
  background: #1a3a2a;
}

.im-messenger.dark-mode .chat-name {
  color: #e0e0e0;
}

.im-messenger.dark-mode .chat-time,
.im-messenger.dark-mode .preview-text {
  color: #888;
}

.im-messenger.dark-mode .chat-main {
  background: #1a1a1a;
}

.im-messenger.dark-mode .chat-window-header {
  background: #1e1e1e;
  border-bottom-color: #333;
}

.im-messenger.dark-mode .header-name {
  color: #e0e0e0;
}

.im-messenger.dark-mode .header-status {
  color: #888;
}

.im-messenger.dark-mode .message-bubble {
  background: #2a2a2a;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.im-messenger.dark-mode .message-wrapper.own .message-bubble {
  background: #1a5a3a;
}

.im-messenger.dark-mode .message-text {
  color: #e0e0e0;
}

.im-messenger.dark-mode .message-time,
.im-messenger.dark-mode .message-status {
  color: #888;
}

.im-messenger.dark-mode .input-area {
  background: #1e1e1e;
  border-top-color: #333;
}

.im-messenger.dark-mode .message-input {
  background: #2a2a2a;
  border-color: #333;
  color: #e0e0e0;
}

.im-messenger.dark-mode .message-input:focus {
  border-color: #07c160;
  box-shadow: 0 0 0 2px rgba(7, 193, 96, 0.1);
}

.im-messenger.dark-mode .message-input::placeholder {
  color: #666;
}

.im-messenger.dark-mode .icon-button {
  color: #888;
}

.im-messenger.dark-mode .icon-button:hover {
  color: #e0e0e0;
}

.im-messenger.dark-mode .empty-main {
  background: #1a1a1a;
}

.im-messenger.dark-mode .empty-content {
  color: #888;
}

.im-messenger.dark-mode .empty-content h2 {
  color: #e0e0e0;
}

.im-messenger.dark-mode .settings-item {
  border-bottom-color: #2a2a2a;
}

.im-messenger.dark-mode .settings-item:hover {
  background: #2a2a2a;
}

.im-messenger.dark-mode .settings-label {
  color: #e0e0e0;
}

.im-messenger.dark-mode .settings-value {
  color: #888;
}

/* Context Menu */
.context-menu {
  position: fixed;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px;
  z-index: 1000;
  min-width: 120px;
  animation: contextMenuFadeIn 0.15s ease-out;
}

@keyframes contextMenuFadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-item {
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  color: #333;
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background 0.15s ease;
}

.context-menu-item:hover {
  background: #f5f5f5;
}

.context-menu-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.context-menu-item svg {
  flex-shrink: 0;
}

.im-messenger.dark-mode .context-menu {
  background: #2a2a2a;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.im-messenger.dark-mode .context-menu-item {
  color: #e0e0e0;
}

.im-messenger.dark-mode .context-menu-item:hover {
  background: #3a3a3a;
}

/* Modal Dialog */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: modalFadeIn 0.2s ease-out;
}

@keyframes modalFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal-content {
  background: #fff;
  border-radius: 12px;
  padding: 24px;
  width: 90%;
  max-width: 400px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  animation: modalSlideIn 0.3s ease-out;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.modal-content h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #333;
  margin-bottom: 8px;
}

.form-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s ease;
}

.form-input:focus {
  outline: none;
  border-color: #07c160;
}

.form-hint {
  margin: 0 0 16px;
  font-size: 13px;
  color: #666;
}

.digit-code {
  margin: 0 6px;
  padding: 2px 6px;
  border-radius: 4px;
  background: #f5f5f5;
  font-family: ui-monospace, monospace;
}

.form-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.tab-button {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  background: #fafafa;
  font-size: 13px;
  cursor: pointer;
}

.tab-button.active {
  border-color: #07c160;
  background: #e8f8ef;
  color: #07c160;
}

.link-button {
  border: none;
  background: none;
  color: #07c160;
  cursor: pointer;
  font-size: inherit;
  padding: 0;
}

.settings-item-static {
  cursor: default;
}

.member-selection {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  padding: 8px;
}

.member-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.member-item:hover {
  background: #f5f5f5;
}

.member-checkbox {
  cursor: pointer;
}

.member-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
}

.member-name {
  font-size: 14px;
  color: #333;
}

.modal-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
  justify-content: flex-end;
}

.primary-button,
.secondary-button {
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.primary-button {
  background: #07c160;
  color: #fff;
  border: none;
}

.primary-button:hover:not(:disabled) {
  background: #06ad56;
}

.primary-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.secondary-button {
  background: #fff;
  color: #333;
  border: 1px solid #e5e5e5;
}

.secondary-button:hover {
  background: #f5f5f5;
}

.im-messenger.dark-mode .modal-content {
  background: #2a2a2a;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
}

.im-messenger.dark-mode .modal-content h3 {
  color: #e0e0e0;
}

.im-messenger.dark-mode .form-group label {
  color: #e0e0e0;
}

.im-messenger.dark-mode .form-input {
  background: #1a1a1a;
  border-color: #333;
  color: #e0e0e0;
}

.im-messenger.dark-mode .form-input:focus {
  border-color: #07c160;
}

.im-messenger.dark-mode .member-selection {
  background: #1a1a1a;
  border-color: #333;
}

.im-messenger.dark-mode .member-item:hover {
  background: #2a2a2a;
}

.im-messenger.dark-mode .member-name {
  color: #e0e0e0;
}

.im-messenger.dark-mode .secondary-button {
  background: #2a2a2a;
  border-color: #333;
  color: #e0e0e0;
}

.im-messenger.dark-mode .secondary-button:hover {
  background: #3a3a3a;
}

.group-settings-content {
  max-width: 500px;
}

.group-info-section {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: #f5f5f5;
  border-radius: 8px;
  margin-bottom: 16px;
}

.group-avatar-large {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  overflow: hidden;
  flex-shrink: 0;
}

.group-avatar-large img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.group-details {
  flex: 1;
  min-width: 0;
}

.group-details h4 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.group-details p {
  margin: 0 0 4px 0;
  font-size: 13px;
  color: #666;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.group-meta {
  font-size: 12px;
  color: #999;
}

.member-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  padding: 8px;
}

.member-list-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border-radius: 4px;
}

.member-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
}

.member-list-item .member-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.member-role {
  font-size: 12px;
  color: #999;
}

.online-dot {
  width: 8px;
  height: 8px;
  background: #52c41a;
  border-radius: 50%;
  flex-shrink: 0;
}

.danger-button {
  background: #ff4d4f;
  color: #fff;
  border: none;
}

.danger-button:hover {
  background: #ff7875;
}

.im-messenger.dark-mode .group-info-section {
  background: #2a2a2a;
}

.im-messenger.dark-mode .group-details h4 {
  color: #e0e0e0;
}

.im-messenger.dark-mode .group-details p {
  color: #999;
}

.im-messenger.dark-mode .group-meta {
  color: #666;
}

.im-messenger.dark-mode .member-list {
  background: #1a1a1a;
  border-color: #333;
}

.im-messenger.dark-mode .member-list-item .member-name {
  color: #e0e0e0;
}

.im-messenger.dark-mode .member-role {
  color: #666;
}

/* WebChat Desktop alignment (full-width main content mode) */
.im-messenger.webchat-desktop {
  --wx-green: #07c160;
  --wx-bg: #191919;
  --wx-panel: #262626;
  --wx-rail: #2e2e2e;
  --wx-border: #2a2a2a;
  --wx-text: #ececec;
  --wx-muted: #8a8a8a;
  --wx-icon: #b2b2b2;
  background: var(--wx-bg);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif;
}

.im-messenger.webchat-desktop .nav-sidebar {
  width: 54px;
  background: #2e2e2e;
}

.im-messenger.webchat-desktop .nav-header {
  padding: 14px 7px;
  border-bottom-color: #3a3a3a;
}

.im-messenger.webchat-desktop .user-avatar-small {
  width: 28px;
  height: 28px;
  border-radius: 4px;
}

.im-messenger.webchat-desktop .nav-item {
  width: 40px;
  height: 40px;
  margin: 0 auto;
  color: #8a8a8a;
}

.im-messenger.webchat-desktop .nav-item:hover {
  background: transparent;
  color: #d6d6d6;
}

.im-messenger.webchat-desktop .nav-item.active .im-icon:not(.im-icon--active) {
  color: var(--wx-green);
}

.im-messenger.webchat-desktop .nav-item.active {
  color: var(--wx-green);
}

.im-messenger.webchat-desktop .nav-badge {
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  font-size: 10px;
  line-height: 16px;
  border-radius: 8px;
  background: #fa5151;
  color: #fff;
  font-weight: 500;
}

.im-messenger.webchat-desktop .nav-badge-muted {
  background: #555;
  color: #ddd;
}

.im-messenger.webchat-desktop .content-sidebar {
  width: 300px;
  background: #262626;
  border-right: 1px solid #1a1a1a;
}

.im-messenger.webchat-desktop .webchat-list-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 12px 10px;
}

.im-messenger.webchat-desktop .webchat-search {
  flex: 1;
  padding: 0;
  margin: 0;
}

.im-messenger.webchat-desktop .webchat-search .search-input {
  background: #1a1a1a;
  border: none;
  border-radius: 4px;
  color: #ececec;
  padding: 7px 12px 7px 34px;
  font-size: 13px;
}

.im-messenger.webchat-desktop .webchat-search .search-input:focus {
  box-shadow: none;
  background: #1a1a1a;
}

.im-messenger.webchat-desktop .webchat-search .search-icon-svg {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: #6b6b6b;
  pointer-events: none;
}

.im-messenger.webchat-desktop .webchat-search .search-icon {
  display: none;
}

.im-messenger.webchat-desktop .webchat-toolbar-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--wx-icon);
  cursor: pointer;
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.im-messenger.webchat-desktop .webchat-toolbar-btn:hover {
  background: #333;
  color: #fff;
}

.im-messenger.webchat-desktop .chat-item {
  padding: 11px 12px;
  border-bottom: none;
}

.im-messenger.webchat-desktop .chat-item:hover {
  background: #2f2f2f;
}

.im-messenger.webchat-desktop .chat-item.active {
  background: #3a3a3a;
}

.im-messenger.webchat-desktop .chat-avatar {
  border-radius: 6px;
}

.im-messenger.webchat-desktop .unread-badge {
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  font-size: 11px;
  line-height: 18px;
  border-radius: 9px;
  background: #fa5151;
  color: #fff;
  font-weight: 500;
}

.im-messenger.webchat-desktop .chat-name {
  color: #ececec;
  font-size: 14px;
}

.im-messenger.webchat-desktop .chat-time,
.im-messenger.webchat-desktop .preview-text {
  color: #8a8a8a;
  font-size: 12px;
}

.im-messenger.webchat-desktop .chat-main,
.im-messenger.webchat-desktop .empty-main,
.im-messenger.webchat-desktop .webchat-empty-main {
  background: #191919;
}

.im-messenger.webchat-desktop .chat-window-header,
.im-messenger.webchat-desktop .webchat-header {
  background: #191919;
  border-bottom: 1px solid #2a2a2a;
  padding: 14px 18px;
}

.im-messenger.webchat-desktop .header-name {
  color: #ececec;
  font-size: 16px;
  font-weight: 500;
}

.im-messenger.webchat-desktop .messages-container {
  background: #191919;
}

.im-messenger.webchat-desktop .message-bubble {
  background: #2f2f2f;
  box-shadow: none;
  border-radius: 6px;
}

.im-messenger.webchat-desktop .message-wrapper.own .message-bubble {
  background: #95ec69;
}

.im-messenger.webchat-desktop .message-wrapper:not(.own) .message-bubble {
  border-radius: 6px;
}

.im-messenger.webchat-desktop .message-text {
  color: #111;
}

.im-messenger.webchat-desktop .message-wrapper:not(.own) .message-text {
  color: #ececec;
}

.im-messenger.webchat-desktop .message-time {
  color: #8a8a8a;
}

.im-messenger.webchat-desktop .input-area {
  background: #191919;
  border-top: 1px solid #2a2a2a;
  padding: 8px 16px 14px;
}

.im-messenger.webchat-desktop .input-toolbar {
  margin-bottom: 6px;
}

.im-messenger.webchat-desktop .input-toolbar .icon-button {
  color: #b2b2b2;
}

.im-messenger.webchat-desktop .input-toolbar .icon-button:hover {
  background: transparent;
  color: #fff;
}

.im-messenger.webchat-desktop .webchat-message-form {
  flex-direction: column;
  align-items: stretch;
  gap: 0;
}

.im-messenger.webchat-desktop .message-input {
  background: transparent;
  border: none;
  color: #ececec;
  min-height: 88px;
  padding: 8px 0;
  resize: none;
  box-shadow: none;
}

.im-messenger.webchat-desktop .message-input:focus {
  outline: none;
  border: none;
  box-shadow: none;
}

.im-messenger.webchat-desktop .message-input::placeholder {
  color: #666;
}

.im-messenger.webchat-desktop .send-button {
  align-self: flex-end;
  background: var(--wx-green);
}

.im-messenger.webchat-desktop .webchat-send-button {
  min-width: 72px;
  height: 32px;
  padding: 0 14px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  color: #fff;
  letter-spacing: 0.02em;
}

.im-messenger.webchat-desktop .webchat-send-button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.im-messenger.webchat-desktop .header-actions .icon-button {
  color: var(--wx-icon);
}

.im-messenger.webchat-desktop .header-actions .icon-button:hover {
  background: transparent;
  color: #fff;
}

.im-messenger.webchat-desktop .empty-content {
  color: #666;
}

.im-messenger.webchat-desktop .webchat-empty-logo {
  margin-bottom: 16px;
  opacity: 0.28;
  color: var(--wx-green);
}

.im-messenger.webchat-desktop .empty-content h2 {
  color: #8a8a8a;
  font-weight: 400;
}

.im-messenger.webchat-desktop .empty-content p {
  color: #666;
}

.im-messenger.webchat-desktop .chat-list::-webkit-scrollbar-thumb,
.im-messenger.webchat-desktop .messages-container::-webkit-scrollbar-thumb {
  background: #444;
}

/* Group grid avatar (WebChat 九宫格) */
.group-grid-avatar {
  width: 100%;
  height: 100%;
  display: grid;
  gap: 1px;
  background: #1a1a1a;
  overflow: hidden;
  border-radius: 4px;
}

.group-grid-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.group-grid-avatar.grid-count-1 {
  grid-template-columns: 1fr;
  grid-template-rows: 1fr;
}

.group-grid-avatar.grid-count-2 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr;
}

.group-grid-avatar.grid-count-3 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}

.group-grid-avatar.grid-count-3 img:first-child {
  grid-row: span 2;
}

.group-grid-avatar.grid-count-4 {
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
}

.group-grid-avatar.grid-count-5,
.group-grid-avatar.grid-count-6,
.group-grid-avatar.grid-count-7,
.group-grid-avatar.grid-count-8,
.group-grid-avatar.grid-count-9 {
  grid-template-columns: repeat(3, 1fr);
  grid-template-rows: repeat(3, 1fr);
}

.mute-indicator {
  display: inline-flex;
  align-items: center;
  color: #8a8a8a;
  margin-right: 4px;
}

/* WebChat macOS address book (collapsible categories + counts) */
.contact-manage-row {
  padding: 8px 12px 6px;
  list-style: none;
}

.contact-manage-btn {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: #333;
  border: none;
  border-radius: 4px;
  color: #ececec;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.contact-manage-btn:hover {
  background: #3a3a3a;
}

.contact-category-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px 9px 10px;
  cursor: pointer;
  list-style: none;
  transition: background 0.15s ease;
}

.contact-category-row:hover {
  background: #2f2f2f;
}

.category-chevron {
  color: #777;
  flex-shrink: 0;
  width: 14px;
}

.category-label {
  flex: 1;
  min-width: 0;
  color: #ececec;
  font-size: 14px;
  line-height: 1.3;
}

.category-count {
  flex-shrink: 0;
  color: #8a8a8a;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  min-width: 28px;
  text-align: right;
}

.contact-nested-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px 8px 34px;
  cursor: pointer;
  list-style: none;
  transition: background 0.15s ease;
}

.contact-nested-item:hover,
.contact-nested-item.active {
  background: #2f2f2f;
}

.contact-nested-item .chat-avatar-small {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
}

.nested-item-name {
  color: #ececec;
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.contact-nested-empty {
  padding: 8px 12px 8px 34px;
  color: #666;
  font-size: 13px;
  list-style: none;
}

.im-messenger.webchat-desktop .contact-manage-row {
  padding-top: 4px;
}

.im-messenger.webchat-desktop .contact-category-row {
  padding: 10px 12px 10px 10px;
}

.contact-manage-actions {
  display: flex;
  gap: 8px;
  padding: 0 12px 8px;
  list-style: none;
}

.contact-manage-action {
  flex: 1;
  border: none;
  border-radius: 4px;
  background: #333;
  color: #ececec;
  font-size: 13px;
  padding: 8px 10px;
  cursor: pointer;
}

.contact-manage-action:hover {
  background: #3a3a3a;
}

.hidden-input {
  display: none;
}

.message-reply-quote {
  margin-bottom: 6px;
  padding: 6px 8px;
  border-left: 3px solid #07c160;
  background: rgba(255, 255, 255, 0.06);
  color: #9ca3af;
  font-size: 12px;
  line-height: 1.4;
  border-radius: 4px;
  word-break: break-word;
}

.im-messenger.webchat-desktop .message-wrapper.own .message-reply-quote {
  border-left-color: #059a4c;
  background: rgba(0, 0, 0, 0.08);
  color: #444;
}

.message-time-divider {
  display: flex;
  justify-content: center;
  margin: 16px 0 12px;
}

.message-time-divider span {
  padding: 4px 10px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: #8a8a8a;
  font-size: 12px;
  line-height: 1.2;
}

.im-messenger.webchat-desktop .message-time-divider span {
  background: #2a2a2a;
  color: #8a8a8a;
}

.conversation-menu {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  min-width: 160px;
  background: #2f2f2f;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  z-index: 30;
  overflow: hidden;
}

.conversation-menu-item {
  display: block;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: #ececec;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.conversation-menu-item:hover {
  background: #3a3a3a;
}

/* Responsive Design */
@media (max-width: 768px) {
  .chat-sidebar {
    width: 240px;
  }
  
  .chat-item {
    padding: 10px 12px;
  }
  
  .chat-avatar {
    width: 40px;
    height: 40px;
  }
  
  .message-wrapper {
    max-width: 85%;
  }
}
</style>
