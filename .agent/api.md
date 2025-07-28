Generally use `bun` for package management, not `npm` or `yarn`.

Prefer HonoJS with TypeScript, they are usually already scaffolded for the project. Alternatively implement SvelteKit server side API routes.

Prefer clean and secure code, best practices and design patterns for Frontend development.

API versioning should be path based following the semver patterns:

- https://<fqdn>/v<MAJOR>.<MINOR>.<PATCH>
- Version routing is generally handled at the Gateway level using path-based routing. Routing should not be implemented to the API code unless needed.

APIs should be secure and rate-limited to prevent abuse. Implement middleware to protect all routes except health and root route.

OpenAPI schema should be generated using middleware automatically. Add suitable framework for the language in question.

CORS and JWT token validation should be used for authenticated endpoints:

- Prefer Auth0 or WorkOS, ask the user which one to use if unsure.

Note that the API might already be running. In this case, skip running the api with wrangler and proceed to test it with for example: curl http://localhost:8787
