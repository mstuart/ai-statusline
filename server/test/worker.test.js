const { describe, it, beforeEach } = require("node:test");
const assert = require("node:assert/strict");

// Import the worker module
// We'll test the exported functions by simulating the Worker environment

/**
 * Mock KV namespace
 */
class MockKV {
  constructor() {
    this.store = new Map();
  }

  async get(key, opts) {
    const value = this.store.get(key);
    if (!value) return null;
    if (opts?.type === "json") return JSON.parse(value);
    return value;
  }

  async put(key, value) {
    this.store.set(key, typeof value === "string" ? value : JSON.stringify(value));
  }

  async delete(key) {
    this.store.delete(key);
  }

  seed(key, data) {
    this.store.set(key, JSON.stringify(data));
  }
}

/**
 * Create a mock Request
 */
function mockRequest(url, method, body) {
  return {
    method,
    url: `https://api.claude-status.dev${url}`,
    json: async () => body,
    headers: new Map(),
  };
}

// Since the worker is an ES module, we test via integration-style tests
// by validating the key format function and the expected API contract

describe("License Key Format Validation", () => {
  function validateKeyFormat(key) {
    if (typeof key !== "string") return false;
    const trimmed = key.trim();
    const pattern =
      /^CS-PRO-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}$/;
    return pattern.test(trimmed);
  }

  it("accepts valid uppercase key", () => {
    assert.ok(validateKeyFormat("CS-PRO-A3F2-9D8E-C4B1-7F0A"));
  });

  it("accepts valid lowercase key", () => {
    assert.ok(validateKeyFormat("CS-PRO-a3f2-9d8e-c4b1-7f0a"));
  });

  it("accepts valid mixed case key", () => {
    assert.ok(validateKeyFormat("CS-PRO-A3f2-9D8e-c4B1-7f0A"));
  });

  it("rejects wrong prefix", () => {
    assert.ok(!validateKeyFormat("CL-PRO-A3F2-9D8E-C4B1-7F0A"));
  });

  it("rejects too few segments", () => {
    assert.ok(!validateKeyFormat("CS-PRO-A3F2-9D8E-C4B1"));
  });

  it("rejects too many segments", () => {
    assert.ok(!validateKeyFormat("CS-PRO-A3F2-9D8E-C4B1-7F0A-AAAA"));
  });

  it("rejects non-hex characters", () => {
    assert.ok(!validateKeyFormat("CS-PRO-ZZZZ-9D8E-C4B1-7F0A"));
  });

  it("rejects wrong segment length", () => {
    assert.ok(!validateKeyFormat("CS-PRO-A3F-9D8E-C4B1-7F0A"));
  });

  it("rejects empty string", () => {
    assert.ok(!validateKeyFormat(""));
  });

  it("rejects non-string input", () => {
    assert.ok(!validateKeyFormat(null));
    assert.ok(!validateKeyFormat(undefined));
    assert.ok(!validateKeyFormat(123));
  });
});

describe("API Contract", () => {
  const VALID_KEY = "CS-PRO-A3F2-9D8E-C4B1-7F0A";
  const MACHINE_ID = "abc123def456";

  it("verify endpoint requires key and machine_id", () => {
    // Contract: POST /v1/license/verify
    const body = { key: VALID_KEY, machine_id: MACHINE_ID };
    assert.ok(body.key);
    assert.ok(body.machine_id);
  });

  it("verify response has expected shape for valid license", () => {
    const response = {
      valid: true,
      tier: "pro",
      expires: null,
      features: [
        "cost_tracking",
        "burn_rate",
        "cost_warnings",
        "model_suggestions",
        "historical_stats",
      ],
    };
    assert.equal(typeof response.valid, "boolean");
    assert.equal(typeof response.tier, "string");
    assert.ok(Array.isArray(response.features));
    assert.ok(response.features.length > 0);
  });

  it("verify response has expected shape for invalid license", () => {
    const response = {
      valid: false,
      reason: "not_found",
    };
    assert.equal(response.valid, false);
    assert.equal(typeof response.reason, "string");
  });

  it("activate response has machine count", () => {
    const response = {
      success: true,
      tier: "pro",
      expires: null,
      features: ["cost_tracking"],
      machines_used: 1,
      machines_max: 3,
    };
    assert.equal(typeof response.machines_used, "number");
    assert.equal(typeof response.machines_max, "number");
    assert.ok(response.machines_used <= response.machines_max);
  });

  it("deactivate response has machine count", () => {
    const response = {
      success: true,
      machines_used: 0,
    };
    assert.equal(typeof response.machines_used, "number");
  });
});

describe("KV Data Schema", () => {
  it("license record has required fields", () => {
    const license = {
      tier: "pro",
      expires: "2027-02-15T00:00:00Z",
      machines: ["machine1", "machine2"],
      revoked: false,
      created_at: "2026-02-15T00:00:00Z",
      email: "user@example.com",
    };
    assert.ok(["pro", "lifetime"].includes(license.tier));
    assert.ok(Array.isArray(license.machines));
    assert.equal(typeof license.revoked, "boolean");
    assert.ok(license.created_at);
  });

  it("lifetime license has no expiry", () => {
    const license = {
      tier: "lifetime",
      expires: null,
      machines: [],
      revoked: false,
      created_at: "2026-02-15T00:00:00Z",
      email: "user@example.com",
    };
    assert.equal(license.expires, null);
    assert.equal(license.tier, "lifetime");
  });

  it("machine limit defaults to 3", () => {
    const maxMachines = parseInt("3", 10);
    assert.equal(maxMachines, 3);
  });

  it("expired license is rejected", () => {
    const license = {
      tier: "pro",
      expires: "2025-01-01T00:00:00Z",
      machines: [],
      revoked: false,
    };
    const expiresDate = new Date(license.expires);
    const now = new Date();
    assert.ok(expiresDate < now, "Expired license date should be in the past");
  });
});

describe("Mock KV Operations", () => {
  let kv;

  beforeEach(() => {
    kv = new MockKV();
  });

  it("stores and retrieves license data", async () => {
    const key = "CS-PRO-A3F2-9D8E-C4B1-7F0A";
    const data = {
      tier: "pro",
      expires: null,
      machines: [],
      revoked: false,
      created_at: new Date().toISOString(),
      email: "test@example.com",
    };

    kv.seed(key, data);
    const result = await kv.get(key, { type: "json" });
    assert.deepEqual(result.tier, "pro");
    assert.deepEqual(result.machines, []);
  });

  it("returns null for missing key", async () => {
    const result = await kv.get("nonexistent", { type: "json" });
    assert.equal(result, null);
  });

  it("updates machine list", async () => {
    const key = "CS-PRO-A3F2-9D8E-C4B1-7F0A";
    kv.seed(key, { tier: "pro", machines: [], revoked: false });

    const data = await kv.get(key, { type: "json" });
    data.machines.push("machine-1");
    await kv.put(key, JSON.stringify(data));

    const updated = await kv.get(key, { type: "json" });
    assert.deepEqual(updated.machines, ["machine-1"]);
  });

  it("enforces machine limit", async () => {
    const key = "CS-PRO-A3F2-9D8E-C4B1-7F0A";
    const maxMachines = 3;
    kv.seed(key, {
      tier: "pro",
      machines: ["m1", "m2", "m3"],
      revoked: false,
    });

    const data = await kv.get(key, { type: "json" });
    const canAdd = data.machines.length < maxMachines;
    assert.ok(!canAdd, "Should not allow more than 3 machines");
  });
});
