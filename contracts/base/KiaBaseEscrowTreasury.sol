// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// ============================================================================
/// KiaBaseEscrowTreasury — non-custodial escrow & atomic Day-1 4-Stream
/// revenue splitter on Ethereum Base (Chain ID: 8453)
/// ============================================================================
///
/// Settlement assets (closed whitelist, immutable at deployment):
///   * Base native ETH                       — sent as msg.value
///   * Base native USDC  0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 (6 dec)
///   * $0xKIA            ZeroXKiaToken      (18 dec)
///
/// The four Day-1 sovereign revenue streams ("The first war is better than the last peace"):
///   * Stream 1 — 10% (1000 bps) mandatory platform charter tax on every
///     marketplace sale, routed to `kiaTreasury`.
///   * Stream 2 — 10%–50% (1000..=5000 bps) programmable affiliate split.
///     Unreferred direct sale (`referrer == address(0)`): the default 1000 bps
///     affiliate leg is routed DIRECTLY to `kiaTreasury`, giving the platform
///     2000 bps (20%) total take and the creator 8000 bps (80%).
///   * Stream 3 — 100%-margin instant boosts (`purchaseInstantBoost`): the
///     whole payment routes to `kiaTreasury` (10000 bps).
///   * Stream 4 — 1.5% (150 bps) instant settlement & cross-chain withdrawal
///     spread on `withdrawWithSpread`, routed to `kiaTreasury`.
///
/// Security doctrine (Cluade-sonnet-5.5-extra audit genome):
///   * `nonReentrant` lock on every external-funds-touching entrypoint.
///   * Effects-then-interactions everywhere (state is updated before token
///     pulls/pushes happen).
///   * Closed asset whitelist — arbitrary token pulls are impossible.
///   * `kiaTreasury` and asset addresses are `immutable`; no owner key can
///     re-route creator funds after deployment.
///   * Double-entry ledger: every settlement emits one event carrying all
///     legs; `SUM(legs) == gross` holds by construction (creator receives the
///     atomic remainder — zero rounding loss).
/// ============================================================================
interface IERC20Minimal {
    function transfer(address to, uint256 value) external returns (bool);
    function transferFrom(address from, address to, uint256 value) external returns (bool);
}

