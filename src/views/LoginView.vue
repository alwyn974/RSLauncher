<script setup lang="ts">
/**
 * First-run / signed-out screen: Microsoft sign-in, plus any saved
 * accounts that can be picked directly.
 */
import { launcher } from "../stores/launcher";
import AvatarBlock from "../components/AvatarBlock.vue";
import PixelButton from "../components/PixelButton.vue";
import PixelIcon from "../components/PixelIcon.vue";
import SignInStatus from "../components/SignInStatus.vue";
</script>

<template>
  <main class="relative flex h-full flex-col items-center justify-center gap-6 p-8">
    <div class="text-center">
      <h1 class="font-pixel pixel-shadow text-4xl tracking-wide text-white">
        RS<span class="text-mc-gold">LAUNCHER</span>
      </h1>
      <p class="mt-3 text-sm text-mc-muted">Pick a pack. Press Play.</p>
    </div>

    <PixelButton
      variant="stone"
      class="px-6 py-3 text-sm"
      :disabled="launcher.state.loginPending"
      @click="launcher.login()"
    >
      <!-- Microsoft mark -->
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true" shape-rendering="crispEdges">
        <rect width="7" height="7" fill="#f25022" />
        <rect x="9" width="7" height="7" fill="#7fba00" />
        <rect y="9" width="7" height="7" fill="#00a4ef" />
        <rect x="9" y="9" width="7" height="7" fill="#ffb900" />
      </svg>
      {{ launcher.state.loginPending ? "Signing in…" : "Sign in with Microsoft" }}
    </PixelButton>

    <SignInStatus v-if="launcher.state.loginPending" />

    <div
      v-if="launcher.state.loginError && !launcher.state.loginPending"
      class="mc-panel w-full min-w-0 max-w-md overflow-hidden border-mc-red p-4"
      role="alert"
    >
      <p class="font-pixel pixel-shadow-sm mb-2 text-sm text-mc-red">
        Sign-in failed
      </p>
      <p
        class="max-h-28 overflow-y-auto text-sm leading-relaxed break-all whitespace-pre-wrap text-mc-text"
      >
        {{ launcher.state.loginError }}
      </p>
      <div
        v-if="launcher.loginLogs.value.length > 0"
        class="mc-inset-well mt-3 max-h-32 overflow-y-auto p-2 font-mono text-[11px] leading-5 select-text"
      >
        <p
          v-for="(line, i) in launcher.loginLogs.value"
          :key="i"
          class="break-all whitespace-pre-wrap"
          :class="{
            'text-mc-red': line.level === 'ERROR',
            'text-mc-gold': line.level === 'WARN',
            'text-mc-text': line.level === 'INFO',
          }"
        >
          <span class="text-mc-faint">[{{ line.time }}] [{{ line.source }}]</span>
          {{ line.message }}
        </p>
      </div>
    </div>

    <div
      v-if="launcher.state.accounts.length > 0 && !launcher.state.loginPending"
      class="mc-panel w-full max-w-sm p-3"
    >
      <p class="font-pixel pixel-shadow-sm mb-2 text-[10px] text-mc-muted">
        Or continue as
      </p>
      <div class="flex flex-col gap-1">
        <div
          v-for="account in launcher.state.accounts"
          :key="account.id"
          class="group flex cursor-pointer items-center gap-3 p-1.5 transition-colors duration-150 hover:bg-mc-panel-2"
          role="button"
          tabindex="0"
          @click="launcher.setActiveAccount(account.id)"
          @keydown.enter="launcher.setActiveAccount(account.id)"
        >
          <AvatarBlock
            :uuid="account.uuid"
            :username="account.username"
            :seed="account.avatarSeed"
            :size="32"
          />
          <span class="min-w-0 flex-1 truncate text-sm text-mc-text">
            {{ account.username }}
          </span>
          <span
            v-if="account.needsReauth"
            class="font-pixel text-[9px] text-mc-gold"
          >
            Reconnect
          </span>
          <button
            type="button"
            class="cursor-pointer p-1 text-mc-faint opacity-0 transition-opacity duration-150 group-hover:opacity-100 hover:text-mc-red"
            :aria-label="`Remove ${account.username}`"
            @click.stop="launcher.removeAccount(account.id)"
          >
            <PixelIcon name="trash" :size="14" />
          </button>
        </div>
      </div>
    </div>
    <p class="absolute bottom-2 left-3 font-mono text-xs text-mc-muted">
      RSLauncher 0.1.0
    </p>
    <p class="absolute right-3 bottom-2 font-mono text-xs text-mc-muted">
      Not affiliated with Mojang or Microsoft.
    </p>
  </main>
</template>
