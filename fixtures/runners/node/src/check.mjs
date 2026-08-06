// SWORN v0.1-final golden vector cross-check (Node.js runner).
//
// Reads the reference test vectors and verifies that this implementation's
// canonical-bytes serializer produces byte-for-byte identical output to the
// fixtures, that signing the reconstructed canonical bytes with the given
// seed reproduces expected_signature_hex, and that expected_signature_hex
// verifies against the reconstructed canonical bytes + signer pubkey.
//
// Per SPEC §10.2 T-1 (Verifier) and T-5 (Signer). Each vector is reported
// against three independent checks (canonical / signature / verify) so a
// failure diagnoses itself per-mode: a canonical mismatch is a serialization
// bug, a signature mismatch is a signing-path bug (usually Ed25519ph vs
// PureEdDSA), a verify mismatch is a verifier-side pubkey handling bug or
// a fixture transcription error.
//
// Exit codes match the Rust runner:
//   0  every vector passed all three checks
//   1  at least one vector failed at least one check
//   2  argv or vector-file schema error

import { readFileSync } from "node:fs";
import { createPrivateKey, createPublicKey, sign, verify } from "node:crypto";
import {
  serializeCanonicalBytesV2,
  CANONICAL_BYTES_LEN_V2,
} from "./canonical.mjs";

/**
 * Node's crypto.sign for Ed25519 requires a KeyObject built from a raw
 * 32-byte seed. RFC 8410 defines the PKCS#8 form:
 *
 *   SEQUENCE {
 *     INTEGER 0,                        -- version
 *     SEQUENCE { OID 1.3.101.112 },     -- algorithm identifier (Ed25519)
 *     OCTET STRING { OCTET STRING seed } -- privateKey (wrapped)
 *   }
 *
 * The concrete DER prefix is stable at 16 bytes.
 *
 * @param {Buffer} seed
 */
function ed25519PrivateKeyFromSeed(seed) {
  if (seed.length !== 32) {
    throw new Error(`seed must be 32 bytes, got ${seed.length}`);
  }
  const der = Buffer.concat([
    Buffer.from("302e020100300506032b657004220420", "hex"),
    seed,
  ]);
  return createPrivateKey({ key: der, format: "der", type: "pkcs8" });
}

/**
 * SubjectPublicKeyInfo DER wrapper for a raw 32-byte Ed25519 public key.
 * @param {Buffer} pub
 */
function ed25519PublicKeyFromRaw(pub) {
  if (pub.length !== 32) {
    throw new Error(`public key must be 32 bytes, got ${pub.length}`);
  }
  const der = Buffer.concat([
    Buffer.from("302a300506032b6570032100", "hex"),
    pub,
  ]);
  return createPublicKey({ key: der, format: "der", type: "spki" });
}

