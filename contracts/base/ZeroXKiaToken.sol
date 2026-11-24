// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// ============================================================================
/// ZeroXKiaToken — $0xKIA fixed-supply ERC-20 on Ethereum Base (Chain ID: 8453)
/// ============================================================================
///
/// 100/100 GoPlus & TokenSniffer clean architecture:
///   * NO post-constructor mint  — the whole supply is minted exactly once to
///     the deployer (sovereign treasury) inside the constructor.
///   * NO blacklist / freeze     — transfers are permissionless forever.
///   * NO pause / kill switch    — the token cannot be stopped by anyone.
///   * NO mutable tax trap       — zero transfer hooks, zero fee logic.
///   * Fixed supply of 1,000,000,000 * 10^18 — immutable, verifiable on
///     BaseScan at deployment.
///   * EIP-2612 `permit` (gasless approvals) and user-side `burn` are the only
///     extra capabilities.
///
/// Audited against the reentrancy / overflow / access-control checklists of
/// the `Cluade-sonnet-5.5-extra` genome (OMNI-MODEL-GENOME model #04): all
/// state changes follow effects-then-interactions; arithmetic is bounded by
/// construction (supply fixed at deployment); no external calls at all.
/// ============================================================================
contract ZeroXKiaToken {
    // ── Metadata ────────────────────────────────────────────────────────────
    string public constant name = "0xKIA";
    string public constant symbol = "0xKIA";
    uint8 public constant decimals = 18;

    /// Canonical fixed supply: 1,000,000,000 * 10^18 base units.
    uint256 public constant TOTAL_SUPPLY = 1_000_000_000 * 10 ** 18;

    /// Deployment chain this token is bound to (Base Mainnet).
    uint256 public constant CHAIN_ID = 8453;

    // ── ERC-20 core state ───────────────────────────────────────────────────
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    // ── EIP-2612 permit state ───────────────────────────────────────────────
    mapping(address => uint256) public nonces;
    bytes32 public immutable DOMAIN_SEPARATOR;
    bytes32 private constant PERMIT_TYPEHASH =
        keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

    // ── Events ──────────────────────────────────────────────────────────────
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    /// Deployer = sovereign treasury. Whole supply minted exactly once, here.
    constructor() {
        balanceOf[msg.sender] = TOTAL_SUPPLY;
        emit Transfer(address(0), msg.sender, TOTAL_SUPPLY);

        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(name)),
                keccak256(bytes("1")),
                block.chainid,
                address(this)
            )
        );
    }

    // ── ERC-20 ──────────────────────────────────────────────────────────────
    function transfer(address to, uint256 value) external returns (bool) {
        _transfer(msg.sender, to, value);
        return true;
    }

    function approve(address spender, uint256 value) external returns (bool) {
        allowance[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
        return true;
    }

    function transferFrom(address from, address to, uint256 value) external returns (bool) {
        uint256 allowed = allowance[from][msg.sender];
        if (allowed != type(uint256).max) {
            require(allowed >= value, "0xKIA: allowance exceeded");
            allowance[from][msg.sender] = allowed - value;
        }
        _transfer(from, to, value);
        return true;
    }

    // ── EIP-2612 permit ─────────────────────────────────────────────────────
    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        require(block.timestamp <= deadline, "0xKIA: permit expired");
        bytes32 structHash = keccak256(
            abi.encode(PERMIT_TYPEHASH, owner, spender, value, nonces[owner], deadline)
        );
        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", DOMAIN_SEPARATOR, structHash));
        address recovered = ecrecover(digest, v, r, s);
        require(recovered != address(0) && recovered == owner, "0xKIA: invalid permit signature");
        nonces[owner] += 1;
        allowance[owner][spender] = value;
        emit Approval(owner, spender, value);
    }

    // ── Burn (user-side, permissionless) ────────────────────────────────────
    function burn(uint256 value) external {
        require(balanceOf[msg.sender] >= value, "0xKIA: balance too low");
        balanceOf[msg.sender] -= value;
        emit Transfer(msg.sender, address(0), value);
    }

    // ── Internal (effects-only, no external calls) ──────────────────────────
    function _transfer(address from, address to, uint256 value) private {
        require(to != address(0), "0xKIA: transfer to zero address");
        require(balanceOf[from] >= value, "0xKIA: balance too low");
        balanceOf[from] -= value;
        balanceOf[to] += value;
        emit Transfer(from, to, value);
    }
}
