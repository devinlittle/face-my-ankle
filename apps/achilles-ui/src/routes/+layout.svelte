<script lang="ts">
  import { onMount } from "svelte";
  import { afterNavigate } from "$app/navigation";
  import { page } from "$app/state";

  import { connectWS } from "#lib/ws.svelte.ts";
  import { subscribe, subscribed_topics, unsubscribe } from "#lib/ws.svelte.ts";

  let { children } = $props();

  /*import { get_ready_for_devin_grfd } from "#lib/auth.svelte.ts";
   onMount(async () => {
    await get_ready_for_devin_grfd(false);
  }); */

  onMount(connectWS);

  onMount(() => {
    switch (page?.route.id) {
      case "/net":
        subscribe(["device_status", "network_metrics"]);
        break;
      case "/battery":
        subscribe(["battery_info"]);
        break;
    }
  });

  const links = [
    { href: "/", label: "Home" },
    //    { href: "/login", label: "Login" },
    //    { href: "/register", label: "Register" },
    { href: "/net", label: "Net" },
    { href: "/battery", label: "Battery" },
  ];

  afterNavigate(({ to }) => {
    if (subscribed_topics.size > 0) {
      unsubscribe([
        "wifi_scan",
        "device_status",
        "network_metrics",
        "battery_info",
      ]);
    }

    switch (to?.route.id) {
      case "/battery":
        subscribe(["battery_info"]);
        break;

      case "/net":
        subscribe(["device_status", "network_metrics"]);
        break;
    }
  });
</script>

<svelte:head>
  <link rel="icon" href="/favicon.ico" />
</svelte:head>

<header>
  <nav>
    <ul>
      {#each links as link}
        <li aria-current={page.url.pathname === link.href ? "page" : undefined}>
          <a href={link.href}>{link.label}</a>
        </li>
      {/each}
    </ul>
  </nav>
</header>

{@render children()}
