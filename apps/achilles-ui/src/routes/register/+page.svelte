<script lang="ts">
  import { authApi } from "#lib/auth.svelte.ts";
  import { goto } from "$app/navigation";

  let username = $state<string | undefined>();
  let password = $state<string | undefined>();
  let loginError = $state<string | undefined>();

  async function handleRegister() {
    loginError = "";
    if (!username || !password) {
      loginError = "fill in all fields";
      return;
    }

    let { response } = await authApi.POST("/auth/register", {
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
          loginError = "Not Found";
          break;

        case 409:
          console.log("User Exists");
          loginError = "User already exists";
          break;

        case 500:
          console.log("Internal server error");
          break;

        default:
      }

      if (response.ok) {
        goto("/login");
      }
    }
  }
</script>

<input type="text" placeholder="username" bind:value={username} />
<input
  type="password"
  placeholder="password"
  bind:value={password}
  onkeydown={(e) => e.key === "Enter" && handleRegister}
/>
{#if loginError}<p class="error">{loginError}</p>{/if}
<button onclick={handleRegister}>Sign in</button>

<p>{loginError}</p>
