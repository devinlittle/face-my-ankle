import { defineEnvVars } from '@sveltejs/kit/env';

export const variables = defineEnvVars({
  API_URL: {
    public: true,
    schema: (value) => value ?? 'http://api.config.ankle:3000'
  }
});

