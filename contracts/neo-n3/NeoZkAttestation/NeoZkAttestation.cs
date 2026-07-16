// Neo N3 smart-contract for neo-zkvm attestation settlement (Path A).
// Compile with Neo.SmartContract.Framework (Neo N3 DevPack).
//
// Architecture: SP1 verify off-chain → N-of-M secp256r1 ECDSA on Neo.
// Wire format MUST match crates/neo-zkvm-attestation (docs/neo-n3-attestation.md).
//
// BLS12-381: optional Path A′ / B / C only when CryptoLib exposes the needed
// natives AND the curve matches the proof system — not required for this contract.

using System;
using System.ComponentModel;
using System.Numerics;
using Neo;
using Neo.SmartContract.Framework;
using Neo.SmartContract.Framework.Attributes;
using Neo.SmartContract.Framework.Native;
using Neo.SmartContract.Framework.Services;

namespace Neo.Zkvm.Attestation
{
    [DisplayName("NeoZkAttestation")]
    [ManifestExtra("Author", "neo-zkvm")]
    [ManifestExtra("Description", "N-of-M ECDSA settlement for neo-zkvm public claims")]
    [ContractPermission("*", "*")]
    public class NeoZkAttestation : SmartContract
    {
        // proof_mode codes — must match ProofModeCode in Rust
        private const byte ModeExecute = 0;
        private const byte ModeMock = 1;
        private const byte ModeSp1 = 2;
        private const byte ModePlonk = 3;
        private const byte ModeGroth16 = 4;

        private static readonly byte[] DomainTag = "neo-zkvm-attestation-v1".ToByteArray();

        // Storage prefixes
        private static readonly byte[] PrefixConfig = new byte[] { 0x01 };
        private static readonly byte[] PrefixAttestor = new byte[] { 0x02 };
        private static readonly byte[] PrefixNonce = new byte[] { 0x03 };
        private static readonly byte[] KeyInitialized = new byte[] { 0x01, 0xFF };

        public static event Action<UInt256> ClaimSettled;
        public static event Action<string> SettlementFailed;

