import { API_URL } from "$app/env/public"
import { SvelteSet } from "svelte/reactivity";
import { auth } from "./auth.svelte";
import { handleNetworkStatistics } from "./net.svelte";
import type { components } from "./types/api";
import { handleBatteryInfo, handleCritical } from "./battery.svelte";

type IncomingMessage = {
  topic: components["schemas"]["SubscriptionTopic"]
  payload: unknown
}

type SubscriptionEvent = components["schemas"]["SubscriptionEvent"];
type SubscriptionTopic = components["schemas"]["SubscriptionTopic"];
type NetStatistics = components["schemas"]["NetStatistics"];
type BatteryInfo = components["schemas"]["BatteryInfo"]

let _socketState = $state<"connected" | "connecting" | "disconnected" | "intial_page">("intial_page");

export const socketState = {
  get value() { return _socketState },
  set value(v) { _socketState = v }
};

function updateSocketState(newStatus: "connected" | "connecting" | "disconnected") {
  _socketState = newStatus;

  switch (newStatus) {
    case "disconnected":
      console.error("disconnected");
      break;
    case "connecting":
      console.log("connecting");
      break;
    case "connected":
      console.log("connected to websocket");
      break;
  }
}

function handleMessage(msg: IncomingMessage) {
  switch (msg.topic) {
    case "wifi_scan":
    case "device_status":
    case "network_metrics": {
      handleNetworkStatistics(msg.payload as NetStatistics)
    }
    case "battery_info": {
      handleBatteryInfo(msg.payload as BatteryInfo)
    }
    case "critical_information": {
      handleCritical(msg.payload as boolean)
    }
  }
}

let socket: WebSocket | null = null

export function connectWS() {
  if (socket) {
    socket.onopen = null;
    socket.onmessage = null;
    socket.onerror = null;
    socket.onclose = null;

    if (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING) {
      socket.close();
    }
    socket = null;
  }

  const url = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/ws`

  const ws = new WebSocket(url)
  socket = ws;

  updateSocketState("connecting");

  //TODO: removed bc of auth issues and what not
  //
  /* const bootstrap_json = JSON.stringify({
      token: auth.accessToken,
    }) */

  ws.onopen = () => {
    if (ws !== socket) return;

    //socket!.send(bootstrap_json)
    updateSocketState("connected");
  }

  ws.onmessage = (e: MessageEvent) => {
    if (ws !== socket) return;

    try {
      const msg = JSON.parse(e.data) as IncomingMessage
      if (!msg.topic) return // drop anything without a namespace
      handleMessage(msg)
    } catch {
      // not json, ignore
    }
  }

  ws.onclose = () => {
    if (ws !== socket) return;
    socket = null

    updateSocketState("disconnected");

  }

  ws.onerror = () => { connectWS(); }
}

export function disconnectWS() {
  if (socket) {
    socket.onopen = null;
    socket.onmessage = null;
    socket.onerror = null;
    socket.onclose = null;

    if (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING) {
      socket.close();
    }
    socket = null;
  }
}

export function getSocket() {
  return socket;
}

export let subscribed_topics = new SvelteSet<SubscriptionTopic>();

export function subscribe(topics: SubscriptionTopic[]) {

  for (const topic of topics) {
    subscribed_topics.add(topic)
  }

  let sub: SubscriptionEvent = {
    action: "subscribe",
    topics
  }
  let sub_json = JSON.stringify(sub);

  getSocket()?.send(sub_json);
}

export function unsubscribe(topics: SubscriptionTopic[]) {

  for (const topic of topics) {
    subscribed_topics.delete(topic)
  }

  let sub: SubscriptionEvent = {
    action: "unsubscribe",
    topics
  }
  let sub_json = JSON.stringify(sub);

  getSocket()?.send(sub_json);
}
