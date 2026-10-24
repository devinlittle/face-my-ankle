import type { components } from "./types/api";

// TODO: update this type to not be so conveluted
type DeviceStatus = components["schemas"]["IncomingMessage_DeviceStatus"]["payload"]

export function handleDeviceStatus(payload: DeviceStatus) {
  device_status.value = payload
}

let _device_status = $state<DeviceStatus>({ status: "Disconnected", ip: null })

export const device_status = {
  get value() { return _device_status },
  set value(v: DeviceStatus) { _device_status = v }
};
