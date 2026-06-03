<!--
  Exodus Browser — Profile management settings.
-->
<template>
  <section class="settings-section" data-testid="profile-settings">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <div class="current-profile">
        <h4>{{ ui.currentProfile }}</h4>
        <div class="profile-card">
          <div class="profile-avatar">{{ currentProfile.name.charAt(0).toUpperCase() }}</div>
          <div class="profile-info">
            <strong>{{ currentProfile.name }}</strong>
            <span class="muted">{{ currentProfile.email || ui.noEmail }}</span>
          </div>
          <button type="button" class="nav-button secondary" @click="() => void editProfile()" data-testid="edit-profile">
            {{ ui.edit }}
          </button>
        </div>
      </div>

      <div class="profile-list">
        <h4>{{ ui.profiles }}</h4>
        <div v-for="profile in profiles" :key="profile.id" class="profile-item">
          <div class="profile-avatar">{{ profile.name.charAt(0).toUpperCase() }}</div>
          <div class="profile-info">
            <strong>{{ profile.name }}</strong>
            <span class="muted">{{ profile.email || ui.noEmail }}</span>
          </div>
          <div class="profile-actions">
            <button
              v-if="profile.id !== currentProfile.id"
              type="button"
              class="nav-button secondary"
              @click="() => void switchProfile(profile.id)"
              data-testid="switch-profile"
            >
              {{ ui.switch }}
            </button>
            <button
              v-if="!profile.isDefault"
              type="button"
              class="nav-button secondary danger"
              @click="() => void deleteProfile(profile.id)"
              data-testid="delete-profile"
            >
              {{ ui.delete }}
            </button>
          </div>
        </div>
      </div>

      <button type="button" class="nav-button secondary" @click="() => void createProfile()" data-testid="create-profile">
        {{ ui.createProfile }}
      </button>

      <div class="guest-profile">
        <label class="checkbox-row">
          <input v-model="enableGuestProfile" type="checkbox" @change="() => void persist()" data-testid="guest-profile" />
          <span>{{ ui.guestProfile }}</span>
        </label>
        <p class="settings-hint">{{ ui.guestProfileHint }}</p>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — profile management settings.
 */
import { ref, computed, onMounted } from 'vue';
import { type AppLocale } from '@/lib/appLocale';
import { profileSettingsStrings } from '@/lib/profileSettingsUi';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => profileSettingsStrings(props.uiLocale));

type Profile = {
  id: string;
  name: string;
  email?: string;
  isDefault: boolean;
  createdAt: number;
};

const loading = ref(true);
const currentProfile = ref<Profile>({
  id: 'default',
  name: 'Default Profile',
  isDefault: true,
  createdAt: Date.now(),
});
const profiles = ref<Profile[]>([
  {
    id: 'default',
    name: 'Default Profile',
    isDefault: true,
    createdAt: Date.now(),
  },
]);
const enableGuestProfile = ref(false);

const STORAGE_KEY = 'exodus-profile-settings';

const DEFAULT_PROFILE: Profile = {
  id: 'default',
  name: 'Default Profile',
  isDefault: true,
  createdAt: Date.now(),
};

/** Load profile settings from localStorage. */
function load(): void {
  loading.value = true;
  try {
    const savedSettings = localStorage.getItem(STORAGE_KEY);
    if (savedSettings) {
      const settings = JSON.parse(savedSettings);
      currentProfile.value = settings.currentProfile || DEFAULT_PROFILE;
      profiles.value = settings.profiles || [DEFAULT_PROFILE];
      enableGuestProfile.value = Boolean(settings.enableGuestProfile);
    } else {
      currentProfile.value = DEFAULT_PROFILE;
      profiles.value = [DEFAULT_PROFILE];
      enableGuestProfile.value = false;
    }
  } catch (error) {
    console.error('ProfileSettings.load failed:', error);
    currentProfile.value = DEFAULT_PROFILE;
    profiles.value = [DEFAULT_PROFILE];
  } finally {
    loading.value = false;
  }
}

/** Persist profile settings to localStorage. */
function persist(): void {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        currentProfile: currentProfile.value,
        profiles: profiles.value,
        enableGuestProfile: enableGuestProfile.value,
      }),
    );
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('ProfileSettings.persist failed:', error);
    emit('status', ui.value.saveError);
  }
}

/** Create a new profile. */
function createProfile(): void {
  const newProfile: Profile = {
    id: `profile-${Date.now()}`,
    name: `Profile ${profiles.value.length + 1}`,
    isDefault: false,
    createdAt: Date.now(),
  };
  profiles.value.push(newProfile);
  persist();
  emit('status', ui.value.profileCreated);
}

/** Switch to a different profile. */
function switchProfile(profileId: string): void {
  const profile = profiles.value.find((p) => p.id === profileId);
  if (profile) {
    currentProfile.value = profile;
    persist();
    emit('status', ui.value.profileSwitched);
  }
}

/** Delete a profile. */
function deleteProfile(profileId: string): void {
  if (!confirm('Delete this profile?')) return;
  profiles.value = profiles.value.filter((p) => p.id !== profileId);
  if (currentProfile.value.id === profileId) {
    currentProfile.value = profiles.value[0];
  }
  persist();
  emit('status', ui.value.profileDeleted);
}

/** Edit current profile. */
function editProfile(): void {
  const newName = prompt(ui.value.enterProfileName, currentProfile.value.name);
  if (newName && newName.trim()) {
    currentProfile.value.name = newName.trim();
    const profileIndex = profiles.value.findIndex((p) => p.id === currentProfile.value.id);
    if (profileIndex !== -1) {
      profiles.value[profileIndex].name = newName.trim();
    }
    persist();
    emit('status', ui.value.profileUpdated);
  }
}

onMounted(() => {
  load();
});
</script>

<style scoped>
.profile-card,
.profile-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  margin-bottom: 8px;
}

.profile-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--primary-color);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 18px;
}

.profile-info {
  flex: 1;
}

.profile-info strong {
  display: block;
}

.profile-info .muted {
  font-size: 12px;
  color: var(--text-muted);
}

.profile-actions {
  display: flex;
  gap: 8px;
}

.current-profile,
.profile-list,
.guest-profile {
  margin-bottom: 24px;
}

.guest-profile {
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}
</style>
