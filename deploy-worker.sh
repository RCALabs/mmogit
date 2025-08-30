#!/bin/bash
# Deploy Cloudflare Worker for visitor book API proxy

echo "Creating Cloudflare Worker for visitor book API..."

# Create wrangler.toml
cat > wrangler.toml << 'EOF'
name = "mmogit-visitor-book-api"
main = "cloudflare-worker.js"
compatibility_date = "2024-01-01"

[env.production]
route = { pattern = "visitor-book-api.mmogit.workers.dev/*", zone_id = "" }
EOF

echo "Deploy with: npx wrangler deploy"
echo "Or paste the cloudflare-worker.js content into Cloudflare dashboard"