import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface AppConfig {
	songs_path: string | null
}

export interface IndexingProgress {
	progress: number
	total: number
	message: string
}

export const useAppStore = defineStore('app', () => {
	const isConfigured = ref(true)
	const isIndexing = ref(false)
	const indexingStatus = ref('')
	const statusIsError = ref(false)
	const indexingProgress = ref<IndexingProgress | null>(null)
	let pollingInterval: number | null = null

	const statusColor = computed(() => {
		return statusIsError.value ? 'bg-red-900/50 text-red-400' : 'bg-osu-pink/20 text-osu-pink'
	})

	async function checkConfiguration() {
		try {
			const [config, dbIndexed] = await Promise.all([invoke<AppConfig>('get_config'), invoke<boolean>('is_db_indexed')])
			isConfigured.value = !!config.songs_path && dbIndexed
		} catch (e) {
			console.error('Failed to get config from backend:', e)
			isConfigured.value = false
		}
	}

	async function startIndexing(songs_path: string, limit: number | null) {
		isIndexing.value = true
		indexingStatus.value = 'Initializing indexing...'
		statusIsError.value = false
		indexingProgress.value = null

		pollingInterval = setInterval(async () => {
			try {
				indexingProgress.value = await invoke('get_indexing_status')
			} catch (e) {
				console.error('Failed to fetch indexing status', e)
			}
		}, 1000)

		try {
			await invoke('index', { songsPath: songs_path, limit: limit || null })
		} catch (e: any) {
			indexingStatus.value = `An error occurred: ${e}`
			statusIsError.value = true
			if (pollingInterval) clearInterval(pollingInterval)
			isIndexing.value = false
			indexingProgress.value = null
		}
	}

	listen<boolean>('state-reloaded', (event) => {
		if (pollingInterval) clearInterval(pollingInterval)
		pollingInterval = null

		const wasSuccessful = event.payload
		if (wasSuccessful) {
			indexingStatus.value = 'Indexing completed successfully! You can now use the search.'
			statusIsError.value = false
			isConfigured.value = true
		} else {
			indexingStatus.value = 'Indexing finished, but failed to load the database.'
			statusIsError.value = true
			isConfigured.value = false
		}

		isIndexing.value = false
		indexingProgress.value = null
	})

	return { isConfigured, isIndexing, indexingStatus, statusIsError, indexingProgress, statusColor, checkConfiguration, startIndexing }
})
