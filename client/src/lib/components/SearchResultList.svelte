<script lang="ts">
	import SearchResult from './SearchResult.svelte';
	import type { searchRes } from '$lib/types';


	export let results: searchRes[] = [];
</script>

<div class="mt-6 bg-white shadow-sm rounded-lg">
	{#if results.length > 0}
		{#each results as result}
			<SearchResult title={result.title} url={result.url} description={result.description} />
		{/each}
	{:else}
		<div class="p-4 text-gray-600">No results found.</div>
	{/if}
</div>