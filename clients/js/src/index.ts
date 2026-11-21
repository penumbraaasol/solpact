// Hand-written helpers on top of the Codama-generated client (./generated — never edit by hand).
import { getAddressEncoder, getProgramDerivedAddress, getU64Encoder, type Address } from "@solana/kit";
import { PACT_PROGRAM_ADDRESS } from "./generated/index.js";

export * from "./generated/index.js";

/** PDA for a pact: seeds = ["pact", creator, id (u64 LE)]. */
export async function findPactPda(creator: Address, id: bigint): Promise<readonly [Address, number]> {
  return getProgramDerivedAddress({
    programAddress: PACT_PROGRAM_ADDRESS,
    seeds: ["pact", getAddressEncoder().encode(creator), getU64Encoder().encode(id)],
  });
}
