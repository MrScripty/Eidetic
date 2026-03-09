<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import SplashScreen from '$lib/components/layout/SplashScreen.svelte';
  import ToastContainer from '$lib/components/layout/ToastContainer.svelte';
  import { WsClient } from '$lib/ws.js';
  import { setupWsHandlers } from '$lib/stores/wsHandlers.js';

  const ws = new WsClient();

  onMount(() => {
    ws.connect();
    const teardownHandlers = setupWsHandlers(ws);
    return () => {
      teardownHandlers();
      ws.disconnect();
    };
  });
</script>

<svelte:head>
  <title>Eidetic</title>
</svelte:head>

<AppShell />
<SplashScreen />
<ToastContainer />
