<script lang="ts">

	import { goto } from '$app/navigation';

	let input:string|null;

	function handleSubmit() {
		if (input != null && input !== '') {
			let value = input.replace(/\s/g, '+');
			goto(`/search?q=${value}&p=0`)
		}
	}

</script>



<div class="flex flex-col items-center justify-center min-h-screen bg-gray-100">
	<div class="text-center">
		<h1 class="text-6xl font-bold text-blue-600">Foxeye</h1>
	</div>
		<div class="mt-10 w-full max-w-md">
			<input
				type="text"
				class="w-full px-6 py-2 border border-gray-300 rounded-full shadow-sm focus:outline-none focus:ring focus:ring-blue-200"
				placeholder="Search Foxeye"
				required
				bind:value={input}
			/>
		</div>
		<div class="mt-8 flex space-x-4">
			<button class="px-8 py-2 bg-blue-600 text-white rounded-full shadow hover:bg-blue-700" on:click={handleSubmit}>
				Search
			</button>
		</div>
</div>
