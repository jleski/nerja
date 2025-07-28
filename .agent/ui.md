Generally use `bun` for package management, not `npm` or `yarn`.

Prefer SvelteKit with TypeScript and TailwindCSS, they are usually already scaffolded for you in the project. Prefer Catppuccin or Rose Pine darker themes, see @.agent/ui-themes.md.

Prefer reusable components, clean and secure code, best practices and design patterns for Frontend development.

Modern seamless User Experience (UX) is a priority.

Prefer SPA architecture.

Authentication in the UI should use Auth0 for SPA applications with refresh tokens so that the user does not need to always re-login. Session cookies should be stored securely without namespace-collisions in the browser for other Auth0-based apps.

Note that the UI might already be running. In this case, skip running the UI and proceed to test it with for example:
`curl http://localhost:5173`. If the UI appears to be running but not responsive, ask the user which address the server is listening on.
