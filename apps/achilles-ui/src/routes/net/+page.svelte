<script lang="ts">
  import { authApi } from "#lib/auth.svelte.ts";
  import { access_points, net_stats } from "#lib/net.svelte.ts";
  import {
    socketState,
    subscribe,
    subscribed_topics,
    unsubscribe,
  } from "#lib/ws.svelte.ts";

  let wifi_password = $state<string | undefined>();

  async function connect(ssid: string, password: string | undefined) {
    await authApi.POST("/net/wifi/connect", {
      body: {
        ssid,
        password,
      },
    });
  }

  async function request_scan() {
    await authApi.POST("/net/wifi/scan");
    let { data } = await authApi.GET("/net/wifi/networks");
    access_points.value = data;
  }

  async function disconnect_net() {
    await authApi.DELETE("/net/wifi/disconnect");
  }

  let network_stats_subscribed = $state<boolean>(false);

  function subscribe_net_stats() {
    subscribe(["network_metrics"]);
    network_stats_subscribed = true;
  }

  function unsubscribe_net_stats() {
    unsubscribe(["network_metrics"]);
    network_stats_subscribed = false;
  }
</script>

<table>
  <thead>
    <tr>
      <th>SSID</th>
      <th>STRENGTH</th>
      <th>CONNECT</th>
    </tr>
  </thead>

  <tbody>
    {#each access_points.value as access_point}
      <tr>
        <td>
          <p>{access_point.ssid}</p>
        </td>
        <td>
          <p>{access_point.strength}</p>
        </td>
        <td>
          <form>
            <input type="password" bind:value={wifi_password} />
          </form>
          <button
            onclick={async () => {
              await connect(access_point.ssid, wifi_password);
            }}>CONNECT</button
          >
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<button onclick={request_scan}> SCAN </button>

<button onclick={disconnect_net}> DISCONNECT </button>

<button onclick={subscribe_net_stats}> SUBSCRIBE NET STATS </button>

<button onclick={unsubscribe_net_stats}>
  <p style="font-weight: bold;">UN</p>
  SUBSCRIBE NET STATS
</button>

<h2>Subscribed to:</h2>

<ul>
  {#each subscribed_topics as topic}
    <li>{topic}</li>
  {/each}
</ul>

<hr />

{#if network_stats_subscribed}
  <table>
    <thead>
      <tr>
        <th>RX</th>
        <th>RX BPS</th>

        <th>TX</th>
        <th>TX BPS</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>{net_stats?.value?.rx_bytes}</td>
      </tr>
      <tr>
        <td>{net_stats?.value?.rx_bps}</td>
      </tr>

      <tr>
        <td>{net_stats?.value?.tx_bytes}</td>
      </tr>
      <tr>
        <td>{net_stats?.value?.tx_bps}</td>
      </tr>
    </tbody>
  </table>
{/if}
