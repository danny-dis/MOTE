# MOTE landing page

Live: https://mote-runtime.vercel.app

A dependency-free static public introduction to MOTE. HTML/CSS and a small script
for accessible code tabs and copying. No account forms, analytics, model calls,
serverless functions, or runtime secrets. Vercel may retain normal hosting logs.

## Local preview

From the repository root:

```sh
python -m http.server 8893 --bind 127.0.0.1 --directory site/public
```

Open http://127.0.0.1:8893. The code panels are integration examples, not a hosted
agent demo. The page links to GitHub; source/download access follows repository
visibility. The MOTE repository is public. Only the landing-page assets are served
by the site; repository documents and build output are not part of its payload.

## Deployment

The Vercel project is `mote-runtime` in `kallmedis-projects`. Its repository root
setting is `site`, framework is Other, and output is `public`. Git integration is
not connected yet: Vercel rejected the repository connection. CLI deployment works.
If Git integration is authorized later, use `production-ready` as its production branch.
Deploy from the repository root after linking the project:

```sh
vercel deploy --prod --scope kallmedis-projects
```

Only `site/public` is served. The repository-root `.vercelignore` allows only that
directory and `site/vercel.json`; check `vercel deploy --dry --json` before publishing.
Keep configuration, private documents, archives, credentials, and build caches
outside `public`. The CSP blocks remote scripts, frames, and browser API calls.

Before publishing a change, check mobile/desktop overflow, keyboard navigation,
tab switching, copy behavior, FAQ expansion, links, security headers, social image,
and unauthenticated HTTP access to the production URL. Check the version label
when publishing a new MOTE release. This site is independent of binary releases.
