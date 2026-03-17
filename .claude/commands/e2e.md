Run the Playwright end-to-end tests for rs-grid. Execute the following steps in order:

1. Build the app: `cd /c/Dev/rs-grid/examples/basic-leptos && trunk build 2>&1`
2. Run the tests: `cd /c/Dev/rs-grid/e2e && npm test 2>&1`

Report a summary: how many tests passed / failed, and show the full output of any failures. If visual regression tests fail, note that snapshots may need to be updated with `npm run update-snapshots`.
