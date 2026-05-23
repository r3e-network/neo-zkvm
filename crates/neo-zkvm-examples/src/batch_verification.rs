use neo_vm_guest::{OpCode, ProofInput, StackItem};
use neo_zkvm_prover::{NeoProof, NeoProver, ProofMode, ProverConfig};
use neo_zkvm_verifier::verify_detailed;

#[path = "batch_verification/batch_summary.rs"]
mod batch_summary;

use batch_summary::BatchSummary;

#[derive(Clone)]
struct BatchJob {
    name: &'static str,
    input: ProofInput,
    expected_result: i128,
}

fn build_arithmetic_jobs() -> Vec<BatchJob> {
    vec![
        BatchJob {
            name: "add_2_plus_3",
            input: ProofInput {
                script: vec![
                    OpCode::PUSH2.byte(),
                    OpCode::PUSH3.byte(),
                    OpCode::ADD.byte(),
                    OpCode::RET.byte(),
                ],
                arguments: vec![],
                gas_limit: 1_000_000,
            },
            expected_result: 5,
        },
        BatchJob {
            name: "mul_5_times_4",
            input: ProofInput {
                script: vec![
                    OpCode::PUSH5.byte(),
                    OpCode::PUSH4.byte(),
                    OpCode::MUL.byte(),
                    OpCode::RET.byte(),
                ],
                arguments: vec![],
                gas_limit: 1_000_000,
            },
            expected_result: 20,
        },
        BatchJob {
            name: "square_7",
            input: ProofInput {
                script: vec![OpCode::DUP.byte(), OpCode::MUL.byte(), OpCode::RET.byte()],
                arguments: vec![StackItem::Integer(7)],
                gas_limit: 1_000_000,
            },
            expected_result: 49,
        },
    ]
}

fn prove_jobs(prover: &NeoProver, jobs: &[BatchJob]) -> Vec<NeoProof> {
    jobs.iter()
        .map(|job| prover.prove(job.input.clone()))
        .collect()
}

fn proof_result_integer(proof: &NeoProof) -> Option<i128> {
    proof.output.result.as_ref().and_then(|item| item.to_i128())
}

fn verify_jobs(jobs: &[BatchJob], proofs: &[NeoProof]) -> BatchSummary {
    let mut valid = 0usize;
    let mut invalid_jobs = Vec::new();

    for (job, proof) in jobs.iter().zip(proofs.iter()) {
        let verification = verify_detailed(proof);
        let result_matches = proof_result_integer(proof) == Some(job.expected_result);

        if verification.valid && result_matches {
            valid += 1;
        } else {
            invalid_jobs.push(job.name.to_string());
        }
    }

    if proofs.len() < jobs.len() {
        for job in &jobs[proofs.len()..] {
            invalid_jobs.push(job.name.to_string());
        }
    }

    if proofs.len() > jobs.len() {
        for index in jobs.len()..proofs.len() {
            invalid_jobs.push(format!("unexpected_proof_{index}"));
        }
    }

    BatchSummary {
        total: jobs.len(),
        valid,
        invalid_jobs,
    }
}

fn main() {
    let prover = NeoProver::new(ProverConfig {
        proof_mode: ProofMode::Mock,
        ..Default::default()
    });

    let jobs = build_arithmetic_jobs();
    let proofs = prove_jobs(&prover, &jobs);
    let summary = verify_jobs(&jobs, &proofs);

    println!("=== Batch Verification Example ===");
    for (job, proof) in jobs.iter().zip(proofs.iter()) {
        println!("{} => {:?}", job.name, proof.output.result);
    }
    println!("Total jobs: {}", summary.total);
    println!("Valid proofs: {}", summary.valid);
    println!("Invalid jobs: {}", summary.invalid_jobs.len());

    assert_eq!(summary.valid, summary.total);
    assert!(summary.invalid_jobs.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_verification_accepts_valid_proofs() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });

        let jobs = build_arithmetic_jobs();
        let proofs = prove_jobs(&prover, &jobs);
        let summary = verify_jobs(&jobs, &proofs);

        assert_eq!(summary.valid, summary.total);
        assert!(summary.invalid_jobs.is_empty());
    }

    #[test]
    fn test_batch_verification_rejects_tampered_proof() {
        let prover = NeoProver::new(ProverConfig {
            proof_mode: ProofMode::Mock,
            ..Default::default()
        });

        let jobs = build_arithmetic_jobs();
        let mut proofs = prove_jobs(&prover, &jobs);

        proofs[0].output.result = Some(StackItem::Integer(999));

        let summary = verify_jobs(&jobs, &proofs);

        assert!(summary.valid < summary.total);
        assert_eq!(summary.invalid_jobs, vec![jobs[0].name.to_string()]);
    }
}
