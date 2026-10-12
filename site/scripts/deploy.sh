#!/usr/bin/env bash
# Publish the static export to the Hetzner host behind nginx.
#
#   bash scripts/deploy.sh --setup   # once: install nginx + certbot and the site config
#   bash scripts/deploy.sh           # build and rsync ./out (every release)
#   bash scripts/deploy.sh --tls     # once DNS points at the host: Let's Encrypt + HTTPS redirect
#
# SWEEPLOOM_HOST (default root@46.225.60.104) and SWEEPLOOM_SSH_KEY
# (default ~/.ssh/hetzner-prod) override the target. --tls registers without
# an email unless SWEEPLOOM_TLS_EMAIL is set; certbot's timer renews either way.
set -euo pipefail

HOST="${SWEEPLOOM_HOST:-root@46.225.60.104}"
KEY="${SWEEPLOOM_SSH_KEY:-$HOME/.ssh/hetzner-prod}"
WEBROOT=/var/www/sweeploom.com
SSH=(ssh -i "$KEY" -o BatchMode=yes -o ConnectTimeout=15)
SCP=(scp -i "$KEY" -o BatchMode=yes -o ConnectTimeout=15)

cd "$(dirname "$0")/.."

remote() { "${SSH[@]}" "$HOST" "$@"; }

setup() {
  # Once certbot has added the TLS block, the live config is the source of truth.
  if remote "grep -q 'managed by Certbot' /etc/nginx/sites-available/sweeploom.com 2>/dev/null"; then
    echo "nginx config already has TLS from certbot; leaving it in place"
  else
    "${SCP[@]}" deploy/nginx.conf "$HOST:/etc/nginx/sites-available/sweeploom.com"
  fi
  remote bash -s <<'REMOTE'
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
command -v nginx >/dev/null || apt-get install -y -q nginx
command -v certbot >/dev/null || apt-get install -y -q certbot python3-certbot-nginx
mkdir -p /var/www/sweeploom.com
ln -sf /etc/nginx/sites-available/sweeploom.com /etc/nginx/sites-enabled/sweeploom.com
rm -f /etc/nginx/sites-enabled/default
nginx -t
systemctl enable --now nginx
systemctl reload nginx
REMOTE
}

publish() {
  npm run build
  rsync -az --delete -e "${SSH[*]}" out/ "$HOST:$WEBROOT/"
  remote "chown -R www-data:www-data $WEBROOT && nginx -t && systemctl reload nginx"
  local ip="${HOST#*@}"
  for path in / /blog/ /terms/ /rss.xml; do
    printf '%-10s %s\n' "$path" \
      "$(curl -s -o /dev/null -w '%{http_code}' --resolve "sweeploom.com:443:$ip" \
        "https://sweeploom.com$path" || curl -s -o /dev/null -w '%{http_code}' \
        -H 'Host: sweeploom.com' "http://$ip$path")"
  done
}

tls() {
  local account=(--register-unsafely-without-email)
  [[ -n "${SWEEPLOOM_TLS_EMAIL:-}" ]] && account=(-m "$SWEEPLOOM_TLS_EMAIL")
  remote certbot --nginx --non-interactive --agree-tos --redirect "${account[@]}" \
    -d sweeploom.com -d www.sweeploom.com
}

case "${1:-}" in
  --setup) setup; publish ;;
  --tls) tls ;;
  "") publish ;;
  *) echo "usage: $0 [--setup|--tls]" >&2; exit 2 ;;
esac
