#!/usr/bin/env node
/**
 * Seed a license key into Cloudflare KV for testing or provisioning.
 *
 * Usage:
 *   node seed-license.js <key> <email> [tier] [expires]
 *
 * Examples:
 *   node seed-license.js CS-PRO-A3F2-9D8E-C4B1-7F0A user@example.com
 *   node seed-license.js CS-PRO-A3F2-9D8E-C4B1-7F0A user@example.com lifetime
 *   node seed-license.js CS-PRO-A3F2-9D8E-C4B1-7F0A user@example.com pro 2027-02-15
 *
 * This outputs the wrangler command to run.
 */

const args = process.argv.slice(2);

if (args.length < 2) {
  console.error("Usage: node seed-license.js <key> <email> [tier] [expires]");
  console.error("");
  console.error("  key     License key (CS-PRO-XXXX-XXXX-XXXX-XXXX)");
  console.error("  email   Customer email");
  console.error("  tier    pro (default) or lifetime");
  console.error("  expires Expiry date (ISO 8601), null for lifetime");
  process.exit(1);
}

const [key, email, tier = "pro", expires = null] = args;

// Validate key format
const pattern =
  /^CS-PRO-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}$/;
if (!pattern.test(key)) {
  console.error(`Invalid key format: ${key}`);
  console.error("Expected: CS-PRO-XXXX-XXXX-XXXX-XXXX (hex characters)");
  process.exit(1);
}

const licenseData = {
  tier,
  expires: tier === "lifetime" ? null : expires ? new Date(expires).toISOString() : null,
  machines: [],
  revoked: false,
  created_at: new Date().toISOString(),
  email,
};

const jsonValue = JSON.stringify(licenseData);

console.log("License data:");
console.log(JSON.stringify(licenseData, null, 2));
console.log("");
console.log("Run this wrangler command to seed the license:");
console.log("");
console.log(
  `wrangler kv:key put --binding LICENSES "${key}" '${jsonValue}'`
);
