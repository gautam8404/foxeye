<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageServerData } from '../../../.svelte-kit/types/src/routes/$types';
	import SearchResultList from '$lib/components/SearchResultList.svelte';
	import { onMount } from 'svelte';

	export let data: PageServerData;

	onMount(() => {
		query = new URLSearchParams(location.search).get('q');
	})

	let query:string|null;

	function handleSearch() {
		if (query != null && query != '') {
			let value = query.replace(/\s/g, '+');
			goto(`/search?q=${value}&p=0`)
		}
	}
</script>


<div class="min-h-screen  ">
	<!-- Search Navigation Bar -->
	<form on:submit|preventDefault={handleSearch} class="bg-white drop-shadow-md">
		<div class="max-w-9xl mx-auto px-4 py-6 sm:px-6 lg:px-8 flex items-center justify-between">
			<a href="/">
			<h1 class="text-2xl font-bold text-gray-600 -translate-y-1 hidden md:inline"><span class="text-orange-400">Fox</span>eye</h1>
			</a>
			<div class="w-full max-w-2xl">
				<input
					type="text"
					class="w-full px-4 py-1 border border-orange-300 rounded-full shadow-sm focus:outline-none focus:ring focus:ring-orange-200"
					placeholder="Search Foxeye or type a URL"
					bind:value={query}
				/>
			</div>
			<button
				class="ml-4 px-4 py-1 bg-orange-600 text-white rounded-full shadow hover:bg-orange-700"
				on:click={handleSearch}
			>
				Search
			</button>
		</div>
	</form>

	<!-- Search Results List -->
	<div class="max-w-7xl pl-3">
		{#if data}
			<SearchResultList results={data.data} />
		{/if}
	</div>
</div>