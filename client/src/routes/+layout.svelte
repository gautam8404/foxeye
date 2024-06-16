<script>
	import "../app.css";

	import ProgressBar from '$lib/components/ProgressBar.svelte';
</script>

<ProgressBar class="text-orange-400"/>

<slot />

<style global>
 * {
		 font-family: 'Poppins', "Roboto Thin" , "sans-serif";
 }
</style>