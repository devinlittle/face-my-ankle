<script lang="ts">
  import "../app.css";
  import { API_URL } from "$app/env/public";
  import { auth } from "#lib/auth.svelte.ts";
</script>

<p>{API_URL}</p>

{#if !auth.ready}
  <p>Login to access</p>
{:else}
  <!-- else content here -->
{/if}
