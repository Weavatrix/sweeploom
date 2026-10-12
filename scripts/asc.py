#!/usr/bin/env python3
"""Minimal App Store Connect API client for the SweepLoom Mac App Store edition.

  bundle-id   register com.weavatrix.sweeploom.mas for MAC_OS when it is missing
  profile     create (or reuse) a MAC_APP_STORE provisioning profile for it with the
              Apple Distribution certificate and save it under target/
  apps        list the account's apps and point out SweepLoom's record
  self-test   offline check of the ES256 JWT signer with a throwaway key

Needs ASC_ISSUER_ID (App Store Connect → Users and Access → Integrations). The key
is read from ~/.private_keys/AuthKey_<ASC_KEY_ID>.p8 and never printed or copied.
"""

import argparse
import base64
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
API = "https://api.appstoreconnect.apple.com"
KEY_ID = os.environ.get("ASC_KEY_ID", "")
BUNDLE_ID = "com.weavatrix.sweeploom.mas"
BUNDLE_NAME = "SweepLoom App Store"
PROFILE_NAME = "SweepLoom Mac App Store"
DISTRIBUTION_NAME = "Apple Distribution: Serhii Ziborov (XMS5ZC28UJ)"


def b64url(data):
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode()


def der_to_raw(der, size=32):
    """ECDSA DER SEQUENCE{INTEGER r, INTEGER s} → fixed-width r || s (JWS ES256)."""
    def element(buffer, index):
        tag, length = buffer[index], buffer[index + 1]
        index += 2
        if length & 0x80:
            count = length & 0x7F
            length = int.from_bytes(buffer[index:index + count], "big")
            index += count
        return tag, buffer[index:index + length], index + length

    tag, body, _ = element(der, 0)
    if tag != 0x30:
        raise ValueError("signature is not a DER sequence")
    parts, index = [], 0
    while index < len(body):
        tag, value, index = element(body, index)
        if tag != 0x02:
            raise ValueError("signature part is not an integer")
        parts.append(int.from_bytes(value, "big"))
    if len(parts) != 2:
        raise ValueError("signature needs exactly r and s")
    return b"".join(part.to_bytes(size, "big") for part in parts)


def sign_es256(key_path, message):
    try:
        from cryptography.hazmat.primitives import hashes, serialization
        from cryptography.hazmat.primitives.asymmetric import ec
    except ImportError:
        der = subprocess.run(["openssl", "dgst", "-sha256", "-sign", str(key_path)],
                             input=message, capture_output=True, check=True).stdout
        return der_to_raw(der)
    from cryptography.hazmat.primitives.asymmetric.utils import decode_dss_signature
    key = serialization.load_pem_private_key(Path(key_path).read_bytes(), password=None)
    r, s = decode_dss_signature(key.sign(message, ec.ECDSA(hashes.SHA256())))
    return r.to_bytes(32, "big") + s.to_bytes(32, "big")


def make_jwt(key_path, key_id, issuer, now=None, lifetime=1200):
    now = int(now if now is not None else time.time())
    header = {"alg": "ES256", "kid": key_id, "typ": "JWT"}
    claims = {"iss": issuer, "iat": now, "exp": now + lifetime, "aud": "appstoreconnect-v1"}
    signing_input = ".".join(b64url(json.dumps(part, separators=(",", ":")).encode())
                             for part in (header, claims)).encode()
    return signing_input.decode() + "." + b64url(sign_es256(key_path, signing_input))


