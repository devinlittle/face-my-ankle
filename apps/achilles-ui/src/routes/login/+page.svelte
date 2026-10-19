<script lang="ts">
  import {
    authApi,
    get_ready_for_devin_grfd,
    onAuthSuccess,
  } from "#lib/auth.svelte.ts";
  import { goto } from "$app/navigation";

  let username = $state<string | undefined>();
  let password = $state<string | undefined>();
  let loginError = $state<string | undefined>();

  async function handleLogin() {
    loginError = "";
    if (!username || !password) {
      loginError = "fill in all fields";
      return;
    }

    let { response, data } = await authApi.POST("/auth/login", {
      body: {
        username,
        password,
      },
    });

    if (response.status > 400) {
      switch (response.status) {
        case 401:
          console.log("Unauthorized");
          loginError = "Unauthorized";
          break;

        case 403:
          console.log("Forbidden");
          loginError = "Forbidden";
          break;

        case 404:
          console.log("Not Found");
          loginError = "Forbidden";
          break;

        case 500:
          console.log("Internal server error");
          break;

        default:
      }

      if (response.ok) {
        if (!data?.access_token) {
          loginError = "invalid credentials";
          return Error("invalid credentials");
        }

        goto("/");

        await onAuthSuccess(data.access_token);

        await get_ready_for_devin_grfd(true);
      }
    }
  }
</script>

<input type="text" placeholder="username" bind:value={username} />
<input
  type="password"
  placeholder="password"
  bind:value={password}
  onkeydown={(e) => e.key === "Enter" && handleLogin()}
/>
{#if loginError}<p class="error">{loginError}</p>{/if}
<button onclick={handleLogin}>Sign in</button>

<p>{loginError}</p>
