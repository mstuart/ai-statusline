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
  "Access-Control-Allow-Headers": "Content-Type",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
  "Access-Control-Allow-Origin": "*",
};

const PRO_FEATURES = [
  "cost_tracking",
  "burn_rate",
  "cost_warnings",
  "model_suggestions",
  "historical_stats",
];

const LICENSE_KEY_PATTERN =
  /^CS-PRO-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}$/;

export default {
  fetch(request, env) {
    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, { headers: CORS_HEADERS, status: 204 });
    }

    const url = new URL(request.url);

    // Route requests
    if (url.pathname === "/v1/license/verify" && request.method === "POST") {
      return handleVerify(request, env);
    }

    if (url.pathname === "/v1/license/activate" && request.method === "POST") {
      return handleActivate(request, env);
    }

    if (
      url.pathname === "/v1/license/deactivate" &&
      request.method === "POST"
    ) {
      return handleDeactivate(request, env);
    }

    if (url.pathname === "/health") {
      return jsonResponse({
        status: "ok",
        timestamp: new Date().toISOString(),
      });
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
    return jsonResponse({ reason: "invalid_request", valid: false }, 400);
  }

  const { key, machine_id } = body;

  if (!key || typeof key !== "string") {
    return jsonResponse({ reason: "missing_key", valid: false }, 400);
  }

  if (!machine_id || typeof machine_id !== "string") {
    return jsonResponse({ reason: "missing_machine_id", valid: false }, 400);
  }

  // Validate key format
  if (!validateKeyFormat(key)) {
    return jsonResponse({ reason: "invalid_format", valid: false });
  }

  // Look up license in KV
  const licenseData = await env.LICENSES.get(key, { type: "json" });

  if (!licenseData) {
    return jsonResponse({ reason: "not_found", valid: false });
  }

  if (licenseData.revoked) {
    return jsonResponse({ reason: "revoked", valid: false });
  }

  // Check expiration
  if (licenseData.expires) {
    const expiresDate = new Date(licenseData.expires);
    if (expiresDate < new Date()) {
      return jsonResponse({ reason: "expired", valid: false });
    }
  }

  // Check machine limit
  const maxMachines = Number.parseInt(env.MAX_MACHINES_PER_LICENSE || "3", 10);
  const machines = licenseData.machines || [];

  if (!machines.includes(machine_id)) {
    if (machines.length >= maxMachines) {
      return jsonResponse({
        max_devices: maxMachines,
        reason: "device_limit",
        valid: false,
      });
    }

    // Register this machine
    machines.push(machine_id);
    licenseData.machines = machines;
    await env.LICENSES.put(key, JSON.stringify(licenseData));
  }

  return jsonResponse({
    expires: licenseData.expires || null,
    features: PRO_FEATURES,
    tier: licenseData.tier || "pro",
    valid: true,
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
    return jsonResponse({ reason: "invalid_request", success: false }, 400);
  }

  const { key, machine_id } = body;

  if (!(key && machine_id)) {
    return jsonResponse({ reason: "missing_fields", success: false }, 400);
  }

  if (!validateKeyFormat(key)) {
    return jsonResponse({ reason: "invalid_format", success: false }, 400);
  }

  const licenseData = await env.LICENSES.get(key, { type: "json" });

  if (!licenseData) {
    return jsonResponse({ reason: "not_found", success: false });
  }

  if (licenseData.revoked) {
    return jsonResponse({ reason: "revoked", success: false });
  }

  if (licenseData.expires) {
    const expiresDate = new Date(licenseData.expires);
    if (expiresDate < new Date()) {
      return jsonResponse({ reason: "expired", success: false });
    }
  }

  const maxMachines = Number.parseInt(env.MAX_MACHINES_PER_LICENSE || "3", 10);
  const machines = licenseData.machines || [];

  if (!machines.includes(machine_id)) {
    if (machines.length >= maxMachines) {
      return jsonResponse({
        max_devices: maxMachines,
        reason: "device_limit",
        success: false,
      });
    }
    machines.push(machine_id);
    licenseData.machines = machines;
    await env.LICENSES.put(key, JSON.stringify(licenseData));
  }

  return jsonResponse({
    expires: licenseData.expires || null,
    features: PRO_FEATURES,
    machines_max: maxMachines,
    machines_used: machines.length,
    success: true,
    tier: licenseData.tier || "pro",
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
    return jsonResponse({ reason: "invalid_request", success: false }, 400);
  }

  const { key, machine_id } = body;

  if (!(key && machine_id)) {
    return jsonResponse({ reason: "missing_fields", success: false }, 400);
  }

  const licenseData = await env.LICENSES.get(key, { type: "json" });

  if (!licenseData) {
    return jsonResponse({ reason: "not_found", success: false });
  }

  const machines = licenseData.machines || [];
  const index = machines.indexOf(machine_id);

  if (index !== -1) {
    machines.splice(index, 1);
    licenseData.machines = machines;
    await env.LICENSES.put(key, JSON.stringify(licenseData));
  }

  return jsonResponse({
    machines_used: machines.length,
    success: true,
  });
}

/**
 * Validate license key format: CS-PRO-XXXX-XXXX-XXXX-XXXX (hex chars)
 */
function validateKeyFormat(key) {
  if (typeof key !== "string") {
    return false;
  }
  const trimmed = key.trim();
  return LICENSE_KEY_PATTERN.test(trimmed);
}

/**
 * Return a JSON response with CORS headers.
 */
function jsonResponse(data, status = 200) {
  return new Response(JSON.stringify(data), {
    headers: {
      "Content-Type": "application/json",
      ...CORS_HEADERS,
    },
    status,
  });
}