class Client:
    def __init__(self, issuer):
        if not KEY_ID:
            raise SystemExit("set ASC_KEY_ID (the App Store Connect API key id)")
        key = Path(os.environ.get("ASC_KEY_PATH",
                                  Path.home() / ".private_keys" / f"AuthKey_{KEY_ID}.p8"))
        if not key.is_file():
            raise SystemExit(f"API key not found: {key}")
        self.token = make_jwt(key, KEY_ID, issuer)

    def call(self, method, path, query=None, body=None):
        url = API + path + ("?" + urllib.parse.urlencode(query) if query else "")
        data = json.dumps(body).encode() if body is not None else None
        request = urllib.request.Request(url, data=data, method=method, headers={
            "Authorization": f"Bearer {self.token}", "Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(request, timeout=60) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            detail = error.read().decode(errors="replace")
            raise SystemExit(f"{method} {path}: HTTP {error.code}\n{detail}") from None

    def bundle(self):
        found = self.call("GET", "/v1/bundleIds", {"filter[identifier]": BUNDLE_ID,
                                                   "limit": "200"})["data"]
        return next((item for item in found
                     if item["attributes"]["identifier"] == BUNDLE_ID), None)


def local_serial():
    """Serial of the local Apple Distribution certificate, to pick the matching one."""
    try:
        pem = subprocess.run(["security", "find-certificate", "-c", DISTRIBUTION_NAME, "-p"],
                             capture_output=True, check=True).stdout
        out = subprocess.run(["openssl", "x509", "-noout", "-serial"], input=pem,
                             capture_output=True, check=True).stdout.decode()
        return out.strip().split("=", 1)[1].upper().lstrip("0")
    except (subprocess.CalledProcessError, IndexError, FileNotFoundError):
        return None


def cmd_bundle_id(client):
    existing = client.bundle()
    if existing:
        attrs = existing["attributes"]
        print(f"Bundle ID exists: {attrs['identifier']} ({attrs['platform']}) id={existing['id']}")
        return existing
    created = client.call("POST", "/v1/bundleIds", body={"data": {"type": "bundleIds",
        "attributes": {"identifier": BUNDLE_ID, "name": BUNDLE_NAME, "platform": "MAC_OS"}}})
    print(f"Registered {BUNDLE_ID} for MAC_OS, id={created['data']['id']}")
    return created["data"]


def distribution_certificate(client):
    certs = client.call("GET", "/v1/certificates", {
        "filter[certificateType]": "DISTRIBUTION,MAC_APP_DISTRIBUTION", "limit": "200"})["data"]
    serial = local_serial()
    # Latest expiry first, then (stable) the local keychain's certificate and Apple Distribution.
    certs.sort(key=lambda item: item["attributes"].get("expirationDate", ""), reverse=True)
    certs.sort(key=lambda item: (
        item["attributes"].get("serialNumber", "").upper().lstrip("0") != serial,
        item["attributes"]["certificateType"] != "DISTRIBUTION"))
    if not certs:
        raise SystemExit("No Apple Distribution certificate in the account.")
    chosen = certs[0]["attributes"]
    print(f"Certificate: {chosen.get('name')} ({chosen['certificateType']}), "
          f"expires {chosen.get('expirationDate')}")
    return certs[0]


def cmd_profile(client):
    bundle = client.bundle() or cmd_bundle_id(client)
    cert = distribution_certificate(client)
    profiles = client.call("GET", f"/v1/bundleIds/{bundle['id']}/profiles",
                           {"limit": "200"})["data"]
    usable = [item for item in profiles if item["attributes"]["profileType"] == "MAC_APP_STORE"
              and item["attributes"]["profileState"] == "ACTIVE"
              and any(entry["id"] == cert["id"] for entry in client.call(
                  "GET", f"/v1/profiles/{item['id']}/certificates")["data"])]
    if usable:
        profile = client.call("GET", f"/v1/profiles/{usable[0]['id']}")["data"]
        print(f"Reusing active profile {profile['attributes']['name']!r}")
    else:
        name = PROFILE_NAME if not profiles else f"{PROFILE_NAME} {time.strftime('%Y%m%d%H%M')}"
        profile = client.call("POST", "/v1/profiles", body={"data": {"type": "profiles",
            "attributes": {"name": name, "profileType": "MAC_APP_STORE"},
            "relationships": {
                "bundleId": {"data": {"type": "bundleIds", "id": bundle["id"]}},
                "certificates": {"data": [{"type": "certificates", "id": cert["id"]}]}}}})["data"]
        print(f"Created profile {name!r}")
    out = ROOT / "target" / "SweepLoom_Mac_App_Store.provisionprofile"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(base64.b64decode(profile["attributes"]["profileContent"]))
    print(f"Saved {out}\nNext: python3 scripts/macos-mas.py --store --profile {out}")


def cmd_apps(client):
    apps = client.call("GET", "/v1/apps", {"fields[apps]": "name,bundleId,sku",
                                           "limit": "200"})["data"]
    for item in apps:
        attrs = item["attributes"]
        mark = "  <- SweepLoom App Store edition" if attrs["bundleId"] == BUNDLE_ID else ""
        print(f"{item['id']}  {attrs['bundleId']}  {attrs['name']}{mark}")
    if not any(item["attributes"]["bundleId"] == BUNDLE_ID for item in apps):
        print(f"No app record uses {BUNDLE_ID} yet. Create it in App Store Connect "
              "(My Apps → + → New App, platform macOS); the API cannot create apps.")


def cmd_self_test():
    with tempfile.TemporaryDirectory() as tmp:
        key = Path(tmp) / "test.p8"
        subprocess.run(["openssl", "genpkey", "-algorithm", "EC", "-pkeyopt",
                        "ec_paramgen_curve:P-256", "-out", str(key)], check=True,
                       capture_output=True)
        token = make_jwt(key, "TESTKEY123", "issuer-uuid", now=1_700_000_000)
        head, claims, signature = token.split(".")
        pad = lambda text: base64.urlsafe_b64decode(text + "=" * (-len(text) % 4))
        assert json.loads(pad(head)) == {"alg": "ES256", "kid": "TESTKEY123", "typ": "JWT"}
        body = json.loads(pad(claims))
        assert body["aud"] == "appstoreconnect-v1" and body["exp"] - body["iat"] == 1200
        raw = pad(signature)
        assert len(raw) == 64, "ES256 signatures are 64 raw bytes"
        der = raw_to_der(raw)
        public = Path(tmp) / "test.pub"
        subprocess.run(["openssl", "pkey", "-in", str(key), "-pubout", "-out", str(public)],
                       check=True, capture_output=True)
        (Path(tmp) / "sig.der").write_bytes(der)
        verify = subprocess.run(["openssl", "dgst", "-sha256", "-verify", str(public),
                                 "-signature", str(Path(tmp) / "sig.der")],
                                input=f"{head}.{claims}".encode(), capture_output=True)
        assert verify.returncode == 0, verify.stdout + verify.stderr
        assert der_to_raw(der) == raw
    print("self-test ok: ES256 JWT header, claims and signature verify with openssl")


def raw_to_der(raw):
    def integer(value):
        value = value.lstrip(b"\0") or b"\0"
        if value[0] & 0x80:
            value = b"\0" + value
        return b"\x02" + bytes([len(value)]) + value
    body = integer(raw[:32]) + integer(raw[32:])
    return b"\x30" + bytes([len(body)]) + body


def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["bundle-id", "profile", "apps", "self-test"])
    args = parser.parse_args()
    if args.command == "self-test":
        return cmd_self_test()
    issuer = os.environ.get("ASC_ISSUER_ID")
    if not issuer:
        raise SystemExit("Set ASC_ISSUER_ID to the App Store Connect API issuer ID.")
    client = Client(issuer)
    {"bundle-id": cmd_bundle_id, "profile": cmd_profile, "apps": cmd_apps}[args.command](client)


if __name__ == "__main__":
    main()
