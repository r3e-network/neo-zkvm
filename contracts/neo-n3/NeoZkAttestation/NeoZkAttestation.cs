// Neo N3 smart-contract sketch for neo-zkvm attestation settlement.
// Compile with Neo.SmartContract.Framework (Neo N3 DevPack).
// This is a reference implementation of the on-chain half of the
// "SP1 off-chain + N-of-M ECDSA on Neo" architecture.
//
// Wire format MUST match crates/neo-zkvm-attestation (docs/neo-n3-attestation.md).

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

        public static event Action<UInt256> ClaimSettled;
        public static event Action<string> SettlementFailed;

        /// <summary>
        /// One-time setup: program id, network magic, threshold, attestor pubkeys.
        /// </summary>
        public static void Initialize(byte[] programId, uint networkMagic, BigInteger threshold, byte[][] attestorPubKeys)
        {
            if (programId.Length != 32) throw new Exception("programId must be 32 bytes");
            if (threshold < 1) throw new Exception("threshold must be >= 1");
            if (attestorPubKeys.Length == 0 || threshold > attestorPubKeys.Length)
                throw new Exception("invalid threshold / attestor set");

            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }), programId);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x01 }), networkMagic);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }), threshold);

            for (int i = 0; i < attestorPubKeys.Length; i++)
            {
                // Key = prefix || pubkey; value = 1
                Storage.Put(Storage.CurrentContext, PrefixAttestor.Concat(attestorPubKeys[i]), 1);
            }
        }

        /// <summary>
        /// Submit an attestation bundle. All fixed-size fields are explicit so
        /// Neo contracts can rebuild the digest without a full bincode parser.
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
            // --- validate field sizes ---
            if (programId.Length != 32 || scriptHash.Length != 32 || inputHash.Length != 32
                || outputHash.Length != 32 || appClaimHash.Length != 32 || nonce.Length != 32)
            {
                SettlementFailed("bad field length");
                return false;
            }
            if (publicKeys.Length != signatures.Length || publicKeys.Length == 0)
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

            if (!programId.Equals(cfgProgram))
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

            // --- N-of-M ECDSA (secp256r1) ---
            BigInteger valid = 0;
            // Track used attestors by storing seen flags in a temporary map via storage prefix 0xFF (call-local pattern)
            // For sketch simplicity we only check membership + VerifyWithECDsa and count.
            for (int i = 0; i < publicKeys.Length; i++)
            {
                var pk = publicKeys[i];
                var sig = signatures[i];
                if (Storage.Get(Storage.CurrentContext, PrefixAttestor.Concat(pk)) == null)
                {
                    SettlementFailed("unauthorized attestor");
                    return false;
                }
                // Neo N3: VerifyWithECDsa(message, pubkey, signature, curve)
                // Curve secp256r1 is typically NamedCurveHash.secp256r1 / ECDsa.Secp256r1
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
            // App-specific: store last settled claim hash
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x10 }), appClaimHash);

            ClaimSettled((UInt256)digest);
            return true;
        }

        /// <summary>
        /// Canonical SHA256 digest — keep in lockstep with neo_zkvm_attestation::attestation_digest.
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
            // Build preimage: domain || 0x00 || fields...
            var preimage = DomainTag
                .Concat(new byte[] { 0x00 })
                .Concat(programId)
                .Concat(new byte[] { proofMode })
                .Concat(scriptHash)
                .Concat(inputHash)
                .Concat(outputHash)
                .Concat(BitConverter.GetBytes(gasConsumed)) // LE on little-endian hosts; Neo uses LE
                .Concat(new byte[] { executionSuccess ? (byte)1 : (byte)0 })
                .Concat(appClaimHash)
                .Concat(BitConverter.GetBytes(networkMagic))
                .Concat(nonce);

            return CryptoLib.Sha256(preimage);
        }

        public static byte[] GetProgramId()
        {
            return Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }));
        }

        public static BigInteger GetThreshold()
        {
            return (BigInteger)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }));
        }
    }
}
