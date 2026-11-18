# ammq32026: Constant-Product AMM (Solana / Anchor)

Week 3 assignment for the Turbin3 Q4 2026 Builders Cohort.

## Overview

A constant-product (x·y = k) AMM program that swaps between two tokens, X and Y.

| Instruction | Purpose |
|---|---|
| `initialize` | Creates the pool: config, LP mint, vaults (X and Y), and the treasury ATAs (X and Y) |
| `deposit` | Adds liquidity and mints LP tokens to the depositor |
| `withdraw` | Burns LP tokens and returns X and Y to the withdrawer |
| `swap` | Swaps X for Y, or Y for X |

## Fees

- The fee is set by `fee` in basis points. The tests use 30 (0.3%).
- On swap, the fee is subtracted from the input amount first, and the remainder is used for the swap. The same formula applies in both directions.
- The fee is sent to the treasury ATA (treasury_x for X input, treasury_y for Y input).
- The maximum fee is 10,000 (100%). Initializing with a higher fee fails.

## Tests

```bash
anchor test
```

| Test | What it checks |
|---|---|
| `test_initialize` | Initialization succeeds |
| `test_deposit` | Adding liquidity succeeds |
| `test_withdraw` | Removing liquidity succeeds |
| `test_swap` | An X-to-Y swap succeeds |
| `test_swap_fee_goes_to_treasury` | After an X-to-Y swap, treasury_x receives the fee of 30,000 |
| `test_swap_y_fee_goes_to_treasury` | After a Y-to-X swap, treasury_y receives the fee of 30,000 |
| `test_initialize_fails_when_fee_too_high` | Initialization fails when fee is 10,001 |
| `test_swap_fails_when_slippage_exceeded` | A swap that requires more than the output allows fails |

## Notes

- The fee is calculated as a percentage of the swap input. The program subtracts the fee itself and does not rely on the fee calculated inside the curve crate.
- The failure-case tests only check that the call fails. They do not check the specific error.
- The optional challenge (own CPMM implementation and downtime analysis) is not included.