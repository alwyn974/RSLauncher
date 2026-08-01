<script setup lang="ts">
/**
 * Authored pixel icons on a 16x16 grid. One fill color (currentColor),
 * square rects only, crisp edges — the launcher's icon system.
 */
import { computed } from "vue";

export type IconName =
  | "gear"
  | "terminal"
  | "back"
  | "x"
  | "plus"
  | "copy"
  | "trash"
  | "check";

const props = withDefaults(defineProps<{ name: IconName; size?: number }>(), {
  size: 16,
});

type Rect = [number, number, number, number]; // x, y, w, h

const ICONS: Record<IconName, Rect[]> = {
  gear: [
    [6, 1, 4, 2], [6, 13, 4, 2], [1, 6, 2, 4], [13, 6, 2, 4],
    [3, 3, 2, 2], [11, 3, 2, 2], [3, 11, 2, 2], [11, 11, 2, 2],
    [4, 4, 8, 8],
  ],
  terminal: [
    [2, 2, 12, 2], [2, 12, 12, 2], [2, 2, 2, 12], [12, 2, 2, 12],
    [5, 6, 1, 1], [6, 7, 1, 1], [7, 8, 1, 1], [6, 9, 1, 1], [5, 10, 1, 1],
    [9, 10, 4, 1],
  ],
  back: [
    [5, 7, 9, 2],
    [4, 6, 1, 1], [3, 7, 1, 1], [4, 8, 1, 1],
    [5, 5, 1, 1], [5, 9, 1, 1], [6, 4, 1, 1], [6, 10, 1, 1],
  ],
  x: [
    [4, 4, 2, 2], [6, 6, 2, 2], [8, 8, 2, 2], [10, 10, 2, 2],
    [10, 4, 2, 2], [8, 6, 2, 2], [6, 8, 2, 2], [4, 10, 2, 2],
  ],
  plus: [
    [7, 3, 2, 10], [3, 7, 10, 2],
  ],
  copy: [
    [6, 2, 8, 2], [6, 4, 2, 6], [12, 4, 2, 8], [6, 12, 8, 2],
    [2, 6, 2, 8], [2, 14, 8, 1], [9, 13, 1, 1], [4, 6, 2, 2],
  ],
  trash: [
    [6, 2, 4, 1], [4, 3, 8, 1],
    [5, 4, 1, 9], [10, 4, 1, 9], [5, 13, 6, 1],
    [7, 6, 1, 5], [9, 6, 1, 5],
  ],
  check: [
    [2, 8, 2, 2], [4, 10, 2, 2], [6, 8, 2, 2], [8, 6, 2, 2], [10, 4, 2, 2],
  ],
};

// gear reads better with a punched center hole
const HOLES: Partial<Record<IconName, Rect[]>> = {
  gear: [[6, 6, 4, 4]],
};

const rects = computed(() => ICONS[props.name]);
const holes = computed(() => HOLES[props.name] ?? []);
const maskId = computed(() => `icon-hole-${props.name}`);
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 16 16"
    shape-rendering="crispEdges"
    fill="currentColor"
    aria-hidden="true"
    class="shrink-0"
  >
    <template v-if="holes.length">
      <mask :id="maskId">
        <rect width="16" height="16" fill="black" />
        <rect
          v-for="(r, i) in rects"
          :key="`r${i}`"
          :x="r[0]"
          :y="r[1]"
          :width="r[2]"
          :height="r[3]"
          fill="white"
        />
        <rect
          v-for="(r, i) in holes"
          :key="`h${i}`"
          :x="r[0]"
          :y="r[1]"
          :width="r[2]"
          :height="r[3]"
          fill="black"
        />
      </mask>
      <rect width="16" height="16" :mask="`url(#${maskId})`" />
    </template>
    <template v-else>
      <rect
        v-for="(r, i) in rects"
        :key="i"
        :x="r[0]"
        :y="r[1]"
        :width="r[2]"
        :height="r[3]"
      />
    </template>
  </svg>
</template>
