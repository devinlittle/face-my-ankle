<script lang="ts">
  import "../app.css";
  import { API_URL } from "$app/env/public";
  import { auth } from "#lib/auth.svelte.ts";
  import { subscribe, subscribed_topics, unsubscribe } from "#lib/ws.svelte.ts";

  function subscribe_all_relevent() {
    subscribe([
      "wifi_scan",
      "device_status",
      "network_metrics",
      "battery_info",
    ]);
  }

  function usubscribe_all_relevent() {
    unsubscribe([
      "wifi_scan",
      "device_status",
      "network_metrics",
      "battery_info",
    ]);
  }
</script>

<p>API URL: {API_URL}</p>

<hr />

<h2>Topics Subscribed to:</h2>

<ul>
  {#each subscribed_topics as topic}
    <li>{topic}</li>
  {/each}
</ul>

<button onclick={subscribe_all_relevent}> Subscribe all</button>
<button onclick={usubscribe_all_relevent}> Unsubscribe all</button>

<hr />
