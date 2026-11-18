# ammq32026: 定積型 AMM（Solana / Anchor）

Turbin3 Q4 2026 Builders Cohort の Week 3 課題です。

## 概要

X と Y の 2 つのトークンを交換する、定積（x·y=k）型の AMM プログラムです。

| 命令 | 役割 |
|---|---|
| `initialize` | プールを作る。config、LP トークン、vault（X・Y）、treasury の ATA（X・Y）を作る |
| `deposit` | 流動性を入れて LP トークンを受け取る |
| `withdraw` | LP トークンを燃やして、X・Y を受け取る |
| `swap` | X を Y に、または Y を X に交換する |

## 手数料

- 手数料は `fee`（basis points）で指定します。テストでは 30（0.3%）です。
- swap では、入力量から手数料を先に引き、残りで交換します。両方向で同じ式を使います。
- 手数料は、treasury の ATA に入ります（X 側は treasury_x、Y 側は treasury_y）。
- 手数料の上限は 10,000（100%）です。これを超える `fee` で初期化すると失敗します。

## テスト

```bash
anchor test
```

| テスト | 確かめること |
|---|---|
| `test_initialize` | 初期化が成功する |
| `test_deposit` | 流動性の追加が成功する |
| `test_withdraw` | 流動性の引き出しが成功する |
| `test_swap` | X 側の swap が成功する |
| `test_swap_fee_goes_to_treasury` | X 側の swap で、treasury_x に手数料 30,000 が入る |
| `test_swap_y_fee_goes_to_treasury` | Y 側の swap で、treasury_y に手数料 30,000 が入る |
| `test_initialize_fails_when_fee_too_high` | fee が 10,001 のとき、初期化が失敗する |
| `test_swap_fails_when_slippage_exceeded` | min を超える要求の swap が失敗する |

## 注意点

- 手数料の計算は、swap の入力量に対する割合です。curve crate の内部の手数料は使わず、プログラムで先に引いています。
- 失敗ケースのテストは、失敗したことだけを確かめています。失敗の理由（エラーの種類）までは確かめていません。
- 任意課題（CPMM の自前実装、ダウンタイムの考察）は含みません。