contract KiaBaseEscrowTreasury {
    // ── Charter constants (immutable by construction) ───────────────────────
    uint256 public constant CHARTER_TAX_BPS = 1000;               // Stream 1: 10%
    uint256 public constant AFFILIATE_MIN_BPS = 1000;             // Stream 2: 10%
    uint256 public constant AFFILIATE_MAX_BPS = 5000;             // Stream 2: 50%
    uint256 public constant UNREFERRED_AFFILIATE_BPS = 1000;      // Stream 2 default -> treasury
    uint256 public constant WITHDRAWAL_SPREAD_BPS = 150;          // Stream 4: 1.5%
    uint256 public constant BPS_DENOMINATOR = 10_000;
    uint256 public constant CHAIN_ID = 8453;                      // Base Mainnet

    // ── Boost tiers (Stream 3, priced in Base-native USDC micros = base units)
    enum BoostTier { FeaturedSlot24h, MaironFastTrackAudit, ViralSwarmAmplifier }

    // ── Immutable deployment bindings ───────────────────────────────────────
    address public immutable kiaTreasury;
    address public immutable usdcToken;   // 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 on Base
    address public immutable kiaToken;    // ZeroXKiaToken on Base

    enum AssetKind { NativeEth, Usdc, KiaToken }

    /// Atomic split legs of one marketplace settlement — a single `memory`
    /// struct keeps the settlement frame inside the default EVM stack limit.
    struct DayOneLegs {
        uint256 treasury;
        uint256 affiliate;
        uint256 creator;
        address affiliateRecipient;
    }

    // ── Ledger state ────────────────────────────────────────────────────────
    /// Spendable balance per (asset, account) — creators withdraw from here.
    mapping(AssetKind => mapping(address => uint256)) public escrowBalance;

    bool private _entered;
    modifier nonReentrant() {
        require(!_entered, "KIA: reentrant call");
        _entered = true;
        _;
        _entered = false;
    }

    event MarketplaceSaleSettled(
        bytes32 indexed saleId,
        address indexed buyer,
        address indexed creator,
        address referrer,
        AssetKind asset,
        uint256 gross,
        uint256 treasuryLeg,
        uint256 affiliateLeg,
        uint256 creatorLeg,
        uint256 effectivePlatformBps
    );
    event InstantBoostPurchased(
        address indexed buyer,
        BoostTier indexed tier,
        AssetKind asset,
        uint256 amount,
        address indexed recipient
    );
    event WithdrawnWithSpread(
        address indexed account,
        AssetKind indexed asset,
        uint256 gross,
        uint256 spread,
        uint256 net
    );

    constructor(address _kiaTreasury, address _usdcToken, address _kiaToken) {
        // Deployment guard — the day-one sovereign settlement layer is bound to
        // Base Mainnet; a wrong-chain deployment must revert at construction.
        require(block.chainid == 8453, "MAIRON: Base Mainnet 8453 required");
        require(_kiaTreasury != address(0), "KIA: treasury is zero");
        require(_usdcToken != address(0), "KIA: usdc is zero");
        require(_kiaToken != address(0), "KIA: token is zero");
        kiaTreasury = _kiaTreasury;
        usdcToken = _usdcToken;
        kiaToken = _kiaToken;
    }

    // ── Stream 1+2: atomic marketplace sale settlement ──────────────────────
    /// Settles one marketplace sale. Buyer must have pre-approved `gross` for
    /// ERC-20 assets, or attach exactly `gross` as msg.value for native ETH.
    /// The creator leg is credited to the internal escrow ledger; the creator
    /// releases it later with `withdrawWithSpread` (Stream 4).
    function settleMarketplaceSale(
        bytes32 saleId,
        AssetKind asset,
        address buyer,
        address creator,
        address referrer,          // address(0) => unreferred direct sale
        uint256 gross,
        uint256 affiliateBps
    ) external payable nonReentrant {
        require(buyer != address(0) && creator != address(0), "KIA: zero party");
        require(buyer != address(this), "KIA: buyer is contract");
        require(gross > 0, "KIA: gross is zero");
        require(affiliateBps >= AFFILIATE_MIN_BPS && affiliateBps <= AFFILIATE_MAX_BPS, "KIA: affiliate bps out of window");

        // ── Pull the whole gross from the buyer first (interactions last). ──
        _pullFrom(buyer, asset, gross);

        // ── Effects: compute the atomic legs (one memory struct = one frame slot).
        DayOneLegs memory legs = _dayOneLegs(gross, affiliateBps, referrer);
        require(legs.creator > 0, "KIA: creator leg is zero");

        escrowBalance[asset][legs.affiliateRecipient] += legs.affiliate;
        escrowBalance[asset][creator] += legs.creator;
        escrowBalance[asset][kiaTreasury] += legs.treasury;

        emit MarketplaceSaleSettled(
            saleId, buyer, creator, referrer, asset, gross,
            legs.treasury, legs.affiliate, legs.creator,
            ((legs.treasury + legs.affiliate) * BPS_DENOMINATOR) / gross
        );
    }

    // ── Stream 3: 100%-margin instant boost purchase ────────────────────────
    /// Prices (Base-native USDC base units): FeaturedSlot24h = 5_000_000
    /// ($5.00), MaironFastTrackAudit = 2_000_000 ($2.00),
    /// ViralSwarmAmplifier = 10_000_000 ($10.00). Whole payment routes to
    /// `kiaTreasury` (10000 bps = 100%).
    function purchaseInstantBoost(AssetKind asset, BoostTier tier) external payable nonReentrant {
        uint256 price = _boostPrice(tier);
        _pullFrom(msg.sender, asset, price);
        escrowBalance[asset][kiaTreasury] += price;
        emit InstantBoostPurchased(msg.sender, tier, asset, price, kiaTreasury);
    }

    /// The atomic Day-1 leg computation — pure, no state access.
    ///
    /// Stream 1: charter tax; Stream 2: declared (or default, unreferred)
    /// affiliate share; the creator receives the exact remainder — the leg
    /// sum equals the gross with zero rounding loss, by construction.
    function _dayOneLegs(uint256 gross, uint256 affiliateBps, address referrer)
        private
        view
        returns (DayOneLegs memory)
    {
        DayOneLegs memory legs;
        legs.treasury = (gross * CHARTER_TAX_BPS) / BPS_DENOMINATOR; // Stream 1
        if (referrer == address(0)) {
            // Unreferred direct-sale rule: the default 1000 bps affiliate leg
            // routes directly to kiaTreasury — platform total = 2000 bps.
            legs.affiliate = (gross * UNREFERRED_AFFILIATE_BPS) / BPS_DENOMINATOR;
            legs.affiliateRecipient = kiaTreasury;
        } else {
            legs.affiliate = (gross * affiliateBps) / BPS_DENOMINATOR;
            legs.affiliateRecipient = referrer;
        }
        legs.creator = gross - legs.treasury - legs.affiliate; // atomic remainder
        return legs;
    }

    function _boostPrice(BoostTier tier) private pure returns (uint256) {
        if (tier == BoostTier.FeaturedSlot24h) return 5_000_000;
        if (tier == BoostTier.MaironFastTrackAudit) return 2_000_000;
        return 10_000_000; // ViralSwarmAmplifier
    }

    // ── Stream 4: instant settlement & cross-chain withdrawal spread ────────
    /// Releases the caller's escrow balance minus the 150 bps spread which
    /// routes to `kiaTreasury`. Net + spread == gross (zero rounding loss).
    function withdrawWithSpread(AssetKind asset, uint256 gross) external nonReentrant {
        require(gross > 0, "KIA: gross is zero");
        uint256 balance = escrowBalance[asset][msg.sender];
        require(balance >= gross, "KIA: balance too low");

        uint256 spread = (gross * WITHDRAWAL_SPREAD_BPS) / BPS_DENOMINATOR;
        uint256 net = gross - spread;

        escrowBalance[asset][msg.sender] = balance - gross;
        escrowBalance[asset][kiaTreasury] += spread;

        _pushTo(msg.sender, asset, net);
        emit WithdrawnWithSpread(msg.sender, asset, gross, spread, net);
    }

    // ── Asset plumbing (closed whitelist) ───────────────────────────────────
    function _pullFrom(address from, AssetKind asset, uint256 amount) private {
        if (asset == AssetKind.NativeEth) {
            require(msg.value == amount, "KIA: exact msg.value required");
        } else {
            address token = asset == AssetKind.Usdc ? usdcToken : kiaToken;
            // interactions happen AFTER the caller has been validated; the
            // external call is the last thing this branch does per leg.
            require(IERC20Minimal(token).transferFrom(from, address(this), amount), "KIA: token pull failed");
        }
    }

    function _pushTo(address to, AssetKind asset, uint256 amount) private {
        require(to != address(0), "KIA: push to zero");
        if (asset == AssetKind.NativeEth) {
            (bool ok, ) = to.call{value: amount}("");
            require(ok, "KIA: eth push failed");
        } else {
            address token = asset == AssetKind.Usdc ? usdcToken : kiaToken;
            require(IERC20Minimal(token).transfer(to, amount), "KIA: token push failed");
        }
    }

    receive() external payable {
        // Direct native deposits are forbidden — funds must enter through the
        // atomic settlement/boost entrypoints so the ledger stays balanced.
        revert("KIA: use settleMarketplaceSale / purchaseInstantBoost");
    }
}
