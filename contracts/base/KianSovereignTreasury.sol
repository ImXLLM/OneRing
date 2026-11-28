// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// ============================================================================
/// KianSovereignTreasury — non-custodial MAIRON Sovereign Treasury & atomic
/// Day-1 4-Stream revenue splitter on Ethereum Base (Chain ID: 8453).
/// ============================================================================
///
/// CHAIN GUARD: the constructor and every external entrypoint assert
/// `block.chainid == 8453` — the MAIRON Sovereign Treasury settles ONLY on
/// Base Mainnet. Reverting message: "MAIRON: Base Mainnet 8453 required".
///
/// Settlement assets (closed whitelist, immutable at deployment):
///   * Base native ETH                       — sent as msg.value
///   * Base native USDC  0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 (6 dec)
///   * $0xKIA            ZeroXKiaToken      (18 dec)
///
/// The four Day-1 sovereign revenue streams:
///   * Stream 1 — 10% (1000 bps) mandatory platform charter tax on every
///     marketplace sale, routed to `sovereignTreasury`.
///   * Stream 2 — 10%–50% (1000..=5000 bps) programmable affiliate split.
///     Unreferred direct sale (`referrer == address(0)`): the default 1000 bps
///     affiliate leg is routed DIRECTLY to `sovereignTreasury`, giving the
///     platform 2000 bps (20%) total take and the creator 8000 bps (80%).
///   * Stream 3 — 100%-margin instant boosts (`purchaseInstantBoost`): the
///     whole payment routes to `sovereignTreasury` (10000 bps) — 100% Treasury,
///     0% Creator, matching `crates/wallet` `compute_instant_boost_split`.
///   * Stream 4 — 1.5% (150 bps) instant settlement & cross-chain withdrawal
///     spread, routed to `sovereignTreasury`.
///
/// Security doctrine:
///   * `nonReentrant` lock on every external-funds-touching entrypoint.
///   * Effects-then-interactions everywhere.
///   * Closed asset whitelist — arbitrary token pulls are impossible.
///   * `sovereignTreasury` and asset addresses are `immutable`.
///   * Double-entry ledger: every settlement emits one event carrying all
///     legs; `SUM(legs) == gross` holds by construction (the creator receives
///     the atomic remainder — zero rounding loss).
/// ============================================================================
interface IERC20Minimal {
    function transfer(address to, uint256 value) external returns (bool);
    function transferFrom(address from, address to, uint256 value) external returns (bool);
}

