<script lang="ts">
  import { subscribed_topics } from "#lib/ws.svelte.ts";
  import { battery_info, is_critical } from "#lib/battery.svelte.ts";

  let battery_info_subscribed = $derived(subscribed_topics.has("battery_info"));
</script>

{#if battery_info_subscribed}
  <p>Bat info:</p>
  <p>Percent: {battery_info.value?.bad_precent}</p>
  <p>Voltage: {battery_info.value?.voltage}</p>
  <p>Current: {battery_info.value?.current_amps} mA</p>
  <p>Power: {battery_info.value?.power_watts} W</p>

  <p>critical: {is_critical.value}</p>
{/if}
