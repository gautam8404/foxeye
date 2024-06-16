<script lang="ts">
	import SearchResult from './SearchResult.svelte';
	import type { searchRes } from '$lib/component_types';

	const maxSummary = 150;

	export let results: searchRes[] = [];
	for (let i = 0; i < results.length; i++) {
		results[i].url = results[i].url.replace(/\/$/, '');
		results[i].summary = results[i].summary.split(" ").slice(0, maxSummary).join(" ");
		if (results[i].title === '') {
			results[i].title = results[i].url;
		}
	}
</script>

<div class="mt-6 bg-white shadow-md rounded-lg">
	{#if results.length > 0}
		{#each results as result}
			<SearchResult title={result.title} url={result.url} description={result.summary} />
		{/each}
	{:else}
		<div class="p-4 text-orange-600">No results found.</div>
	{/if}
</div>
