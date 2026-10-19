import type { components } from "./types/api";
type AccessPoint = components["schemas"]["AccessPoint"];
type NetStats = components["schemas"]["NetStatistics"];

let _access_points = $state<AccessPoint[]>();

export const access_points = {
  get value() { return _access_points },
  set value(v: AccessPoint[] | undefined) { _access_points = v }
};

let _net_stats = $state<NetStats>();

export const net_stats = {
  get value() { return _net_stats },
  set value(v: NetStats | undefined) { _net_stats = v }
};

export function handleNetworkStatistics(payload: NetStats) {

  console.log(`NET_STATS: ${payload}`);
  net_stats.value = payload;

}