        /// <summary>
        /// One-time setup: program id, network magic, threshold, attestor pubkeys.
        /// Uncompressed SEC1 pubkeys (65 bytes, 0x04-prefixed) recommended.
        /// </summary>
        public static void Initialize(byte[] programId, uint networkMagic, BigInteger threshold, byte[][] attestorPubKeys)
        {
            if (Storage.Get(Storage.CurrentContext, KeyInitialized) != null)
                throw new Exception("already initialized");
            if (programId == null || programId.Length != 32)
                throw new Exception("programId must be 32 bytes");
            if (IsAllZero(programId))
                throw new Exception("programId must not be all zeros");
            if (threshold < 1)
                throw new Exception("threshold must be >= 1");
            if (attestorPubKeys == null || attestorPubKeys.Length == 0 || threshold > attestorPubKeys.Length)
                throw new Exception("invalid threshold / attestor set");

            for (int i = 0; i < attestorPubKeys.Length; i++)
            {
                var pk = attestorPubKeys[i];
                if (pk == null || (pk.Length != 33 && pk.Length != 65))
                    throw new Exception("attestor pubkey must be 33 or 65 bytes");
                // Reject duplicates in the committee set.
                for (int j = 0; j < i; j++)
                {
                    if (BytesEqual(pk, attestorPubKeys[j]))
                        throw new Exception("duplicate attestor pubkey");
                }
            }

            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }), programId);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x01 }), networkMagic);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }), threshold);

            for (int i = 0; i < attestorPubKeys.Length; i++)
            {
                Storage.Put(Storage.CurrentContext, PrefixAttestor.Concat(attestorPubKeys[i]), 1);
            }

            Storage.Put(Storage.CurrentContext, KeyInitialized, 1);
        }

        /// <summary>
        /// Submit an attestation bundle. Fixed-size fields are explicit so
        /// Neo contracts rebuild the digest without a full bincode parser.
        /// </summary>
        public static bool Submit(
            byte[] programId,
            byte proofMode,
            byte[] scriptHash,
            byte[] inputHash,
            byte[] outputHash,
            ulong gasConsumed,
            bool executionSuccess,
            byte[] appClaimHash,
            uint networkMagic,
            byte[] nonce,
            byte[][] publicKeys,
            byte[][] signatures)
        {
            if (Storage.Get(Storage.CurrentContext, KeyInitialized) == null)
            {
                SettlementFailed("not initialized");
                return false;
            }

            // --- validate field sizes ---
            if (programId == null || scriptHash == null || inputHash == null
                || outputHash == null || appClaimHash == null || nonce == null)
            {
                SettlementFailed("null field");
                return false;
            }
            if (programId.Length != 32 || scriptHash.Length != 32 || inputHash.Length != 32
                || outputHash.Length != 32 || appClaimHash.Length != 32 || nonce.Length != 32)
            {
                SettlementFailed("bad field length");
                return false;
            }
            if (publicKeys == null || signatures == null
                || publicKeys.Length != signatures.Length || publicKeys.Length == 0)
            {
                SettlementFailed("sig list mismatch");
                return false;
            }

            // --- reject non-production modes ---
            if (proofMode == ModeExecute || proofMode == ModeMock)
            {
                SettlementFailed("mock/execute not allowed");
                return false;
            }
            if (proofMode != ModeSp1 && proofMode != ModePlonk && proofMode != ModeGroth16)
            {
                SettlementFailed("unknown proof mode");
                return false;
            }

            // --- config checks ---
            var cfgProgram = Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }));
            var cfgMagic = (uint)(BigInteger)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x01 }));
            var threshold = (BigInteger)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }));

            if (!BytesEqual(programId, cfgProgram))
            {
                SettlementFailed("program_id mismatch");
                return false;
            }
            if (networkMagic != cfgMagic)
            {
                SettlementFailed("network magic mismatch");
                return false;
            }

            // --- replay protection ---
            var nonceKey = PrefixNonce.Concat(nonce);
            if (Storage.Get(Storage.CurrentContext, nonceKey) != null)
            {
                SettlementFailed("nonce replay");
                return false;
            }

            // --- recompute digest (must match Rust attestation_digest) ---
            byte[] digest = ComputeDigest(
                programId, proofMode, scriptHash, inputHash, outputHash,
                gasConsumed, executionSuccess, appClaimHash, networkMagic, nonce);

            // --- N-of-M ECDSA (secp256r1) with duplicate rejection ---
            BigInteger valid = 0;
            for (int i = 0; i < publicKeys.Length; i++)
            {
                var pk = publicKeys[i];
                var sig = signatures[i];
                if (pk == null || sig == null)
                {
                    SettlementFailed("null sig entry");
                    return false;
                }
                // Signature: 64-byte compact r||s (Neo also accepts ASN.1 on some builds).
                if (sig.Length != 64 && sig.Length != 65)
                {
                    SettlementFailed("bad signature length");
                    return false;
                }
                // Reject duplicate attestors in this submission.
                for (int j = 0; j < i; j++)
                {
                    if (BytesEqual(pk, publicKeys[j]))
                    {
                        SettlementFailed("duplicate attestor");
                        return false;
                    }
                }
                if (Storage.Get(Storage.CurrentContext, PrefixAttestor.Concat(pk)) == null)
                {
                    SettlementFailed("unauthorized attestor");
                    return false;
                }
                // Neo N3: VerifyWithECDsa hashes the message with SHA256 when using
                // secp256r1SHA256. Rust p256 Signer does the same over the 32-byte digest
                // (double-hash of the preimage). Keep both sides aligned.
                if (!CryptoLib.VerifyWithECDsa(digest, pk, sig, NamedCurveHash.secp256r1SHA256))
                {
                    SettlementFailed("bad signature");
                    return false;
                }
                valid += 1;
            }

            if (valid < threshold)
            {
                SettlementFailed("threshold not met");
                return false;
            }

            // --- commit nonce + settle ---
            Storage.Put(Storage.CurrentContext, nonceKey, 1);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x10 }), appClaimHash);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x11 }), digest);

            ClaimSettled((UInt256)digest);
            return true;
        }

        /// <summary>
        /// Canonical SHA256 digest — lockstep with neo_zkvm_attestation::attestation_digest.
        /// All multi-byte integers are little-endian.
        /// </summary>
        private static byte[] ComputeDigest(
            byte[] programId,
            byte proofMode,
            byte[] scriptHash,
            byte[] inputHash,
            byte[] outputHash,
            ulong gasConsumed,
            bool executionSuccess,
            byte[] appClaimHash,
            uint networkMagic,
            byte[] nonce)
        {
            var preimage = DomainTag
                .Concat(new byte[] { 0x00 })
                .Concat(programId)
                .Concat(new byte[] { proofMode })
                .Concat(scriptHash)
                .Concat(inputHash)
                .Concat(outputHash)
                .Concat(UInt64ToLeBytes(gasConsumed))
                .Concat(new byte[] { executionSuccess ? (byte)1 : (byte)0 })
                .Concat(appClaimHash)
                .Concat(UInt32ToLeBytes(networkMagic))
                .Concat(nonce);

            return CryptoLib.Sha256(preimage);
        }

        private static byte[] UInt64ToLeBytes(ulong value)
        {
            return new byte[]
            {
                (byte)value,
                (byte)(value >> 8),
                (byte)(value >> 16),
                (byte)(value >> 24),
                (byte)(value >> 32),
                (byte)(value >> 40),
                (byte)(value >> 48),
                (byte)(value >> 56)
            };
        }

        private static byte[] UInt32ToLeBytes(uint value)
        {
            return new byte[]
            {
                (byte)value,
                (byte)(value >> 8),
                (byte)(value >> 16),
                (byte)(value >> 24)
            };
        }

        private static bool BytesEqual(byte[] a, byte[] b)
        {
            if (a == null || b == null || a.Length != b.Length) return false;
            for (int i = 0; i < a.Length; i++)
            {
                if (a[i] != b[i]) return false;
            }
            return true;
        }

        private static bool IsAllZero(byte[] data)
        {
            for (int i = 0; i < data.Length; i++)
            {
                if (data[i] != 0) return false;
            }
            return true;
        }

        public static byte[] GetProgramId()
        {
            return Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }));
        }

        public static BigInteger GetThreshold()
        {
            return (BigInteger)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }));
        }

        public static byte[] GetLastAppClaimHash()
        {
            return Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x10 }));
        }

        public static byte[] GetLastDigest()
        {
            return Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x11 }));
        }

        public static bool IsNonceUsed(byte[] nonce)
        {
            if (nonce == null || nonce.Length != 32) return false;
            return Storage.Get(Storage.CurrentContext, PrefixNonce.Concat(nonce)) != null;
        }
    }
}
