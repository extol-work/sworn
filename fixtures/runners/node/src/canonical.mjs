// SWORN v0.1-final canonical byte sequence serialization.
//
// Serializes an attestation record into the 248-byte canonical sequence
// that Ed25519 signs (per SPEC §3.1). Any conforming SWORN implementation
// MUST reproduce these bytes byte-for-byte given the same input fields.
//
// Layout (from SPEC §3.1):
//
//   canonical_bytes_v2 =
//         spec_version              (2 bytes, u16 LE)
//      || signer                    (32 bytes)
//      || subject                   (32 bytes)
//      || activity_hash             (32 bytes)
//      || data_hash                 (32 bytes)
//      || witness_for               (32 bytes)
//      || source_hash               (32 bytes)
//      || source_type               (2 bytes, u16 LE)
//      || confidence                (2 bytes, u16 LE)
//      || witnessing_depth          (1 byte)
//      || attestor_relationship     (1 byte)
//      || created_at                (8 bytes, int64 LE)
//      || retention_hint            (8 bytes, int64 LE, signed)
//      || nonce                     (32 bytes)
//
//   Total: 248 bytes.
//
// This module is deliberately dependency-free. Anyone reviewing the SWORN
// spec should be able to read this alongside SPEC §3.1 and confirm the two
// agree, without traversing a library graph.

export const SPEC_VERSION_V0_1_PREVIEW = 1;
export const SPEC_VERSION_V0_1_FINAL = 2;
export const CANONICAL_BYTES_LEN_V1 = 208;
export const CANONICAL_BYTES_LEN_V2 = 248;

/**
 * Thrown when a reader encounters a spec_version it does not recognize.
 * A version-mismatch condition is "reader is behind the registry," not
 * "attestation is corrupt." Callers MUST distinguish the two paths.
 */
export class SpecVersionUnknownError extends Error {
  /** @param {number} specVersion */
  constructor(specVersion) {
    super(
      `unknown spec_version=${specVersion}; this reader supports ${SPEC_VERSION_V0_1_PREVIEW} and ${SPEC_VERSION_V0_1_FINAL}`,
    );
    this.name = "SpecVersionUnknownError";
    this.specVersion = specVersion;
  }
}

/**
 * @typedef {object} SwornAttestationV2Input
 * @property {Buffer} signer
 * @property {Buffer} subject
 * @property {Buffer} activityHash
 * @property {Buffer} dataHash
 * @property {Buffer} witnessFor
 * @property {Buffer} sourceHash
 * @property {number} sourceType         u16, per SPEC §9.2 (0..14)
 * @property {number} confidence         u16, basis points (0..10000)
 * @property {number} witnessingDepth    u8, per SPEC §9.3 (0..5)
 * @property {number} attestorRelationship u8, per SPEC §9.4 (0..6)
 * @property {bigint} createdAt          signer_asserted_at, int64 seconds
 * @property {bigint} retentionHint      int64 (signed; -1 = keep indefinitely)
 * @property {Buffer} nonce
 */

/**
 * @typedef {object} SwornAttestationV1Input
 * @property {Buffer} signer
 * @property {Buffer} subject
 * @property {Buffer} activityHash
 * @property {Buffer} dataHash
 * @property {Buffer} witnessFor
 * @property {bigint} createdAt
 * @property {bigint} retentionHint
 * @property {Buffer} nonce
 */

/** @param {string} name @param {Buffer} buf @param {number} len */
function assertLen(name, buf, len) {
  if (!Buffer.isBuffer(buf)) {
    throw new Error(`${name}: expected Buffer, got ${typeof buf}`);
  }
  if (buf.length !== len) {
    throw new Error(`${name}: expected ${len} bytes, got ${buf.length}`);
  }
}

/** @param {string} name @param {number} value */
function assertU16(name, value) {
  if (!Number.isInteger(value) || value < 0 || value > 0xffff) {
    throw new Error(`${name}: expected u16 (0..65535), got ${value}`);
  }
}

/** @param {string} name @param {number} value */
function assertU8(name, value) {
  if (!Number.isInteger(value) || value < 0 || value > 0xff) {
    throw new Error(`${name}: expected u8 (0..255), got ${value}`);
  }
}

/**
 * Produces the 248-byte canonical byte sequence for v0.1-final attestations
 * (spec_version = 2). Field widths and byte order per SPEC §3.1.
 *
 * @param {SwornAttestationV2Input} input
 * @returns {Buffer}
 */
