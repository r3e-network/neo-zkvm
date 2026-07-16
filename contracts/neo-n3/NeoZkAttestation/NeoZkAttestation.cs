// Neo N3 attestation settlement — Path A (N-of-M ECDSA after off-chain SP1 verify).
// Wire format matches crates/neo-zkvm-attestation (docs/neo-n3-attestation.md).

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
    public class NeoZkAttestation : Neo.SmartContract.Framework.SmartContract
    {
        private const byte ModeExecute = 0;
        private const byte ModeMock = 1;
        private const byte ModeSp1 = 2;
        private const byte ModePlonk = 3;
        private const byte ModeGroth16 = 4;

        private static readonly byte[] DomainTag = "neo-zkvm-attestation-v1".ToByteArray();

        private static readonly byte[] PrefixConfig = new byte[] { 0x01 };
        private static readonly byte[] PrefixAttestor = new byte[] { 0x02 };
        private static readonly byte[] PrefixNonce = new byte[] { 0x03 };
        private static readonly byte[] KeyInitialized = new byte[] { 0x01, 0xFF };

        public static event Action<UInt256> ClaimSettled;
        public static event Action<string> SettlementFailed;

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
                for (int j = 0; j < i; j++)
                {
                    if (BytesEqual(pk, attestorPubKeys[j]))
                        throw new Exception("duplicate attestor pubkey");
                }
            }

            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }), (ByteString)programId);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x01 }), networkMagic);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }), threshold);

            for (int i = 0; i < attestorPubKeys.Length; i++)
            {
                // Neo storage keys max 64 bytes; 0x02‖65-byte SEC1 is too long.
                // Store by SHA256(pubkey) so both compressed (33) and uncompressed (65) fit.
                Storage.Put(Storage.CurrentContext, AttestorKey(attestorPubKeys[i]), 1);
            }

            Storage.Put(Storage.CurrentContext, KeyInitialized, 1);
        }

        private static byte[] AttestorKey(byte[] pubkey)
        {
            return PrefixAttestor.Concat((byte[])CryptoLib.Sha256((ByteString)pubkey));
        }

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

            var cfgProgram = (byte[])(ByteString)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }));
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

            var nonceKey = PrefixNonce.Concat(nonce);
            if (Storage.Get(Storage.CurrentContext, nonceKey) != null)
            {
                SettlementFailed("nonce replay");
                return false;
            }

            byte[] digest = ComputeDigest(
                programId, proofMode, scriptHash, inputHash, outputHash,
                gasConsumed, executionSuccess, appClaimHash, networkMagic, nonce);

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
                if (sig.Length != 64 && sig.Length != 65)
                {
                    SettlementFailed("bad signature length");
                    return false;
                }
                for (int j = 0; j < i; j++)
                {
                    if (BytesEqual(pk, publicKeys[j]))
                    {
                        SettlementFailed("duplicate attestor");
                        return false;
                    }
                }
                if (Storage.Get(Storage.CurrentContext, AttestorKey(pk)) == null)
                {
                    SettlementFailed("unauthorized attestor");
                    return false;
                }
                if (!CryptoLib.VerifyWithECDsa(
                        (ByteString)digest,
                        (ECPoint)(ByteString)pk,
                        (ByteString)sig,
                        NamedCurveHash.secp256r1SHA256))
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

            Storage.Put(Storage.CurrentContext, nonceKey, 1);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x10 }), (ByteString)appClaimHash);
            Storage.Put(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x11 }), (ByteString)digest);

            ClaimSettled((UInt256)(ByteString)digest);
            return true;
        }

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

            return (byte[])CryptoLib.Sha256((ByteString)preimage);
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

        [Safe]
        public static byte[] GetProgramId()
        {
            return (byte[])(ByteString)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x00 }));
        }

        [Safe]
        public static BigInteger GetThreshold()
        {
            return (BigInteger)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x02 }));
        }

        [Safe]
        public static byte[] GetLastAppClaimHash()
        {
            return (byte[])(ByteString)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x10 }));
        }

        [Safe]
        public static byte[] GetLastDigest()
        {
            return (byte[])(ByteString)Storage.Get(Storage.CurrentContext, PrefixConfig.Concat(new byte[] { 0x11 }));
        }

        [Safe]
        public static bool IsNonceUsed(byte[] nonce)
        {
            if (nonce == null || nonce.Length != 32) return false;
            return Storage.Get(Storage.CurrentContext, PrefixNonce.Concat(nonce)) != null;
        }
    }
}
