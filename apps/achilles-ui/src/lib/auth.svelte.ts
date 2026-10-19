import { API_URL } from "$app/env/public"

type Auth = {
  id: string | null;
  session_id: string | null;
  username: string | null;
  accessToken: string | null;
  ready: boolean;
};

export let auth = $state<Auth>({
  id: null,
  session_id: null,
  username: null,
  accessToken: null,
  ready: false,
});

async function setToken(token: string) {
  auth.accessToken = token;
  auth.id = decode(token)?.sub ?? null;
  auth.username = decode(token)?.username ?? null;
  auth.session_id = decode(token)?.session_id ?? null;

  auth.ready = true;
}

function clear() {
  auth.accessToken = null;
  auth.username = null;
  auth.id = null;
  auth.ready = false;
  localStorage.removeItem("access_token");
}

import createClient from 'openapi-fetch';
import type { paths } from '#lib/types/api.ts';
import { goto } from "$app/navigation";
import { connectWS, disconnectWS } from "./ws.svelte";

const authFetch = async (
  input: RequestInfo | URL,
  init?: RequestInit,
) => {
  const original = new Request(input, init);

  const withAuth = (request: Request) => {
    const headers = new Headers(request.headers);

    if (auth.accessToken) {
      headers.set('Authorization', `Bearer ${auth.accessToken}`);
    }

    return new Request(request, { headers });
  };

  const response = await fetch(withAuth(original.clone()));

  if (response.status !== 401 && response.status !== 403) {
    return response;
  }

  const refreshed = await getRefreshStatus();

  if (!refreshed) {
    return response;
  }

  await getRefreshStatus();
  return fetch(withAuth(original.clone()));
};

export function createApiClient() {
  const client = createClient<paths>({
    baseUrl: API_URL,
    credentials: 'include',
    fetch: authFetch
  });

  return client;
}

export const api = createApiClient();

function decode(token: String) {
  try {
    return JSON.parse(atob(token.split(".")[1]));
  } catch {
    return null;
  }
}

let refreshPromise: Promise<boolean> | null = null;

export async function getRefreshStatus() {
  if (refreshPromise) {
    return refreshPromise;
  }

  refreshPromise = refresh();

  try {
    return await refreshPromise;
  } finally {
    refreshPromise = null;
  }
}

export const authApi = createClient<paths>({
  baseUrl: API_URL,
  credentials: 'include',
  fetch: authFetch,
});


export async function initAuth() {
  if (localStorage.getItem("access_token") === null) { clear(); return false; }

  let data = (await authApi.POST("/auth/refresh"))

  if (data.response.ok) {
    const access_token = data.data?.access_token;

    if (!access_token) {
      throw new Error('Refresh response did not contain an access token');
    }

    await setToken(access_token);
    return true;
  }
}

async function refresh() {
  if (localStorage.getItem("access_token") === null) { clear(); return false; }

  const res = await fetch(`${API_URL}/auth/refresh`, {
    method: "POST",
    credentials: "include",
  });
  if (!res.ok) { clear(); return false; }
  const { access_token } = await res.json();
  await setToken(access_token);
  return true;
}


export async function logout() {
  await api.GET("/auth/logout");
  clear();
  goto("/");
  disconnectWS();
}

export async function onAuthSuccess(token: string) {
  await setToken(token);
}

export async function get_ready_for_devin_grfd(from_login: boolean): Promise<boolean> {
  if (!from_login) {
    const ok = await initAuth();
    if (!ok) {
      disconnectWS();
      return false;
    }
  }

  console.log("begining notification connection")
  connectWS();
  return true;
}

