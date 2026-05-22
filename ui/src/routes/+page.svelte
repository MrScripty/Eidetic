<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import SplashScreen from '$lib/components/layout/SplashScreen.svelte';
  import ToastContainer from '$lib/components/layout/ToastContainer.svelte';
  import { createServerEventClient } from '$lib/serverEventClient.js';
  import { setupServerEventHandlers } from '$lib/stores/serverEventHandlers.js';

  onMount(() => {
    const events = createServerEventClient();
    events.connect();
    const teardownHandlers = setupServerEventHandlers(events);
    return () => {
      teardownHandlers();
      events.disconnect();
    };
  });
</script>

<svelte:head>
  <title>Eidetic</title>
</svelte:head>

<AppShell />
<SplashScreen />
<ToastContainer />
