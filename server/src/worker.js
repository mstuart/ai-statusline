/**
 * claude-status License Server
 * Cloudflare Worker for Pro license validation
 *
 * KV Schema (LICENSES namespace):
 *   key: license key string (e.g., "CS-PRO-A3F2-9D8E-C4B1-7F0A")
 *   value: JSON {
 *     tier: "pro" | "lifetime",
 *     expires: ISO 8601 string | null,
 *     machines: string[],  // machine_id hashes
 *     revoked: boolean,
 *     created_at: ISO 8601 string,
 *     email: string
 *   }
 */

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type",
};

const PRO_FEATURES = [
  "cost_tracking",
  "burn_rate",
  "cost_warnings",
  "model_suggestions",
  "historical_stats",
];

export default {
  async fetch(request, env) {
    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    const url = new URL(request.url);

    // Route requests
    if (url.pathname === "/v1/license/verify" && request.method === "POST") {
      return handleVerify(request, env);
    }

    if (url.pathname === "/v1/license/activate" && request.method === "POST") {
      return handleActivate(request, env);
    }

    if (url.pathname === "/v1/license/deactivate" && request.method === "POST") {
      return handleDeactivate(request, env);
    }

    if (url.pathname === "/health") {
      return jsonResponse({ status: "ok", timestamp: new Date().toISOString() });
    }

    return jsonResponse({ error: "Not found" }, 404);
  },
};

/**
 * POST /v1/license/verify
 * Body: { key: string, machine_id: string }
 * Response: { valid: boolean, tier?: string, expires?: string, features?: string[], reason?: string }
 */
async function handleVerify(request, env) {
  let body;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ valid: false, reason: "invalid_request" }, 400);
  }

  const { key, machine_id } = body;

  if (!key || typeof key !== "string") {
    return jsonResponse({ valid: false, reason: "missing_key" }, 400);
  }

  if (!machine_id || typeof machine_id !== "string") {
    return jsonResponse({ valid: false, reason: "missing_machine_id" }, 400);
  }

  // Validate key format
  if (!validateKeyFormat(key)) {
    return jsonResponse({ valid: false, reason: "invalid_format" });
  }

  // Look up license in KV
  const licenseData = await env.LICENSES.get(key, { type: "json" });

  if (!licenseData) {
    return jsonResponse({ valid: false, reason: "not_found" });
  }

  if (licenseData.revoked) {
    return jsonResponse({ valid: false, reason: "revoked" });
  }

  // Check expiration
  if (licenseData.expires) {
    const expiresDate = new Date(licenseData.expires);
    if (expiresDate < new Date()) {
      return jsonResponse({ valid: false, reason: "expired" });
    }
  }

  // Check machine limit
  const maxMachines = parseInt(env.MAX_MACHINES_PER_LICENSE || "3", 10);
  const machines = licenseData.machines || [];

  if (!machines.includes(machine_id)) {
    if (machines.length >= maxMachines) {
      return jsonResponse({
        valid: false,
        reason: "device_limit",
        max_devices: maxMachines,
      });
    }

    // Register this machine
    machines.push(machine_id);
    licenseData.machines = machines;
    await env.LICENSES.put(key, JSON.stringify(licenseData));
  }

  return jsonResponse({
    valid: true,
    tier: licenseData.tier || "pro",
    expires: licenseData.expires || null,
    features: PRO_FEATURES,
  });
}

/**
 * POST /v1/license/activate
 * Body: { key: string, machine_id: string, email?: string }
 * Creates or registers a machine for a license.
 */
async function handleActivate(request, env) {
  let body;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ success: false, reason: "invalid_request" }, 400);
  }

  const { key, machine_id } = body;

  if (!key || !machine_id) {
    return jsonResponse({ success: false, reason: "missing_fields" }, 400);
  }

  if (!validateKeyFormat(key)) {
    return jsonResponse({ success: false, reason: "invalid_format" }, 400);
  }

  const licenseData = await env.LICENSES.get(key, { type: "json" });

  if (!licenseData) {
    return jsonResponse({ success: false, reason: "not_found" });
  }

  if (licenseData.revoked) {
    return jsonResponse({ success: false, reason: "revoked" });
  }

  if (licenseData.expires) {
    const expiresDate = new Date(licenseData.expires);
    if (expiresDate < new Date()) {
      return jsonResponse({ success: false, reason: "expired" });
    }
  }

  const maxMachines = parseInt(env.MAX_MACHINES_PER_LICENSE || "3", 10);
  const machines = licenseData.machines || [];

  if (!machines.includes(machine_id)) {
    if (machines.length >= maxMachines) {
      return jsonResponse({
        success: false,
        reason: "device_limit",
        max_devices: maxMachines,
      });
    }
    machines.push(machine_id);
    licenseData.machines = machines;
    await env.LICENSES.put(key, JSON.stringify(licenseData));
  }

  return jsonResponse({
    success: true,
    tier: licenseData.tier || "pro",
    expires: licenseData.expires || null,
    features: PRO_FEATURES,
    machines_used: machines.length,
    machines_max: maxMachines,
  });
}

/**
 * POST /v1/license/deactivate
 * Body: { key: string, machine_id: string }
 * Removes a machine from a license.
 */
async function handleDeactivate(request, env) {
  let body;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ success: false, reason: "invalid_request" }, 400);
  }

  const { key, machine_id } = body;

  if (!key || !machine_id) {
    return jsonResponse({ success: false, reason: "missing_fields" }, 400);
  }

  const licenseData = await env.LICENSES.get(key, { type: "json" });

  if (!licenseData) {
    return jsonResponse({ success: false, reason: "not_found" });
  }

  const machines = licenseData.machines || [];
  const index = machines.indexOf(machine_id);

  if (index !== -1) {
    machines.splice(index, 1);
    licenseData.machines = machines;
    await env.LICENSES.put(key, JSON.stringify(licenseData));
  }

  return jsonResponse({
    success: true,
    machines_used: machines.length,
  });
}

/**
 * Validate license key format: CS-PRO-XXXX-XXXX-XXXX-XXXX (hex chars)
 */
function validateKeyFormat(key) {
  if (typeof key !== "string") return false;
  const trimmed = key.trim();
  const pattern = /^CS-PRO-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}$/;
  return pattern.test(trimmed);
}

/**
 * Return a JSON response with CORS headers.
 */
function jsonResponse(data, status = 200) {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      "Content-Type": "application/json",
      ...CORS_HEADERS,
    },
  });
}
