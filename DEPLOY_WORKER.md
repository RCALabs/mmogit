# Deploy Cloudflare Worker for Visitor Book

## Quick Deploy via Dashboard

1. Go to: https://dash.cloudflare.com/
2. Click "Workers & Pages" in the left sidebar
3. Click "Create" → "Create Worker"
4. Name it: `mmogit-visitor-book-api`
5. Click "Deploy" to create the worker first
6. Click "Edit Code"
7. Replace ALL the code with:

```javascript
export default {
  async fetch(request, env, ctx) {
    // Handle preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, {
        headers: {
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
          'Access-Control-Allow-Headers': 'Content-Type',
          'Access-Control-Max-Age': '86400',
        }
      });
    }

    // Proxy to our Hetzner server
    const url = new URL(request.url);
    url.hostname = '91.98.123.26';
    url.port = '3000';
    url.protocol = 'http:';
    
    const response = await fetch(url, {
      method: request.method,
      headers: request.headers,
      body: request.method !== 'GET' ? await request.text() : undefined,
    });

    // Add CORS headers
    const newResponse = new Response(await response.text(), {
      status: response.status,
      statusText: response.statusText,
      headers: {
        ...Object.fromEntries(response.headers),
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
        'Access-Control-Allow-Headers': 'Content-Type',
      }
    });

    return newResponse;
  },
};
```

8. Click "Deploy" in the top right
9. Copy your worker URL (will be something like: `https://mmogit-visitor-book-api.YOUR-SUBDOMAIN.workers.dev`)

## Update Frontend

Once deployed, update `index.html` line 220 with your worker URL:

```javascript
const API_URL = window.location.protocol === 'https:' 
    ? 'https://mmogit-visitor-book-api.YOUR-SUBDOMAIN.workers.dev'  // <-- Replace this
    : 'http://91.98.123.26:3000';
```

## Test It

Visit https://mmogit.sh and try signing the guest book!