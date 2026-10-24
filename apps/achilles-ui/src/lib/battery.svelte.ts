import type { components } from "./types/api";
type BatteryInfo = components["schemas"]["BatteryInfo"]

let _battery_info = $state<BatteryInfo>();

export const battery_info = {
  get value() { return _battery_info },
  set value(v: BatteryInfo | undefined) { _battery_info = v }
};

let _is_critical = $state<boolean>(false);

export const is_critical = {
  get value() { return _is_critical },
  set value(v: boolean) { _is_critical = v }
};

export function handleBatteryInfo(payload: BatteryInfo) {
  battery_info.value = payload
}

export function handleCritical(payload: boolean) {
  is_critical.value = payload
}