contract KianSovereignTreasury {
    // ── Charter constants (immutable by construction) ───────────────────────
    uint256 public constant CHARTER_TAX_BPS = 1000;               // Stream 1: 10%
    uint256 public constant AFFILIATE_MIN_BPS = 1000;             // Stream 2: 10%
    uint256 public constant AFFILIATE_MAX_BPS = 5000;             // Stream 2: 50%
    uint256 public constant UNREFERRED_AFFILIATE_BPS = 1000;      // Stream 2 default -> treasury
    uint256 public constant INSTANT_BOOST_BPS = 10_000;           // Stream 3: 100% Treasury / 0% Creator
    uint256 public constant WITHDRAWAL_SPREAD_BPS = 150;          // Stream 4: 1.5%
    uint256 public constant BPS_DENOMINATOR = 10_000;
    uint256 public constant CHAIN_ID = 8453;                      // Base Mainnet

    string public constant CHAIN_GUARD_MESSAGE = "MAIRON: Base Mainnet 8453 required";

    // ── Boost tiers (Stream 3, priced in Base-native USDC base units) ───────
    enum BoostTier { FeaturedSlot24h, MaironFastTrackAudit, ViralSwarmAmplifier }

    // ── Immutable deployment bindings ───────────────────────────────────────
    address public immutable sovereignTreasury;
    address public immutable usdcToken;   // 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 on Base
    address public immutable kiaToken;    // ZeroXKiaToken on Base

    enum AssetKind { NativeEth, Usdc, KiaToken }

    // ── Ledger state ────────────────────────────────────────────────────────
    /// Spendable balance per (asset, account) — creators withdraw from here.
    mapping(AssetKind => mapping(address => uint256)) public escrowBalance;

    bool private _entered;

    modifier nonReentrant() {
        require(!_entered, "MAIRON: reentrant call");
        _entered = true;
        _;
        _entered = false;
    }

    /// The Base-8453 chain guard — applied to every external entrypoint.
    modifier onlyBase8453() {
        require(block.chainid == 8453, "MAIRON: Base Mainnet 8453 required");
        _;
    }

    constructor(
        address _sovereignTreasury,
        address _usdcToken,
        address _kiaToken
    ) {
        require(block.chainid == 8453, "MAIRON: Base Mainnet 8453 required");
        require(_sovereignTreasury != address(0), "MAIRON: treasury required");
        require(_usdcToken != address(0), "MAIRON: usdc required");
        require(_kiaToken != address(0), "MAIRON: kia required");
        sovereignTreasury = _sovereignTreasury;
        usdcToken = _usdcToken;
        kiaToken = _kiaToken;
    }

    // ── Stream 1 + 2 — marketplace sale settlement ──────────────────────────
    /// Settles one marketplace sale: charter tax (10%) + affiliate split.
    /// Unreferred sales route the default affiliate leg to the treasury
    /// (20% treasury / 80% creator); referred sales route 10% referrer /
    /// 80% creator / 10% treasury.
    function settleMarketplaceSale(
        bytes32 saleId,
        address creator,
        address referrer,
        AssetKind asset,
        uint256 gross,
        uint256 affiliateBps
    ) external payable onlyBase8453 nonReentrant {
        require(gross > 0, "MAIRON: empty sale");
        require(affiliateBps >= AFFILIATE_MIN_BPS && affiliateBps <= AFFILIATE_MAX_BPS, "MAIRON: affiliate bps out of charter");
        _pull(asset, msg.sender, gross);

        uint256 taxLeg = (gross * CHARTER_TAX_BPS) / BPS_DENOMINATOR;
        uint256 affiliateLeg = (gross * (referrer == address(0) ? UNREFERRED_AFFILIATE_BPS : affiliateBps)) / BPS_DENOMINATOR;
        uint256 creatorLeg = gross - taxLeg - affiliateLeg;        // atomic remainder — zero loss

        if (referrer == address(0)) {
            // unreferred: 20% treasury (10% charter tax + default 10% affiliate leg) / 80% creator
            escrowBalance[asset][sovereignTreasury] += taxLeg + affiliateLeg;
        } else {
            // referred: 10% treasury / 10% referrer / 80% creator
            escrowBalance[asset][sovereignTreasury] += taxLeg;
            escrowBalance[asset][referrer] += affiliateLeg;
        }
        escrowBalance[asset][creator] += creatorLeg;

        emit MarketplaceSaleSettled(saleId, msg.sender, creator, referrer, asset, gross, taxLeg, affiliateLeg, creatorLeg);
    }

    // ── Stream 3 — instant boosts: 100% Treasury / 0% Creator ───────────────
    function purchaseInstantBoost(
        bytes32 boostId,
        address creator,
        AssetKind asset,
        BoostTier tier,
        uint256 gross
    ) external payable onlyBase8453 nonReentrant {
        require(gross > 0, "MAIRON: empty boost");
        _pull(asset, msg.sender, gross);
        uint256 treasuryLeg = (gross * INSTANT_BOOST_BPS) / BPS_DENOMINATOR; // == gross
        uint256 creatorLeg = gross - treasuryLeg;                            // == 0 by charter
        escrowBalance[asset][sovereignTreasury] += treasuryLeg;
        escrowBalance[asset][creator] += creatorLeg;                          // 0 — charter-honest
        emit InstantBoostPurchased(boostId, msg.sender, creator, asset, tier, gross, treasuryLeg, creatorLeg);
    }

    // ── Stream 4 — withdrawal with the 1.5% settlement spread ───────────────
    function withdrawWithSpread(AssetKind asset, uint256 amount) external onlyBase8453 nonReentrant {
        require(amount > 0 && amount <= escrowBalance[asset][msg.sender], "MAIRON: bad amount");
        uint256 spreadLeg = (amount * WITHDRAWAL_SPREAD_BPS) / BPS_DENOMINATOR;
        uint256 netLeg = amount - spreadLeg;
        escrowBalance[asset][msg.sender] -= amount;
        escrowBalance[asset][sovereignTreasury] += spreadLeg;
        _push(asset, msg.sender, netLeg);
        emit WithdrawalWithSpread(msg.sender, asset, amount, spreadLeg, netLeg);
    }

    // ── internal asset rails (closed whitelist) ─────────────────────────────
    function _pull(AssetKind asset, address from, uint256 value) private {
        if (asset == AssetKind.NativeEth) {
            require(msg.value == value, "MAIRON: eth mismatch");
        } else if (asset == AssetKind.Usdc) {
            require(IERC20Minimal(usdcToken).transferFrom(from, address(this), value), "MAIRON: usdc pull failed");
        } else {
            require(IERC20Minimal(kiaToken).transferFrom(from, address(this), value), "MAIRON: kia pull failed");
        }
    }

    function _push(AssetKind asset, address to, uint256 value) private {
        if (asset == AssetKind.NativeEth) {
            (bool ok, ) = to.call{value: value}("");
            require(ok, "MAIRON: eth push failed");
        } else if (asset == AssetKind.Usdc) {
            require(IERC20Minimal(usdcToken).transfer(to, value), "MAIRON: usdc push failed");
        } else {
            require(IERC20Minimal(kiaToken).transfer(to, value), "MAIRON: kia push failed");
        }
    }

    // ── events (double-entry: SUM(legs) == gross) ───────────────────────────
    event MarketplaceSaleSettled(
        bytes32 indexed saleId,
        address indexed buyer,
        address indexed creator,
        address referrer,
        AssetKind asset,
        uint256 gross,
        uint256 taxLeg,
        uint256 affiliateLeg,
        uint256 creatorLeg
    );
    event InstantBoostPurchased(
        bytes32 indexed boostId,
        address indexed buyer,
        address indexed creator,
        AssetKind asset,
        BoostTier tier,
        uint256 gross,
        uint256 treasuryLeg,
        uint256 creatorLeg
    );
    event WithdrawalWithSpread(
        address indexed account,
        AssetKind asset,
        uint256 amount,
        uint256 spreadLeg,
        uint256 netLeg
    );
}
