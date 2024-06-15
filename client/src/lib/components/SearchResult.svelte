<script lang="ts">
	export let title: string;
	export let url: string;
	export let description: string;
</script>

<div class="p-4 border-b border-gray-200">
	<a href={url} class="text-blue-600 text-xl font-semibold hover:underline">
		{title}
	</a>
	<p class="text-green-700">{url}</p>
	<p class="mt-2 text-gray-700">{description}</p>
</div>