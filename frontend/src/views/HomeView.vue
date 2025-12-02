<template>
	<div class="mx-auto max-w-4xl p-8">
		<div v-if="appStore.isConfigured">
			<p class="mb-8 text-gray-400">Find similar beatmaps based on mapping style.</p>
			<div class="mb-4 flex items-start gap-3">
				<UiInput v-model="mapID" placeholder="Enter beatmap ID or URL to search..." @keyup.enter="search" />
				<UiButton @click="search" :disabled="isLoading" class="!px-4 py-3">
					<SpinnerIcon v-if="isLoading" />
					<SearchIcon v-else />
				</UiButton>
			</div>

			<div v-if="error" class="rounded-md bg-red-900/50 p-4 text-red-400">{{ error }}</div>

			<div v-if="results.length > 0" class="mt-8 space-y-3">
				<div v-for="item in results" :key="item.map_info.beatmap_id" class="bg-osu-light flex items-center gap-4 rounded-lg p-4">
					<BeatmapCover :src="item.cover_url" />

					<div class="flex-grow">
						<a
							:href="`https://osu.ppy.sh/beatmapsets/${item.map_info.beatmapset_id}#osu/${item.map_info.beatmap_id}`"
							target="_blank"
							class="transition-colors hover:text-cyan-400"
						>
							<p class="text-lg font-bold">{{ item.map_info.artist }} - {{ item.map_info.title }}</p>
							<p class="text-md text-gray-300">[{{ item.map_info.difficulty_name }}]</p>
						</a>

						<div class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-2">
							<p class="text-sm text-gray-400">
								Score: {{ item.score.toFixed(4) }} | Divergence:
								<span :class="getDivergenceInfo(item.divergence).colorClass" class="font-semibold">
									{{ item.divergence.toFixed(2) }} ({{ getDivergenceInfo(item.divergence).label }})
								</span>
							</p>
							<a
								:href="`osu://b/${item.map_info.beatmap_id}`"
								class="bg-osu-dark rounded-full px-2 py-1 text-xs text-gray-300 transition-all hover:scale-102 hover:bg-gray-900 active:scale-98"
								title="Open in osu! (supporter only)"
							>
								Open in osu! (supporter only)
							</a>
						</div>
					</div>
				</div>
			</div>
		</div>

		<div v-else class="flex h-[60vh] flex-col items-center justify-center text-center">
			<h2 class="text-2xl font-bold text-yellow-400">Welcome!</h2>
			<p class="mt-4 max-w-md text-gray-300">
				To get started, please go to the settings page to select your osu! songs folder and run the indexer.
			</p>
			<RouterLink
				to="/settings"
				class="bg-osu-pink mt-8 inline-block rounded-md px-8 py-3 font-bold text-white transition-transform hover:scale-105"
			>
				Go to Settings
			</RouterLink>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/core'
import UiButton from '@/components/ui/UiButton.vue'
import UiInput from '@/components/ui/UiInput.vue'
import SearchIcon from '@/components/icons/SearchIcon.vue'
import SpinnerIcon from '@/components/icons/SpinnerIcon.vue'
import BeatmapCover from '@/components/BeatmapCover.vue'

interface MapInfo {
	path: string
	beatmap_id: number
	beatmapset_id: number
	title: string
	artist: string
	difficulty_name: string
}

interface SearchResult {
	score: number
	divergence: number
	map_info: MapInfo
	cover_url: string
}

const mapID = ref('')
const results = ref<SearchResult[]>([])
const isLoading = ref(false)
const error = ref<string | null>(null)
const appStore = useAppStore()

onMounted(async () => {
	appStore.checkConfiguration()
})

const search = async () => {
	if (!mapID.value) return

	let finalID = mapID.value
	// https://osu.ppy.sh/beatmapsets/SET_ID#osu/BEATMAP_ID
	// https://osu.ppy.sh/beatmaps/BEATMAP_ID
	// https://osu.ppy.sh/b/BEATMAP_ID
	const match = mapID.value.match(/osu\.ppy\.sh\/(?:beatmapsets\/\d+#osu|beatmaps|b)\/(\d+)/)

	if (match && match[1]) finalID = match[1]

	isLoading.value = true
	error.value = null
	results.value = []

	try {
		results.value = await invoke('search', { beatmapId: Number(finalID) })
	} catch (e: any) {
		console.error(e)
		if (typeof e === 'object' && e !== null && e.type) {
			switch (e.type) {
				case 'DatabaseNotIndexed':
					error.value = 'Database is not indexed. Please go to Settings to start indexing.'
					break
				case 'MapNotFound':
					error.value = `Beatmap with ID ${e.payload} was not found in your local library.`
					break
				default:
					error.value = `An unknown error occurred: ${e.type}`
			}
		} else {
			error.value = typeof e === 'string' ? e : 'An unexpected error occurred.'
		}
	} finally {
		isLoading.value = false
	}
}

const getDivergenceInfo = (divergence: number): { label: string; colorClass: string } => {
	if (divergence <= 0.25) {
		return { label: 'Close', colorClass: 'text-green-400' }
	} else if (divergence <= 1.0) {
		return { label: 'Similar', colorClass: 'text-yellow-400' }
	} else if (divergence <= 2.0) {
		return { label: 'Related', colorClass: 'text-orange-400' }
	} else {
		return { label: 'Distant', colorClass: 'text-red-400' }
	}
}
</script>
