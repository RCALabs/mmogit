// Cloudflare Worker to proxy visitor book API requests
// Deploy this to visitor-book-api.mmogit.workers.dev

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