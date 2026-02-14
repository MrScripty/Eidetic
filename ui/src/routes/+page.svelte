<script lang="ts">
	import { onMount } from 'svelte';
	import AppShell from '$lib/components/layout/AppShell.svelte';
	import ToastContainer from '$lib/components/layout/ToastContainer.svelte';
	import { WsClient } from '$lib/ws.js';
	import { setupWsHandlers } from '$lib/stores/wsHandlers.js';

	const ws = new WsClient();

	onMount(() => {
		ws.connect();
		setupWsHandlers(ws);
		return () => ws.disconnect();
	});
</script>

<svelte:head>
	<title>Eidetic</title>
</svelte:head>

<AppShell />
<ToastContainer />
