<template>
	<div class="mx-auto max-w-2xl p-8">
		<h1 class="text-osu-pink mb-4 text-4xl font-bold">Settings</h1>
		<div class="max-w-xl space-y-6">
			<UiInput
				v-model="songsPath"
				id="songs-path"
				label="osu! Songs Path"
				placeholder="e.g., C:\Users\YourUser\AppData\Local\osu!\Songs"
				hint="The path to your osu! songs folder. This will be saved for the next time."
			>
				<template #append>
					<UiButton @click="openFileDialog" variant="secondary" class="!p-3 !px-3">
						<FolderIcon />
					</UiButton>
				</template>
			</UiInput>

			<UiInput
				v-model.number="limit"
				type="number"
				id="limit"
				label="Indexing Limit (Optional)"
				placeholder="Leave empty to index all maps"
			/>

			<UiButton @click="startIndexing" :disabled="appStore.isIndexing || !songsPath">
				{{ appStore.isIndexing ? 'Indexing in progress...' : 'Start Indexing' }}
			</UiButton>

			<div v-if="appStore.indexingStatus" class="mt-4 rounded-md p-4" :class="appStore.statusColor">
				<p>{{ appStore.indexingStatus }}</p>
				<div v-if="appStore.isIndexing && appStore.indexingProgress" class="mt-2 flex items-center gap-3">
					<div class="w-full rounded-full bg-gray-700">
						<div
							class="bg-osu-pink rounded-full p-0.5 text-center text-xs leading-none font-medium text-white"
							:style="{ width: `${(appStore.indexingProgress.progress / appStore.indexingProgress.total) * 100}%` }"
						>
							&nbsp;
						</div>
					</div>
					<span class="text-xs whitespace-nowrap text-gray-400">
						{{ appStore.indexingProgress.progress }} / {{ appStore.indexingProgress.total }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import UiButton from '@/components/ui/UiButton.vue'
import UiInput from '@/components/ui/UiInput.vue'
import FolderIcon from '@/components/icons/FolderIcon.vue'

interface AppConfig {
	songs_path: string | null
}

const songsPath = ref('')
const limit = ref<number | null>(null)
const appStore = useAppStore()

onMounted(async () => {
	const savedPath = localStorage.getItem('osuSongsPath')
	if (savedPath) {
		songsPath.value = savedPath
	} else {
		const config = await invoke<AppConfig>('get_config')
		if (config.songs_path) songsPath.value = config.songs_path
	}
})

const startIndexing = async () => {
	if (!songsPath.value) return
	localStorage.setItem('osuSongsPath', songsPath.value)
	await appStore.startIndexing(songsPath.value, limit.value)
}

const openFileDialog = async () => {
	const selected = await open({
		directory: true,
		multiple: false,
	})
	if (typeof selected === 'string') songsPath.value = selected
}
</script>