export function serializeCanonicalBytesV2(input) {
  assertLen("signer", input.signer, 32);
  assertLen("subject", input.subject, 32);
  assertLen("activityHash", input.activityHash, 32);
  assertLen("dataHash", input.dataHash, 32);
  assertLen("witnessFor", input.witnessFor, 32);
  assertLen("sourceHash", input.sourceHash, 32);
  assertLen("nonce", input.nonce, 32);
  assertU16("sourceType", input.sourceType);
  assertU16("confidence", input.confidence);
  assertU8("witnessingDepth", input.witnessingDepth);
  assertU8("attestorRelationship", input.attestorRelationship);

  // SPEC §2.4: sourceless source_types (unknown=0, self_reported=1) MUST have
  // source_hash = 32 zero bytes.
  if (
    (input.sourceType === 0 || input.sourceType === 1) &&
    !input.sourceHash.equals(Buffer.alloc(32))
  ) {
    throw new Error(
      `sourceless source_type=${input.sourceType} requires 32 zero bytes for source_hash`,
    );
  }

  const buf = Buffer.alloc(CANONICAL_BYTES_LEN_V2);
  let offset = 0;

  buf.writeUInt16LE(SPEC_VERSION_V0_1_FINAL, offset);
  offset += 2;

  input.signer.copy(buf, offset);
  offset += 32;
  input.subject.copy(buf, offset);
  offset += 32;
  input.activityHash.copy(buf, offset);
  offset += 32;
  input.dataHash.copy(buf, offset);
  offset += 32;
  input.witnessFor.copy(buf, offset);
  offset += 32;
  input.sourceHash.copy(buf, offset);
  offset += 32;

  buf.writeUInt16LE(input.sourceType, offset);
  offset += 2;
  buf.writeUInt16LE(input.confidence, offset);
  offset += 2;

  buf.writeUInt8(input.witnessingDepth, offset);
  offset += 1;
  buf.writeUInt8(input.attestorRelationship, offset);
  offset += 1;

  buf.writeBigInt64LE(input.createdAt, offset);
  offset += 8;
  buf.writeBigInt64LE(input.retentionHint, offset);
  offset += 8;

  input.nonce.copy(buf, offset);
  offset += 32;

  if (offset !== CANONICAL_BYTES_LEN_V2) {
    throw new Error(
      `internal error: expected ${CANONICAL_BYTES_LEN_V2}-byte output, produced ${offset}`,
    );
  }

  return buf;
}

/**
 * v0.1-preview (deprecated) 208-byte layout. Retained for reader-side
 * verification of historical rows with spec_version = 1. Writers MUST NOT
 * emit new v1 attestations — v0.1-preview is deprecated per SPEC §9.1.1.
 *
 * @param {SwornAttestationV1Input} input
 * @returns {Buffer}
 */
export function serializeCanonicalBytesV1(input) {
  assertLen("signer", input.signer, 32);
  assertLen("subject", input.subject, 32);
  assertLen("activityHash", input.activityHash, 32);
  assertLen("dataHash", input.dataHash, 32);
  assertLen("witnessFor", input.witnessFor, 32);
  assertLen("nonce", input.nonce, 32);

  const buf = Buffer.alloc(CANONICAL_BYTES_LEN_V1);
  let offset = 0;

  input.signer.copy(buf, offset);
  offset += 32;
  input.subject.copy(buf, offset);
  offset += 32;
  input.activityHash.copy(buf, offset);
  offset += 32;
  input.dataHash.copy(buf, offset);
  offset += 32;
  input.witnessFor.copy(buf, offset);
  offset += 32;

  buf.writeBigInt64LE(input.createdAt, offset);
  offset += 8;
  buf.writeBigInt64LE(input.retentionHint, offset);
  offset += 8;

  input.nonce.copy(buf, offset);
  offset += 32;

  if (offset !== CANONICAL_BYTES_LEN_V1) {
    throw new Error(
      `internal error: expected ${CANONICAL_BYTES_LEN_V1}-byte output, produced ${offset}`,
    );
  }

  return buf;
}

/**
 * Version-aware dispatch. Reads spec_version, routes to the correct
 * canonical-bytes construction, throws SpecVersionUnknownError for
 * unrecognized versions per SPEC §3.1.1 fail-closed-on-unknown.
 *
 * @param {SwornAttestationV1Input | SwornAttestationV2Input} input
 * @param {number} specVersion
 * @returns {Buffer}
 */
export function serializeCanonicalBytes(input, specVersion) {
  if (specVersion === SPEC_VERSION_V0_1_PREVIEW) {
    return serializeCanonicalBytesV1(/** @type {SwornAttestationV1Input} */ (input));
  }
  if (specVersion === SPEC_VERSION_V0_1_FINAL) {
    return serializeCanonicalBytesV2(/** @type {SwornAttestationV2Input} */ (input));
  }
  throw new SpecVersionUnknownError(specVersion);
}
