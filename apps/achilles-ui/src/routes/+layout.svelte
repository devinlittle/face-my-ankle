<script lang="ts">
  import favicon from "#lib/assets/favicon.svg";
  import { get_ready_for_devin_grfd } from "#lib/auth.svelte.ts";
  import { connectWS } from "#lib/ws.svelte.ts";
  import { page } from "$app/state";
  import { onMount } from "svelte";

  let { children } = $props();

  /* onMount(async () => {
    await get_ready_for_devin_grfd(false);
  }); */
  onMount(connectWS);

  const links = [
    { href: "/", label: "Home" },
    //    { href: "/login", label: "Login" },
    //    { href: "/register", label: "Register" },
    { href: "/net", label: "Net" },
    { href: "/battery", label: "Battery" },
  ];
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
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
