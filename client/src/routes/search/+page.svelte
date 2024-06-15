<script lang="ts">
	import { goto } from '$app/navigation';
	import type { PageServerData } from '../../../.svelte-kit/types/src/routes/$types';
	import SearchResultList from '$lib/components/SearchResultList.svelte';

	export let data: PageServerData;

	let query:string|null;

	function handleSearch() {
		if (query != null && query != '') {
			let value = query.replace(/\s/g, '+');
			goto(`/search?q=${value}&p=0`)
		}
	}
</script>


<div class="min-h-screen bg-gray-100">
	<!-- Search Navigation Bar -->
	<div class="bg-white shadow-md">
		<div class="max-w-7xl mx-auto px-4 py-6 sm:px-6 lg:px-8 flex items-center justify-between">
			<h1 class="text-2xl font-bold text-blue-600">Foxeye</h1>
			<div class="w-full max-w-xl">
				<input
					type="text"
					class="w-full px-4 py-2 border border-gray-300 rounded-full shadow-sm focus:outline-none focus:ring focus:ring-blue-200"
					placeholder="Search Foxeye or type a URL"
					bind:value={query}
				/>
			</div>
			<button
				class="ml-4 px-4 py-2 bg-blue-600 text-white rounded-full shadow hover:bg-blue-700"
				on:click={handleSearch}
			>
				Search
			</button>
		</div>
	</div>

	<!-- Search Results List -->
	<div class="max-w-7xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
		{#if data}
			<SearchResultList results={data.data} />
<!--			<pre>-->
<!--				{JSON.stringify(data)}-->
<!--			</pre>-->
		{/if}
	</div>
</div>