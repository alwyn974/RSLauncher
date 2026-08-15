<script setup lang="ts">
/**
 * Active-account chip with a dropdown to switch, add, or remove accounts.
 */
import { onBeforeUnmount, onMounted, ref } from "vue";
import { launcher } from "../stores/launcher";
import AvatarBlock from "./AvatarBlock.vue";
import PixelIcon from "./PixelIcon.vue";

const open = ref(false);
const root = ref<HTMLElement | null>(null);

function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false;
}
onMounted(() => document.addEventListener("mousedown", onDocClick));
onBeforeUnmount(() => document.removeEventListener("mousedown", onDocClick));

function choose(id: string) {
  launcher.setActiveAccount(id);
  open.value = false;
}

function addAccount() {
  launcher.login();
  open.value = false;
}

function remove(id: string) {
  launcher.removeAccount(id);
}
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      class="mc-panel flex cursor-pointer items-center gap-2 py-1 pr-3 pl-1 transition-colors duration-150 hover:bg-mc-panel-2"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="open = !open"
    >
      <AvatarBlock
        v-if="launcher.activeAccount.value"
        :uuid="launcher.activeAccount.value.uuid"
        :username="launcher.activeAccount.value.username"
        :seed="launcher.activeAccount.value.avatarSeed"
        :size="28"
      />
      <span class="font-pixel pixel-shadow-sm max-w-40 truncate text-xs text-mc-text">
        {{ launcher.activeAccount.value?.username ?? "No account" }}
      </span>
      <PixelIcon name="back" :size="12" class="text-mc-muted -rotate-90" />
    </button>

    <div
      v-if="open"
      class="mc-panel absolute left-0 z-50 mt-1 w-64 min-w-0 overflow-hidden p-1"
      role="listbox"
      aria-label="Accounts"
    >
      <div
        v-for="account in launcher.state.accounts"
        :key="account.id"
        role="option"
        :aria-selected="account.id === launcher.state.activeAccountId"
        class="group flex cursor-pointer items-center gap-2 p-1.5 transition-colors duration-150 hover:bg-mc-panel-2"
        @click="choose(account.id)"
      >
        <AvatarBlock
          :uuid="account.uuid"
          :username="account.username"
          :seed="account.avatarSeed"
          :size="24"
        />
        <span class="min-w-0 flex-1 truncate text-sm text-mc-text">
          {{ account.username }}
        </span>
        <span
          v-if="account.needsReauth"
          class="font-pixel text-[8px] text-mc-gold"
        >
          Reconnect
        </span>
        <PixelIcon
          v-if="account.id === launcher.state.activeAccountId"
          name="check"
          :size="14"
          class="text-mc-gold"
        />
        <button
          type="button"
          class="cursor-pointer p-0.5 text-mc-faint opacity-0 transition-opacity duration-150 group-hover:opacity-100 hover:text-mc-red"
          :aria-label="`Remove ${account.username}`"
          @click.stop="remove(account.id)"
        >
          <PixelIcon name="trash" :size="14" />
        </button>
      </div>

      <button
        type="button"
        class="flex w-full cursor-pointer items-center gap-2 border-t-2 border-mc-border p-1.5 text-sm text-mc-muted transition-colors duration-150 hover:bg-mc-panel-2 hover:text-mc-text"
        @click="addAccount"
      >
        <span class="mc-bevel flex size-6 items-center justify-center bg-mc-panel-2 text-mc-gold">
          <PixelIcon name="plus" :size="12" />
        </span>
        {{ launcher.state.loginPending ? "Signing in..." : "Add account" }}
      </button>

      <p
        v-if="launcher.state.loginError"
        class="max-h-32 min-w-0 overflow-y-auto border-t-2 border-mc-border px-2 py-2 text-sm leading-relaxed break-all whitespace-pre-wrap text-mc-red"
        role="alert"
      >
        {{ launcher.state.loginError }}
      </p>
    </div>
  </div>
</template>
