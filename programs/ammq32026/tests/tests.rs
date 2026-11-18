use {
    anchor_lang::InstructionData,
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::{get_spl_account, spl_token::state::Account as TokenAccount, CreateMint},
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

// Setup function to initialize LiteSVM and create a payer keypair
fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = ammq32026::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/ammq32026.so"
    ));

    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
    // This done using litesvm-token's CreateMint utility which creates the mint in the LiteSVM environment
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let config =
        Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &ammq32026::id()).0;
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &ammq32026::id()).0;

    // Derive the PDA for the vault associated token account using the config PDA and Mint A
    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    (
        svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
    )
}

#[test]
fn test_initialize() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);
    
    let instruction = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_deposit() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    
    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);
    
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
pub fn test_withdraw() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();

    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);
    
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm,  &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let withdraw_ix = create_withdraw_ix(
        &payer, mint_x, mint_y,mint_lp, config, vault_x, vault_y,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, withdraw_ix], &payer, &[&payer]);
    assert!(res.is_ok());            
}

#[test]
pub fn test_swap() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    
    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);
    
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,treasury,treasury_x,treasury_y,);
    let deposit_ix = create_deposit_ix(
        &mut svm,  &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,);
    let swap_ix = create_swap_ix(
        &payer, mint_x, mint_y,mint_lp, config, vault_x, vault_y,treasury_x,treasury_y,);
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());            
}

#[test]
pub fn test_swap_fee_goes_to_treasury() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    let init_ix = create_initialise_ix(&mut svm, &payer, mint_x, mint_y, config, mint_lp,  vault_x, vault_y,
        treasury, treasury_x, treasury_y
    );
    let deposit_ix = create_deposit_ix(&mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_ix(&payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury_x, treasury_y,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    //swapの入力は 10_000_000、手数料は 30bps（0.3%）なので、手数料は 30_000
    let expected_fee = 10_000_000u64 * 30 / 10_000;
    let fee_x = get_spl_account::<TokenAccount>(&svm, &treasury_x).unwrap().amount;
    assert_eq!(fee_x, expected_fee);

}

#[test]
pub fn test_swap_y_fee_goes_to_treasury() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    let init_ix = create_initialise_ix(&mut svm, &payer, mint_x, mint_y, config, mint_lp,  vault_x, vault_y,
        treasury, treasury_x, treasury_y
    );
    let deposit_ix = create_deposit_ix(&mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let swap_ix = create_swap_y_ix(&payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury_x, treasury_y,
    );
    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    //Y側のswapでも手数料は 30bps（0.3%）なので、treasuryには 30_000 が入る
    let expected_fee = 10_000_000u64 * 30 / 10_000;
    let fee_y = get_spl_account::<TokenAccount>(&svm, &treasury_y).unwrap().amount;
    assert_eq!(fee_y, expected_fee);

}

#[test]
pub fn test_initialize_fails_when_fee_too_high() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    let mut init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
        treasury, treasury_x, treasury_y,
    );
    // fee の上限は 10_000（100%）。10_001 にすると失敗するはず
    init_ix.data = ammq32026::instruction::Initialize {
        seed: 123,
        fee: 10_001,
        authority: Some(payer.pubkey()),
    }
    .data();

    let res = send(&mut svm, &[init_ix], &payer, &[&payer]);
    assert!(res.is_err());
}

#[test]
pub fn test_swap_fails_when_slippage_exceeded() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y) = setup();
    let treasury = Pubkey::new_unique();
    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y,
        treasury, treasury_x, treasury_y,
    );
    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let mut swap_ix = create_swap_ix(
        &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury_x, treasury_y,
    );
    // 受け取れる量より大きい min を要求すると、slippage で失敗するはず
    swap_ix.data = ammq32026::instruction::Swap {
        is_x: true,
        amount: 10_000_000,
        min: u64::MAX,
    }
    .data();

    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_err());
}
