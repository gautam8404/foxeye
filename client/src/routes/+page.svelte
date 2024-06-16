<script lang="ts">
	import { goto } from '$app/navigation';

	let input: string | null;

	function handleSubmit(event: Event) {
		event.preventDefault();
		if (input != null && input !== '') {
			let value = input.replace(/\s/g, '+');
			goto(`/search?q=${value}&p=0`);
		}
	}
</script>

<div class="flex flex-col items-center justify-center min-h-screen bg-gradient-to-r from-gray-100 to-gray-300">
	<div class="text-center">
		<h1 class="text-6xl font-extrabold text-gray-900">
			<span class="text-orange-600">Fox</span>eye
		</h1>
	</div>
	<form on:submit={handleSubmit} class="mt-8 w-full max-w-md">
		<input
			type="text"
			class="w-full px-6 py-3 border border-orange-300 rounded-full shadow-sm focus:outline-none focus:ring-4 focus:ring-orange-200 transition duration-300 ease-in-out"
			placeholder="Search Foxeye"
			required
			bind:value={input}
		/>
		<div class="mt-6 flex justify-center">
			<button type="submit" class="px-8 py-2 bg-orange-500 text-white rounded-full shadow-lg hover:bg-orange-600 transition duration-300 ease-in-out transform hover:scale-105">
				Search
			</button>
		</div>
	</form>
</div>
