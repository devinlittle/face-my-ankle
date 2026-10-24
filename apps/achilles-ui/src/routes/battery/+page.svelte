<script lang="ts">
  import { subscribe, unsubscribe } from "#lib/ws.svelte.ts";
  import { battery_info, is_critical } from "#lib/battery.svelte.ts";

  let battery_info_subscribed = $state<boolean>(false);

  function subscribe_battery_info() {
    subscribe(["battery_info"]);
    battery_info_subscribed = true;
  }

  function unsubscribe_battery_info() {
    unsubscribe(["battery_info"]);
    battery_info_subscribed = false;
  }
</script>

<button onclick={subscribe_battery_info}> SUBSCRIBE BATTERY INFO </button>

<button onclick={unsubscribe_battery_info}>
  <p style="font-weight: bold;">UN</p>
  SUBSCRIBE BATTERY INFO
</button>

{#if battery_info_subscribed}
  <p>Bat info:</p>
  <p>Percent: {battery_info.value?.bad_precent}</p>
  <p>Voltage: {battery_info.value?.voltage}</p>
  <p>Current: {battery_info.value?.current_amps} mA</p>
  <p>Power: {battery_info.value?.power_watts} W</p>

  <p>critical: {is_critical.value}</p>
{/if}