/** @param {any} v */
function checkVector(v) {
  /** @type {string[]} */
  const notes = [];
  const f = v.input_fields;

  const canonical = serializeCanonicalBytesV2({
    signer: Buffer.from(f.signer_hex, "hex"),
    subject: Buffer.from(f.subject_hex, "hex"),
    activityHash: Buffer.from(f.activity_hash_hex, "hex"),
    dataHash: Buffer.from(f.data_hash_hex, "hex"),
    witnessFor: Buffer.from(f.witness_for_hex, "hex"),
    sourceHash: Buffer.from(f.source_hash_hex, "hex"),
    sourceType: f.source_type,
    confidence: f.confidence,
    witnessingDepth: f.witnessing_depth,
    attestorRelationship: f.attestor_relationship,
    createdAt: BigInt(f.signer_asserted_at),
    retentionHint: BigInt(f.retention_hint),
    nonce: Buffer.from(f.nonce_hex, "hex"),
  });

  if (canonical.length !== CANONICAL_BYTES_LEN_V2) {
    notes.push(`length mismatch: got ${canonical.length}, want ${CANONICAL_BYTES_LEN_V2}`);
  }

  const canonicalHex = canonical.toString("hex");
  const canonicalOk = canonicalHex === v.expected_canonical_bytes_hex;
  if (!canonicalOk) {
    notes.push(`canonical hex mismatch`);
    notes.push(`  ours:     ${canonicalHex}`);
    notes.push(`  expected: ${v.expected_canonical_bytes_hex}`);
  }

  // T-5: Re-sign with our copy of the private key and compare byte-for-byte.
  // sign(null, message, keyObject) is Node's PureEdDSA path per SPEC §3.2.
  // Do NOT use crypto.createSign('ed25519') — that path prehashes and produces
  // Ed25519ph signatures, which SPEC §3.2 explicitly forbids.
  const seed = Buffer.from(v.signer_secret_seed_hex, "hex");
  const privateKey = ed25519PrivateKeyFromSeed(seed);
  const ourSignature = sign(null, canonical, privateKey);
  const signatureOk = ourSignature.toString("hex") === v.expected_signature_hex;
  if (!signatureOk) {
    notes.push(`signature mismatch`);
    notes.push(`  ours:     ${ourSignature.toString("hex")}`);
    notes.push(`  expected: ${v.expected_signature_hex}`);
  }

  // T-1: Verify the expected signature against our reconstructed canonical
  // bytes using the vector's stated signer pubkey. Round-trip proof that a
  // verifier built on our serializer accepts fixtures produced by any other
  // conforming implementation.
  const publicKey = ed25519PublicKeyFromRaw(Buffer.from(f.signer_hex, "hex"));
  const expectedSig = Buffer.from(v.expected_signature_hex, "hex");
  const verifyOk = verify(null, canonical, publicKey, expectedSig);
  if (!verifyOk) {
    notes.push(`signature verification failed against expected_signature`);
  }

  return { canonicalOk, signatureOk, verifyOk, notes };
}

function usage() {
  console.error("usage: node src/check.mjs <path-to-vectors.json>");
  console.error("");
  console.error("  The path argument is required. There is no default. A runner");
  console.error("  that silently checks the wrong vectors is worse than one that");
  console.error("  refuses to run.");
  process.exit(2);
}

function main() {
  const path = process.argv[2];
  if (!path) usage();

  /** @type {any} */
  let parsed;
  try {
    parsed = JSON.parse(readFileSync(path, "utf8"));
  } catch (err) {
    console.error(`failed to read or parse ${path}: ${(err && err.message) || err}`);
    process.exit(2);
  }

  if (!Array.isArray(parsed.vectors)) {
    console.error(`${path}: expected top-level 'vectors' array`);
    process.exit(2);
  }

  console.log(`sworn-vector-check: ${parsed.spec_version_name || "SPEC"} (${parsed.vectors.length} vectors)`);
  console.log(`  fixtures: ${path}`);
  console.log();

  let passed = 0;
  let failed = 0;
  const maxNameLen = Math.max(...parsed.vectors.map((/** @type {any} */ v) => v.name.length));

  for (const v of parsed.vectors) {
    const { canonicalOk, signatureOk, verifyOk, notes } = checkVector(v);
    const allOk = canonicalOk && signatureOk && verifyOk;
    const mark = allOk ? "✓" : "✗";
    const paddedName = v.name.padEnd(maxNameLen);
    const line = `  ${mark} ${paddedName}    canonical ${canonicalOk ? "✓" : "✗"}  signature ${signatureOk ? "✓" : "✗"}  verify ${verifyOk ? "✓" : "✗"}`;
    console.log(line);
    if (allOk) {
      passed += 1;
    } else {
      for (const note of notes) console.log(`      ${note}`);
      failed += 1;
    }
  }

  console.log();
  console.log(`${passed}/${parsed.vectors.length} passed${failed > 0 ? `, ${failed} failed` : "."}`);
  process.exit(failed === 0 ? 0 : 1);
}

main();
