//! Neo zkVM CLI — development toolkit for execution, assembly, proving, and attestation.

use clap::{Parser, Subcommand, ValueEnum};
use neo_vm_guest::{ProofInput, bincode_serialize, deserialize_neoproof, execute};
use neo_vm_rs::{MAX_SCRIPT_SIZE, interpret_with_stack_and_syscalls};
use neo_zkvm_attestation::{
    AttestationBundle, AttestationConfig, AttestorKeypair, BundleJson, ClaimJson, SignatureJson,
    app_claim_hash, attestation_digest, claim_from_public_inputs, program_id_from_vkey_hash,
    random_nonce, sign_claim, verify_bundle,
};
use neo_zkvm_attestation::{ProofModeCode, parse_hex32};
use neo_zkvm_cli::{Assembler, Disassembler, Inspector, trace_host::TraceHost};
use neo_zkvm_prover::{NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::{verify_detailed_for_mode, verify_for_mode};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_GAS: u64 = 1_000_000;

/// Default prove mode: Mock when the binary is built without SP1 (crates.io /
/// plain `cargo install`), Sp1 when `--features sp1` is enabled so release
/// toolchains do not surprise operators with a silent mock default.
const fn default_cli_proof_mode() -> CliProofMode {
    if cfg!(feature = "sp1") {
        CliProofMode::Sp1
    } else {
        CliProofMode::Mock
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum CliProofMode {
    Execute,
    Mock,
    Sp1,
    Plonk,
    Groth16,
}

impl From<CliProofMode> for ProofMode {
    fn from(mode: CliProofMode) -> Self {
        match mode {
            CliProofMode::Execute => ProofMode::Execute,
            CliProofMode::Mock => ProofMode::Mock,
            CliProofMode::Sp1 => ProofMode::Sp1,
            CliProofMode::Plonk => ProofMode::Plonk,
            CliProofMode::Groth16 => ProofMode::Groth16,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "neo-zkvm",
    version,
    about = "Neo zkVM development toolkit",
    long_about = "Execute, assemble, disassemble, debug, inspect, and prove Neo VM scripts with shared neo-vm-rs semantics."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Execute a script and show results
    Run {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
        /// Gas / step budget
        #[arg(short = 'g', long = "gas", default_value_t = DEFAULT_GAS)]
        gas: u64,
    },
    /// Generate a ZK proof for script execution
    Prove {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
        /// Gas / step budget
        #[arg(short = 'g', long = "gas", default_value_t = DEFAULT_GAS)]
        gas: u64,
        /// Proof mode (default: mock without SP1 feature, sp1 with `--features sp1`)
        #[arg(short = 'm', long = "proof-mode", value_enum, default_value_t = default_cli_proof_mode())]
        proof_mode: CliProofMode,
        /// Allow mock fallback when SP1 is unavailable
        #[arg(long = "allow-fallback")]
        allow_fallback: bool,
        /// Write serialized NeoProof bytes to this path
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Assemble source code to bytecode
    Asm {
        /// Inline assembly or a .neoasm file path
        source: String,
    },
    /// Disassemble bytecode to readable form
    Disasm {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
    },
    /// Trace execution with shared neo-vm-rs semantics
    Debug {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
        /// Gas / step budget
        #[arg(short = 'g', long = "gas", default_value_t = DEFAULT_GAS)]
        gas: u64,
    },
    /// Analyze and display script information
    Inspect {
        /// Hex bytecode, or a .bin/.nef file
        script: String,
    },
    /// Verify a serialized NeoProof file (from `prove -o`)
    Verify {
        /// Path to a bincode-serialized NeoProof
        proof: PathBuf,
        /// Required proof mode — must match the proof (pinned; never attacker-chosen alone)
        #[arg(short = 'm', long = "proof-mode", value_enum)]
        proof_mode: CliProofMode,
    },
    /// Neo N3 settlement attestations (N-of-M ECDSA after off-chain proof verify)
    Attest {
        #[command(subcommand)]
        command: AttestCommands,
    },
}

#[derive(Debug, Subcommand)]
enum AttestCommands {
    /// Generate a secp256r1 attestor keypair (prints secret + public hex)
    Keygen {
        /// Write JSON keypair to this path
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Build an attestation claim + digest from a verified NeoProof
    Digest {
        /// Path to a bincode-serialized NeoProof (from `prove -o`)
        #[arg(long = "proof", value_name = "PATH")]
        proof: PathBuf,
        /// Required proof mode — must match the proof (pinned verification)
        #[arg(short = 'm', long = "proof-mode", value_enum)]
        proof_mode: CliProofMode,
        /// 32-byte program id hex (default: proof vkey_hash when non-zero)
        #[arg(long = "program-id")]
        program_id: Option<String>,
        /// Neo network magic (u32)
        #[arg(long = "network-magic")]
        network_magic: u32,
        /// 32-byte nonce hex (default: random)
        #[arg(long = "nonce")]
        nonce: Option<String>,
        /// Raw app claim bytes as hex → SHA-256 for app_claim_hash
        #[arg(long = "app-claim-hex", conflicts_with = "app_claim_string")]
        app_claim_hex: Option<String>,
        /// UTF-8 app claim string → SHA-256 for app_claim_hash
        #[arg(long = "app-claim-string", conflicts_with = "app_claim_hex")]
        app_claim_string: Option<String>,
        /// Explicit 32-byte app_claim_hash hex (skips hashing)
        #[arg(long = "app-claim-hash")]
        app_claim_hash_hex: Option<String>,
        /// Allow Mock/Execute modes (demo only; never for value settlement)
        #[arg(long = "allow-unsafe-mode")]
        allow_unsafe_mode: bool,
        /// Write claim JSON to this path
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Sign an attestation claim with a secp256r1 secret key
    Sign {
        /// Path to claim JSON (from `attest digest -o`)
        #[arg(long = "claim", value_name = "PATH")]
        claim: PathBuf,
        /// 32-byte secret key hex
        #[arg(long = "secret-key")]
        secret_key: String,
        /// Allow Mock/Execute modes (demo only)
        #[arg(long = "allow-unsafe-mode")]
        allow_unsafe_mode: bool,
        /// Write signature JSON to this path
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Build a full N-of-M attestation bundle from a proof + secret keys
    Bundle {
        /// Path to a bincode-serialized NeoProof
        #[arg(long = "proof", value_name = "PATH")]
        proof: PathBuf,
        /// Required proof mode — must match the proof
        #[arg(short = 'm', long = "proof-mode", value_enum)]
        proof_mode: CliProofMode,
        /// Operator config JSON (program_id, network_magic, threshold, attestors)
        #[arg(long = "config", value_name = "PATH")]
        config: PathBuf,
        /// One or more 32-byte secret key hex values (attestors that will sign)
        #[arg(long = "secret-key", required = true, num_args = 1..)]
        secret_keys: Vec<String>,
        /// 32-byte nonce hex (default: random)
        #[arg(long = "nonce")]
        nonce: Option<String>,
        /// Raw app claim bytes as hex → SHA-256
        #[arg(long = "app-claim-hex", conflicts_with = "app_claim_string")]
        app_claim_hex: Option<String>,
        /// UTF-8 app claim string → SHA-256
        #[arg(long = "app-claim-string", conflicts_with = "app_claim_hex")]
        app_claim_string: Option<String>,
        /// Explicit 32-byte app_claim_hash hex
        #[arg(long = "app-claim-hash")]
        app_claim_hash_hex: Option<String>,
        /// Allow Mock/Execute modes (demo only)
        #[arg(long = "allow-unsafe-mode")]
        allow_unsafe_mode: bool,
        /// Write bundle JSON to this path
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Verify an attestation bundle against operator config
    Check {
        /// Path to bundle JSON
        #[arg(long = "bundle", value_name = "PATH")]
        bundle: PathBuf,
        /// Operator config JSON
        #[arg(long = "config", value_name = "PATH")]
        config: PathBuf,
        /// Allow Mock/Execute modes (demo only)
        #[arg(long = "allow-unsafe-mode")]
        allow_unsafe_mode: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = dispatch(cli.command) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn dispatch(command: Commands) -> Result<(), String> {
    match command {
        Commands::Run { script, gas } => cmd_run(&script, gas),
        Commands::Prove {
            script,
            gas,
            proof_mode,
            allow_fallback,
            output,
        } => cmd_prove(&script, gas, proof_mode.into(), allow_fallback, output),
        Commands::Asm { source } => cmd_assemble(&source),
        Commands::Disasm { script } => cmd_disassemble(&script),
        Commands::Debug { script, gas } => cmd_debug(&script, gas),
        Commands::Inspect { script } => cmd_inspect(&script),
        Commands::Verify { proof, proof_mode } => cmd_verify(&proof, proof_mode.into()),
        Commands::Attest { command } => dispatch_attest(command),
    }
}

fn dispatch_attest(command: AttestCommands) -> Result<(), String> {
    match command {
        AttestCommands::Keygen { output } => cmd_attest_keygen(output),
        AttestCommands::Digest {
            proof,
            proof_mode,
            program_id,
            network_magic,
            nonce,
            app_claim_hex,
            app_claim_string,
            app_claim_hash_hex,
            allow_unsafe_mode,
            output,
        } => cmd_attest_digest(
            &proof,
            proof_mode.into(),
            program_id.as_deref(),
            network_magic,
            nonce.as_deref(),
            app_claim_hex.as_deref(),
            app_claim_string.as_deref(),
            app_claim_hash_hex.as_deref(),
            allow_unsafe_mode,
            output,
        ),
        AttestCommands::Sign {
            claim,
            secret_key,
            allow_unsafe_mode,
            output,
        } => cmd_attest_sign(&claim, &secret_key, allow_unsafe_mode, output),
        AttestCommands::Bundle {
            proof,
            proof_mode,
            config,
            secret_keys,
            nonce,
            app_claim_hex,
            app_claim_string,
            app_claim_hash_hex,
            allow_unsafe_mode,
            output,
        } => cmd_attest_bundle(
            &proof,
            proof_mode.into(),
            &config,
            &secret_keys,
            nonce.as_deref(),
            app_claim_hex.as_deref(),
            app_claim_string.as_deref(),
            app_claim_hash_hex.as_deref(),
            allow_unsafe_mode,
            output,
        ),
        AttestCommands::Check {
            bundle,
            config,
            allow_unsafe_mode,
        } => cmd_attest_check(&bundle, &config, allow_unsafe_mode),
    }
}

fn cmd_run(script_input: &str, gas_limit: u64) -> Result<(), String> {
    let script = parse_script(script_input)?;

    println!("Executing script with shared neo-vm-rs semantics...\n");

    let output = execute(ProofInput {
        script,
        arguments: vec![],
        gas_limit,
    });

    println!("=======================================");
    println!("  EXECUTION RESULT");
    println!("=======================================");
    println!(
        "  State:        {}",
        if output.state == 0 { "Halt" } else { "Fault" }
    );
    println!("  Gas consumed: {}", output.gas_consumed);
    println!("  Result:       {:?}", output.result);
    if let Some(ref error) = output.error {
        println!("  Error:        {error}");
    }
    println!("=======================================");

    if output.state != 0 {
        return Err(output
            .error
            .unwrap_or_else(|| "Script execution faulted".to_string()));
    }

    Ok(())
}

fn cmd_prove(
    script_input: &str,
    gas_limit: u64,
    proof_mode: ProofMode,
    allow_fallback: bool,
    output_path: Option<PathBuf>,
) -> Result<(), String> {
    if let Some(ref path) = output_path {
        validate_output_path(path)?;
    }

    let script = parse_script(script_input)?;

    if !allow_fallback
        && matches!(
            proof_mode,
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16
        )
        && cfg!(debug_assertions)
    {
        return Err(
            "Requested cryptographic proof mode requires a release build. Re-run with `cargo run --release --bin neo-zkvm -- prove ...` or pass --allow-fallback.".to_string(),
        );
    }

    println!("Generating ZK proof...\n");

    let input = ProofInput {
        script,
        arguments: vec![],
        gas_limit,
    };

    let prover = NeoProver::new(ProverConfig {
        proof_mode,
        allow_mock_fallback: allow_fallback,
        ..ProverConfig::default()
    });
    let proof = if allow_fallback {
        prover.prove(input)
    } else {
        prover.prove_strict(input)?
    };

    if should_error_on_fallback(proof_mode, proof.proof_mode, allow_fallback) {
        return Err(format!(
            "Requested proof mode {:?} but prover produced {:?}. Re-run with --allow-fallback to accept mock fallback, or fix SP1 setup.",
            proof_mode, proof.proof_mode
        ));
    }

    // Pin verification to the *actual* mode produced (after any allowed
    // fallback). Never use bare `verify()`, which accepts attacker-chosen Mock.
    let verified = verify_for_mode(&proof, proof.proof_mode);

    println!("=======================================");
    println!("  PROOF GENERATION RESULT");
    println!("=======================================");
    println!("  Requested: {proof_mode:?}");
    println!("  Mode:      {:?}", proof.proof_mode);
    println!("  Result:    {:?}", proof.output.result);
    println!("  Gas:       {}", proof.output.gas_consumed);
    println!("  Verified:  {verified}");
    println!("=======================================");

    if !verified {
        return Err("Proof verification failed; aborting.".to_string());
    }

    if let Some(path) = output_path {
        let bytes =
            bincode_serialize(&proof).map_err(|e| format!("Failed to serialize NeoProof: {e}"))?;
        fs::write(&path, &bytes)
            .map_err(|e| format!("Failed to write proof to '{}': {e}", path.display()))?;
        println!("Wrote {} proof bytes to {}", bytes.len(), path.display());
    }

    Ok(())
}

fn cmd_assemble(source_arg: &str) -> Result<(), String> {
    let source = if source_arg.ends_with(".neoasm") {
        fs::read_to_string(source_arg).map_err(|e| format!("Failed to read file: {e}"))?
    } else {
        source_arg.to_string()
    };

    let mut assembler = Assembler::new();
    let bytecode = assembler.assemble(&source)?;

    println!("{}", hex::encode(&bytecode));

    for warning in assembler.warnings() {
        eprintln!("Warning: {warning}");
    }

    Ok(())
}

fn cmd_disassemble(script_input: &str) -> Result<(), String> {
    let script = parse_script(script_input)?;
    let disasm = Disassembler::new(&script);
    println!("{}", disasm.disassemble());
    Ok(())
}

fn cmd_debug(script_input: &str, gas_limit: u64) -> Result<(), String> {
    let script = parse_script(script_input)?;
    let mut trace_host = TraceHost::new(script.clone(), gas_limit);

    println!("Tracing script with shared neo-vm-rs semantics...\n");
    let result = interpret_with_stack_and_syscalls(&script, Vec::new(), &mut trace_host);
    trace_host.print_trace();

    match result {
        Ok(result) => {
            println!("\nExecution result:");
            println!("  State: {:?}", result.state);
            println!("  Stack: {:?}", result.stack);
            println!("  Steps: {}", trace_host.steps_executed());
            if let Some(error) = &result.fault_message {
                println!("  Fault: {error}");
            }
            // Match `run`: non-zero exit on Fault for scripting automation.
            if !matches!(result.state, neo_vm_rs::VmState::Halt) {
                return Err(result
                    .fault_message
                    .unwrap_or_else(|| format!("Trace execution faulted: {:?}", result.state)));
            }
            Ok(())
        }
        Err(error) => Err(format!("Trace execution failed: {error}")),
    }
}

fn cmd_verify(proof_path: &Path, expected_mode: ProofMode) -> Result<(), String> {
    let bytes = fs::read(proof_path)
        .map_err(|e| format!("Failed to read proof '{}': {e}", proof_path.display()))?;

    // Bound proof file size before deserialization (DoS guard).
    const MAX_PROOF_FILE_BYTES: usize = 2 * 1024 * 1024;
    if bytes.len() > MAX_PROOF_FILE_BYTES {
        return Err(format!(
            "Proof file exceeds maximum size ({} > {MAX_PROOF_FILE_BYTES} bytes)",
            bytes.len()
        ));
    }

    let proof =
        deserialize_neoproof(&bytes).map_err(|e| format!("Failed to deserialize NeoProof: {e}"))?;

    let detailed = verify_detailed_for_mode(&proof, expected_mode);
    let simple = verify_for_mode(&proof, expected_mode);

    println!("=======================================");
    println!("  PROOF VERIFICATION");
    println!("=======================================");
    println!("  File:     {}", proof_path.display());
    println!("  Expected: {expected_mode:?}");
    println!("  Actual:   {:?}", proof.proof_mode);
    println!("  Type:     {:?}", detailed.proof_type);
    println!("  Valid:    {simple}");
    if let Some(err) = &detailed.error {
        println!("  Error:    {err}");
    }
    println!("  Gas:      {}", proof.output.gas_consumed);
    println!("  Result:   {:?}", proof.output.result);
    println!("=======================================");

    if !simple {
        return Err(detailed
            .error
            .unwrap_or_else(|| "Proof verification failed".to_string()));
    }
    Ok(())
}

fn cmd_inspect(script_input: &str) -> Result<(), String> {
    let script = parse_script(script_input)?;
    let inspector = Inspector::new(&script);
    println!("{}", inspector.analyze());
    Ok(())
}

fn cmd_attest_keygen(output: Option<PathBuf>) -> Result<(), String> {
    if let Some(ref path) = output {
        validate_output_path(path)?;
    }
    let kp = AttestorKeypair::generate();
    let secret = hex::encode(kp.to_bytes());
    let public = hex::encode(kp.public_key_compressed());
    let json = serde_json::json!({
        "curve": "secp256r1",
        "secret_key": secret,
        "public_key": public,
    });
    let pretty = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize keypair: {e}"))?;
    if let Some(path) = output {
        fs::write(&path, &pretty)
            .map_err(|e| format!("Failed to write keypair to '{}': {e}", path.display()))?;
        println!("Wrote attestor keypair to {}", path.display());
        println!("public_key = {public}");
    } else {
        println!("{pretty}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_attest_digest(
    proof_path: &Path,
    expected_mode: ProofMode,
    program_id_hex: Option<&str>,
    network_magic: u32,
    nonce_hex: Option<&str>,
    app_claim_hex: Option<&str>,
    app_claim_string: Option<&str>,
    app_claim_hash_hex: Option<&str>,
    allow_unsafe_mode: bool,
    output: Option<PathBuf>,
) -> Result<(), String> {
    if let Some(ref path) = output {
        validate_output_path(path)?;
    }
    let proof = load_and_verify_proof(proof_path, expected_mode)?;
    let mode_code = ProofModeCode::from(proof.proof_mode);
    if !allow_unsafe_mode && !mode_code.is_settlement_safe() {
        return Err(
            "Proof mode is Mock/Execute — not safe for settlement. \
             Re-prove with sp1/plonk/groth16, or pass --allow-unsafe-mode for demos only."
                .to_string(),
        );
    }

    let program_id = resolve_program_id(program_id_hex, &proof.vkey_hash)?;
    let nonce = resolve_nonce(nonce_hex)?;
    let app_hash = resolve_app_claim_hash(app_claim_hex, app_claim_string, app_claim_hash_hex)?;

    let claim = claim_from_public_inputs(
        program_id,
        mode_code,
        proof.public_inputs.clone(),
        app_hash,
        network_magic,
        nonce,
    )
    .map_err(|e| e.to_string())?;
    let digest = attestation_digest(&claim);
    let claim_json = ClaimJson::from(&claim);

    println!("=======================================");
    println!("  ATTESTATION DIGEST");
    println!("=======================================");
    println!("  Proof:        {}", proof_path.display());
    println!("  Mode:         {:?}", proof.proof_mode);
    println!("  Program id:   {}", hex::encode(program_id));
    println!("  Network magic:{network_magic}");
    println!("  Nonce:        {}", hex::encode(nonce));
    println!("  App claim:    {}", hex::encode(app_hash));
    println!("  Digest:       {}", hex::encode(digest));
    println!("=======================================");

    if let Some(path) = output {
        let pretty = serde_json::to_string_pretty(&claim_json)
            .map_err(|e| format!("Failed to serialize claim: {e}"))?;
        fs::write(&path, pretty)
            .map_err(|e| format!("Failed to write claim to '{}': {e}", path.display()))?;
        println!("Wrote claim JSON to {}", path.display());
    } else {
        let pretty = serde_json::to_string_pretty(&claim_json)
            .map_err(|e| format!("Failed to serialize claim: {e}"))?;
        println!("{pretty}");
    }
    Ok(())
}

fn cmd_attest_sign(
    claim_path: &Path,
    secret_key_hex: &str,
    allow_unsafe_mode: bool,
    output: Option<PathBuf>,
) -> Result<(), String> {
    if let Some(ref path) = output {
        validate_output_path(path)?;
    }
    let claim_json: ClaimJson = read_json(claim_path)?;
    let claim = claim_json.into_claim().map_err(|e| e.to_string())?;
    let keypair = AttestorKeypair::from_hex(secret_key_hex).map_err(|e| e.to_string())?;
    let sig = sign_claim(&keypair, &claim, allow_unsafe_mode).map_err(|e| e.to_string())?;
    let sig_json = SignatureJson::from(&sig);
    let digest = attestation_digest(&claim);

    println!("=======================================");
    println!("  ATTESTATION SIGNATURE");
    println!("=======================================");
    println!("  Digest:     {}", hex::encode(digest));
    println!("  Public key: {}", hex::encode(&sig.public_key));
    println!("  Signature:  {}", hex::encode(&sig.signature));
    println!("=======================================");

    let pretty = serde_json::to_string_pretty(&sig_json)
        .map_err(|e| format!("Failed to serialize signature: {e}"))?;
    if let Some(path) = output {
        fs::write(&path, pretty)
            .map_err(|e| format!("Failed to write signature to '{}': {e}", path.display()))?;
        println!("Wrote signature JSON to {}", path.display());
    } else {
        println!("{pretty}");
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_attest_bundle(
    proof_path: &Path,
    expected_mode: ProofMode,
    config_path: &Path,
    secret_keys: &[String],
    nonce_hex: Option<&str>,
    app_claim_hex: Option<&str>,
    app_claim_string: Option<&str>,
    app_claim_hash_hex: Option<&str>,
    allow_unsafe_mode: bool,
    output: Option<PathBuf>,
) -> Result<(), String> {
    if let Some(ref path) = output {
        validate_output_path(path)?;
    }
    let config: AttestationConfig = read_json(config_path)?;
    let parsed = config.parse().map_err(|e| e.to_string())?;
    let proof = load_and_verify_proof(proof_path, expected_mode)?;
    let mode_code = ProofModeCode::from(proof.proof_mode);
    if !allow_unsafe_mode && !mode_code.is_settlement_safe() {
        return Err(
            "Proof mode is Mock/Execute — not safe for settlement. \
             Use production proof modes or --allow-unsafe-mode for demos only."
                .to_string(),
        );
    }

    // Config program_id is authoritative for the committee.
    if let Some(from_vkey) = program_id_from_vkey_hash(&proof.vkey_hash)
        && from_vkey != parsed.program_id
    {
        eprintln!(
            "Warning: proof vkey_hash ({}) differs from config program_id ({})",
            hex::encode(from_vkey),
            hex::encode(parsed.program_id)
        );
    }

    let nonce = resolve_nonce(nonce_hex)?;
    let app_hash = resolve_app_claim_hash(app_claim_hex, app_claim_string, app_claim_hash_hex)?;
    let claim = claim_from_public_inputs(
        parsed.program_id,
        mode_code,
        proof.public_inputs.clone(),
        app_hash,
        parsed.network_magic,
        nonce,
    )
    .map_err(|e| e.to_string())?;

    let mut signatures = Vec::with_capacity(secret_keys.len());
    for sk in secret_keys {
        let kp = AttestorKeypair::from_hex(sk).map_err(|e| e.to_string())?;
        signatures.push(sign_claim(&kp, &claim, allow_unsafe_mode).map_err(|e| e.to_string())?);
    }
    let bundle = AttestationBundle { claim, signatures };
    verify_bundle(&bundle, &parsed, allow_unsafe_mode).map_err(|e| e.to_string())?;

    let bundle_json = BundleJson::from(&bundle);
    let digest = attestation_digest(&bundle.claim);

    println!("=======================================");
    println!("  ATTESTATION BUNDLE");
    println!("=======================================");
    println!("  Proof:     {}", proof_path.display());
    println!("  Mode:      {:?}", proof.proof_mode);
    println!("  Signers:   {}", bundle.signatures.len());
    println!("  Threshold: {}", parsed.threshold);
    println!("  Digest:    {}", hex::encode(digest));
    println!("  Status:    verified against config");
    println!("=======================================");

    let pretty = serde_json::to_string_pretty(&bundle_json)
        .map_err(|e| format!("Failed to serialize bundle: {e}"))?;
    if let Some(path) = output {
        fs::write(&path, pretty)
            .map_err(|e| format!("Failed to write bundle to '{}': {e}", path.display()))?;
        println!("Wrote bundle JSON to {}", path.display());
    } else {
        println!("{pretty}");
    }
    Ok(())
}

fn cmd_attest_check(
    bundle_path: &Path,
    config_path: &Path,
    allow_unsafe_mode: bool,
) -> Result<(), String> {
    let config: AttestationConfig = read_json(config_path)?;
    let parsed = config.parse().map_err(|e| e.to_string())?;
    let bundle_json: BundleJson = read_json(bundle_path)?;
    let bundle = bundle_json.into_bundle().map_err(|e| e.to_string())?;
    verify_bundle(&bundle, &parsed, allow_unsafe_mode).map_err(|e| e.to_string())?;
    let digest = attestation_digest(&bundle.claim);

    println!("=======================================");
    println!("  ATTESTATION CHECK");
    println!("=======================================");
    println!("  Bundle:    {}", bundle_path.display());
    println!("  Config:    {}", config_path.display());
    println!("  Mode:      {}", bundle.claim.proof_mode.as_str());
    println!("  Signers:   {}", bundle.signatures.len());
    println!("  Threshold: {}", parsed.threshold);
    println!("  Digest:    {}", hex::encode(digest));
    println!("  Valid:     true");
    println!("=======================================");
    Ok(())
}

fn load_and_verify_proof(
    proof_path: &Path,
    expected_mode: ProofMode,
) -> Result<neo_vm_guest::NeoProof, String> {
    let bytes = fs::read(proof_path)
        .map_err(|e| format!("Failed to read proof '{}': {e}", proof_path.display()))?;
    const MAX_PROOF_FILE_BYTES: usize = 2 * 1024 * 1024;
    if bytes.len() > MAX_PROOF_FILE_BYTES {
        return Err(format!(
            "Proof file exceeds maximum size ({} > {MAX_PROOF_FILE_BYTES} bytes)",
            bytes.len()
        ));
    }
    let proof =
        deserialize_neoproof(&bytes).map_err(|e| format!("Failed to deserialize NeoProof: {e}"))?;
    if !verify_for_mode(&proof, expected_mode) {
        let detailed = verify_detailed_for_mode(&proof, expected_mode);
        return Err(detailed
            .error
            .unwrap_or_else(|| "Proof verification failed".to_string()));
    }
    Ok(proof)
}

fn resolve_program_id(
    program_id_hex: Option<&str>,
    vkey_hash: &[u8; 32],
) -> Result<[u8; 32], String> {
    if let Some(hex_str) = program_id_hex {
        return parse_hex32(hex_str).map_err(|e| e.to_string());
    }
    program_id_from_vkey_hash(vkey_hash).ok_or_else(|| {
        "program_id required: proof vkey_hash is zero (Mock/Execute). \
         Pass --program-id <hex32> for demos, or use SP1 proofs with a real vkey."
            .to_string()
    })
}

fn resolve_nonce(nonce_hex: Option<&str>) -> Result<[u8; 32], String> {
    match nonce_hex {
        Some(h) => parse_hex32(h).map_err(|e| e.to_string()),
        None => Ok(random_nonce()),
    }
}

fn resolve_app_claim_hash(
    app_claim_hex: Option<&str>,
    app_claim_string: Option<&str>,
    app_claim_hash_hex: Option<&str>,
) -> Result<[u8; 32], String> {
    if let Some(h) = app_claim_hash_hex {
        return parse_hex32(h).map_err(|e| e.to_string());
    }
    if let Some(hex_data) = app_claim_hex {
        let raw = neo_zkvm_attestation::parse_hex_bytes(hex_data).map_err(|e| e.to_string())?;
        return Ok(app_claim_hash(&raw));
    }
    if let Some(s) = app_claim_string {
        return Ok(app_claim_hash(s.as_bytes()));
    }
    // Default empty claim (still domain-separated by other fields).
    Ok(app_claim_hash(b""))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read '{}': {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("Failed to parse JSON '{}': {e}", path.display()))
}

fn parse_script(input: &str) -> Result<Vec<u8>, String> {
    if input.ends_with(".nef") {
        return Err(format!(
            "NEF container files are not supported yet ('{input}'). \
             Extract the raw script bytes and pass a .bin file or hex bytecode."
        ));
    }
    if input.ends_with(".bin") {
        let metadata =
            fs::metadata(input).map_err(|e| format!("Failed to read file '{input}': {e}"))?;
        if metadata.len() > MAX_SCRIPT_SIZE as u64 {
            return Err(format!(
                "Script file exceeds maximum size of {MAX_SCRIPT_SIZE} bytes"
            ));
        }
        let content = fs::read(input).map_err(|e| format!("Failed to read file '{input}': {e}"))?;
        // Defense-in-depth: re-check after read in case the file was replaced
        // between metadata() and read() (TOCTOU).
        if content.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script content exceeds maximum size of {MAX_SCRIPT_SIZE} bytes"
            ));
        }
        Ok(content)
    } else {
        if (input.contains('.') || input.contains('/')) && !input.starts_with("0x") {
            return Err(format!(
                "'{input}' does not look like hex or a known script file — use a .bin file or hex bytes (optionally 0x-prefixed)"
            ));
        }
        let hex_str = input.trim_start_matches("0x");
        let decoded = hex::decode(hex_str).map_err(|e| format!("Invalid hex string: {e}"))?;
        if decoded.len() > MAX_SCRIPT_SIZE {
            return Err(format!(
                "Script exceeds maximum size of {MAX_SCRIPT_SIZE} bytes"
            ));
        }
        Ok(decoded)
    }
}

fn validate_output_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Output path must not be empty".to_string());
    }
    if path.is_dir() {
        return Err(format!(
            "Output path '{}' is a directory; provide a file path",
            path.display()
        ));
    }
    Ok(())
}

fn should_error_on_fallback(
    requested_mode: ProofMode,
    actual_mode: ProofMode,
    allow_fallback: bool,
) -> bool {
    !allow_fallback
        && matches!(
            requested_mode,
            ProofMode::Sp1 | ProofMode::Plonk | ProofMode::Groth16
        )
        && actual_mode != requested_mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_cli(args: &[&str]) -> Result<Cli, String> {
        Cli::try_parse_from(std::iter::once("neo-zkvm").chain(args.iter().copied()))
            .map_err(|e| e.to_string())
    }

    #[test]
    fn test_parse_proof_mode_default_depends_on_sp1_feature() {
        let cli = parse_cli(&["prove", "12139E40"]).unwrap();
        match cli.command {
            Commands::Prove { proof_mode, .. } => {
                let expected = if cfg!(feature = "sp1") {
                    ProofMode::Sp1
                } else {
                    ProofMode::Mock
                };
                assert_eq!(ProofMode::from(proof_mode), expected);
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_parse_attest_keygen() {
        let cli = parse_cli(&["attest", "keygen", "-o", "k.json"]).unwrap();
        match cli.command {
            Commands::Attest {
                command: AttestCommands::Keygen { output },
            } => {
                assert_eq!(output.unwrap(), PathBuf::from("k.json"));
            }
            _ => panic!("expected attest keygen"),
        }
    }

    #[test]
    fn test_parse_attest_digest_requires_proof_and_mode() {
        let err = parse_cli(&["attest", "digest"]).unwrap_err();
        assert!(
            err.contains("required") || err.contains("proof") || err.contains("proof-mode"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn test_parse_attest_check() {
        let cli = parse_cli(&[
            "attest",
            "check",
            "--bundle",
            "b.json",
            "--config",
            "c.json",
            "--allow-unsafe-mode",
        ])
        .unwrap();
        match cli.command {
            Commands::Attest {
                command:
                    AttestCommands::Check {
                        bundle,
                        config,
                        allow_unsafe_mode,
                    },
            } => {
                assert_eq!(bundle, PathBuf::from("b.json"));
                assert_eq!(config, PathBuf::from("c.json"));
                assert!(allow_unsafe_mode);
            }
            _ => panic!("expected attest check"),
        }
    }

    #[test]
    fn test_parse_script_rejects_nef_containers() {
        let err = parse_script("contract.nef").unwrap_err();
        assert!(err.contains("NEF"), "unexpected: {err}");
    }

    #[test]
    fn test_parse_proof_mode_accepts_all_modes() {
        let cases = [
            ("execute", ProofMode::Execute),
            ("mock", ProofMode::Mock),
            ("sp1", ProofMode::Sp1),
            ("plonk", ProofMode::Plonk),
            ("groth16", ProofMode::Groth16),
        ];

        for (mode_str, expected_mode) in cases {
            let cli = parse_cli(&["prove", "12139E40", "--proof-mode", mode_str]).unwrap();
            match cli.command {
                Commands::Prove { proof_mode, .. } => {
                    assert_eq!(ProofMode::from(proof_mode), expected_mode);
                }
                _ => panic!("expected prove"),
            }
        }
    }

    #[test]
    fn test_parse_proof_mode_accepts_short_alias() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "mock"]).unwrap();
        match cli.command {
            Commands::Prove { proof_mode, .. } => {
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Mock);
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_parse_proof_mode_rejects_invalid_mode() {
        let err = parse_cli(&["prove", "12139E40", "--proof-mode", "bad-mode"]).unwrap_err();
        assert!(
            err.to_ascii_lowercase().contains("invalid")
                || err.to_ascii_lowercase().contains("possible values"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_proof_mode_requires_value() {
        let err = parse_cli(&["prove", "12139E40", "--proof-mode"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("proof-mode"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_proof_mode_requires_value_short_alias() {
        let err = parse_cli(&["prove", "12139E40", "-m"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("proof-mode")
                || err.contains("-m"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_requested_proof_mode_detects_explicit_mode() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "sp1"]).unwrap();
        match cli.command {
            Commands::Prove { proof_mode, .. } => {
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Sp1);
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_parse_allow_fallback_flag() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "sp1", "--allow-fallback"]).unwrap();
        match cli.command {
            Commands::Prove { allow_fallback, .. } => assert!(allow_fallback),
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_should_error_on_fallback_for_crypto_modes() {
        assert!(should_error_on_fallback(
            ProofMode::Sp1,
            ProofMode::Mock,
            false,
        ));
        assert!(!should_error_on_fallback(
            ProofMode::Sp1,
            ProofMode::Mock,
            true,
        ));
        assert!(!should_error_on_fallback(
            ProofMode::Mock,
            ProofMode::Mock,
            false,
        ));
    }

    #[test]
    fn test_parse_gas_limit_requires_value() {
        let err = parse_cli(&["run", "12139E40", "--gas"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("gas"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_cmd_prove_requires_release_for_crypto_mode_without_fallback() {
        if !cfg!(debug_assertions) {
            return;
        }
        let err = cmd_prove("12139E40", DEFAULT_GAS, ProofMode::Sp1, false, None).unwrap_err();
        assert!(err.contains("requires a release build"));
    }

    #[test]
    fn test_parse_output_path_requires_value() {
        let err = parse_cli(&["prove", "12139E40", "-m", "mock", "--output"]).unwrap_err();
        assert!(
            err.contains("requires a value")
                || err.contains("a value is required")
                || err.contains("output"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_parse_output_path_accepts_file() {
        let cli = parse_cli(&["prove", "12139E40", "-m", "mock", "-o", "proof.bin"]).unwrap();
        match cli.command {
            Commands::Prove { output, .. } => {
                assert_eq!(
                    output.as_deref().map(Path::as_os_str),
                    Some("proof.bin".as_ref())
                );
            }
            _ => panic!("expected prove"),
        }
    }

    #[test]
    fn test_cmd_run_fails_on_fault() {
        // PUSH5 PUSH0 DIV RET (division by zero)
        let err = cmd_run("1510A140", DEFAULT_GAS).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_cmd_debug_fails_on_fault() {
        let err = cmd_debug("1510A140", DEFAULT_GAS).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_cmd_prove_strict_fails_on_fault() {
        let err = cmd_prove("1510A140", DEFAULT_GAS, ProofMode::Mock, false, None).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_cli_run_parses_gas() {
        let cli = parse_cli(&["run", "12139E40", "-g", "42"]).unwrap();
        match cli.command {
            Commands::Run { gas, .. } => assert_eq!(gas, 42),
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn test_cli_verify_requires_mode() {
        let err = parse_cli(&["verify", "proof.bin"]).unwrap_err();
        assert!(
            err.contains("proof-mode") || err.contains("required"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_cli_verify_parses() {
        let cli = parse_cli(&["verify", "proof.bin", "-m", "mock"]).unwrap();
        match cli.command {
            Commands::Verify { proof, proof_mode } => {
                assert_eq!(proof.as_os_str(), "proof.bin");
                assert_eq!(ProofMode::from(proof_mode), ProofMode::Mock);
            }
            _ => panic!("expected verify"),
        }
    }

    #[test]
    fn test_prove_and_verify_file_roundtrip() {
        let dir =
            std::env::temp_dir().join(format!("neo-zkvm-proof-roundtrip-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("proof.bin");
        let _ = fs::remove_file(&path);

        cmd_prove(
            "12139E40",
            DEFAULT_GAS,
            ProofMode::Mock,
            false,
            Some(path.clone()),
        )
        .expect("prove should succeed");

        cmd_verify(&path, ProofMode::Mock).expect("verify should accept matching mode");
        let err = cmd_verify(&path, ProofMode::Sp1).unwrap_err();
        assert!(err.contains("Proof mode mismatch") || err.contains("mismatch"));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
