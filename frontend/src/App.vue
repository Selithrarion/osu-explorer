<template>
	<div class="bg-osu-dark flex min-h-screen flex-col font-sans text-white">
		<header class="bg-osu-header/10 sticky top-0 z-10 backdrop-blur-sm">
			<nav class="mx-auto flex max-w-7xl items-center justify-between p-6 lg:px-8" aria-label="Global">
				<div class="flex lg:flex-1">
					<RouterLink to="/" class="text-osu-pink -m-1.5 p-1.5 text-2xl font-bold"> osu!explorer </RouterLink>
				</div>
				<div class="flex gap-12">
					<RouterLink to="/" class="text-sm leading-6 font-semibold text-gray-300 hover:text-white">Search</RouterLink>
					<RouterLink to="/settings" class="text-sm leading-6 font-semibold text-gray-300 hover:text-white">
						Settings & Indexing
					</RouterLink>
				</div>
			</nav>
		</header>

		<main class="flex-grow">
			<RouterView />
		</main>

		<footer class="p-4 text-center text-xs text-gray-500">
			<a
				href="https://github.com/Selithrarion/osu-explorer"
				target="_blank"
				class="inline-flex items-center gap-2 transition-colors hover:text-white"
			>
				<GitHubIcon />
				<span>osu!explorer by selithrarion</span>
			</a>
		</footer>
	</div>
</template>

<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import GitHubIcon from './components/icons/GitHubIcon.vue'
</script>
