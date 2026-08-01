<script setup lang="ts">
/**
 * Minecraft-style beveled button: flat field, black 2px border, inset
 * highlight top-left / shadow bottom-right. Square corners, pixel label.
 */
import { computed } from "vue";

export type ButtonVariant = "stone" | "green" | "gold" | "red";

const props = withDefaults(
  defineProps<{
    variant?: ButtonVariant;
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  { variant: "stone", disabled: false, type: "button" },
);

const VARIANTS: Record<ButtonVariant, string> = {
  stone: "bg-mc-stone hover:bg-mc-stone-hi text-white",
  green: "bg-mc-green hover:bg-mc-green-hi text-white",
  gold: "bg-[#a8862a] hover:bg-[#c29c33] text-white",
  red: "bg-mc-red-lo hover:bg-[#c13f43] text-white",
};

const classes = computed(() => [
  "mc-bevel font-pixel pixel-shadow-sm inline-flex items-center justify-center gap-2",
  "px-4 py-2 text-sm tracking-wide transition-colors duration-150 select-none",
  "focus-visible:outline-2 focus-visible:outline-mc-gold focus-visible:outline-offset-2",
  props.disabled
    ? "bg-mc-stone-lo text-mc-faint cursor-not-allowed"
    : `${VARIANTS[props.variant]} cursor-pointer`,
]);
</script>

<template>
  <button :type="type" :disabled="disabled" :class="classes">
    <slot />
  </button>
</template>

<style scoped>
button:not(:disabled):active {
  box-shadow:
    inset -2px -2px 0 rgba(255, 255, 255, 0.14),
    inset 2px 2px 0 rgba(0, 0, 0, 0.5);
}
</style>
