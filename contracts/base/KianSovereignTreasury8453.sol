// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// ============================================================================
/// KianSovereignTreasury8453 — the Base-Mainnet-bound deployment variant of
/// `KianSovereignTreasury` (Chain ID: 8453 hardcoded at construction).
/// ============================================================================
///
/// This variant exists so the canonical deployment artifact can never be
/// mistaken for a testnet or L2-fork deployment: the chain-id guard is part of
/// the type itself, re-asserted in the constructor and inherited on every
/// external entrypoint through `onlyBase8453`.
///
///   require(block.chainid == 8453, "MAIRON: Base Mainnet 8453 required");
///
/// Revenue charter (identical to the parent contract):
///   * Stream 1 — 10% (1000 bps) mandatory platform charter tax.
///   * Stream 2 — 10%–50% programmable affiliate split; unreferred sales give
///     the platform 2000 bps (20%) and the creator 8000 bps (80%).
///   * Stream 3 — instant boosts route 100% (10000 bps) to the MAIRON
///     Sovereign Treasury and 0% to the creator.
///   * Stream 4 — 1.5% (150 bps) settlement & withdrawal spread.
/// ============================================================================
import { KianSovereignTreasury } from "./KianSovereignTreasury.sol";

contract KianSovereignTreasury8453 is KianSovereignTreasury {
    /// Immutably binds the deployment to Base Mainnet (chain id 8453).
    /// Any deployment attempt on another chain reverts at construction.
    constructor(
        address _sovereignTreasury,
        address _usdcToken,
        address _kiaToken
    )
        KianSovereignTreasury(_sovereignTreasury, _usdcToken, _kiaToken)
    {
        // The parent constructor already ran the Base-8453 guard; this explicit
        // re-assertion makes the deployment contract self-documenting.
        require(block.chainid == 8453, "MAIRON: Base Mainnet 8453 required");
        require(block.chainid == CHAIN_ID, "MAIRON: Base Mainnet 8453 required");
    }

    /// Hardcoded Base Mainnet chain id — the deployment constant of this variant.
    uint256 public constant DEPLOYMENT_CHAIN_ID = 8453;
}
