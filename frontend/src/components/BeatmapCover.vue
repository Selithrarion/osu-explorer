<template>
	<div class="flex-center size-24 flex-shrink-0 rounded-md" :class="hasError ? 'bg-osu-dark' : 'bg-osu-light'">
		<img v-if="!hasError" :src="src" alt="Beatmap Cover" class="size-full rounded-md object-cover" @error="handleImageError" />
		<ImageIcon v-else />
	</div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import ImageIcon from './icons/ImageIcon.vue'

defineProps<{
	src: string
}>()

const hasError = ref(false)

const handleImageError = () => {
	hasError.value = true
}
</script>
